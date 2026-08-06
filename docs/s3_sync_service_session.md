# Compte-rendu de session — Conception du service de synchronisation S3

> **Date** : 2026-08-06 · **Branche** : `perf/crop-dimensions-for-square-plate-format`
> **Plan produit** : [`s3_sync_service_plan.md`](s3_sync_service_plan.md)
> **Portée** : session de conception uniquement — **aucun code modifié**.

Ce document rejoue le raisonnement de la session : l'état des lieux du code, les questions ouvertes, et **pourquoi** chaque décision a été prise. Il existe pour que la reprise du travail sur une autre machine ne reparte pas de zéro sur les arbitrages déjà tranchés.

---

## 1. Demande initiale

Implémenter un service de synchronisation entre le cache S3 et les données de plaques issues de chaque recherche effectuée par un agent, afin qu'**aucune recherche n'échappe à la sauvegarde** dans le bucket S3. But : enrichir le cache et garantir la continuité de service quand l'API intermédiaire est indisponible.

**Comportement attendu côté recherche agent :**

| Situation | Comportement |
|---|---|
| API externe up, données trouvées | Répondre à l'agent, sérialiser une copie et l'uploader sous `vehicle-cache/` |
| API externe up, donnée inexistante | L'agent filme la carte grise et la soumet au backoffice |
| API externe down, donnée en cache | Lire `vehicle-cache/`, désérialiser, répondre à l'agent |
| API externe down, cache vide | Écrire la plaque sous `retry-queue/`, répondre « erreur de recherche, veuillez réessayer » |

**Comportement attendu côté service de synchronisation :**

- Cycle de 3 h : 1 h de fenêtre active avec sonde toutes les 5 min, puis 2 h de repos.
- Si l'API est up : lister `retry-queue/`, récupérer les données de chaque immatriculation, sérialiser, uploader dans `vehicle-cache/`.
- Si une immatriculation n'existe pas au registre : l'uploader sous `unregistered/`, pour que le backoffice l'affiche dans « pending validation » sans image (juste le numéro).
- Après traitement, retirer l'immatriculation de la file.
- File vide → ne rien faire.

---

## 2. État des lieux du code existant

Exploration menée sur `iviss-backend/`, `iviss-mock-ext-api/` et `frontend/`.

### Ce qui existe et est réutilisable

| Composant | Emplacement | État |
|---|---|---|
| Couche S3 (client, crypto AES-256-GCM, reader, writer, key layout) | [`src/s3_cache_layer/`](../iviss-backend/src/s3_cache_layer/) | Fonctionnel, testé sur MinIO via testcontainers |
| Client API externe (`query_plate`, parsing HTML, TLS épinglé) | [`src/vehicle_client/`](../iviss-backend/src/vehicle_client/) | Fonctionnel |
| Repli cache sur panne API | [`handlers/search_vehicle.rs`](../iviss-backend/src/handlers/search_vehicle.rs) lignes 73-106 | Déjà câblé, avec timeout de 3 s |
| Mock de l'API externe | [`iviss-mock-ext-api/`](../iviss-mock-ext-api/) | Reproduit fidèlement le registre réel |
| Validation de format de plaque | [`utils/plate_format.rs`](../iviss-backend/src/utils/plate_format.rs) | Complet, 15+ gabarits regex |
| Page backoffice pending validation | [`PendingVehicles.tsx`](../frontend/src/pages/backoffice/PendingVehicles.tsx) | Fonctionnel |

### Ce qui ne va pas

1. **`services/vehicle_data_cache.rs` ne compile pas.** Refactor à moitié appliqué et commité : imports manquants (`Cache`, `Context`, `RegionProviderChain`…), variable `client` jamais liée mais utilisée dans le `Ok(Self { … })`, `store_vehicle_data` présent dans l'`impl` mais absent du trait, champ `dedup_cache` référencé mais inexistant, signature à 2 arguments appelée avec 1 aux deux sites d'appel. **Bloque toute vérification.**

2. **Le service de sync actuel ne fait pas ce qui est demandé.** [`bin/s3-cache-sync.rs`](../iviss-backend/src/bin/s3-cache-sync.rs) balaie l'API par préfixe de plaque (`GET /batch`) toutes les 5 min. Aucune notion de file d'attente, de fenêtre, de sonde de santé, ni de `unregistered/`.

3. **Aucun write-through.** `search_vehicle` lit le cache mais ne l'écrit jamais. Le cache ne peut donc être alimenté que par le balayage batch.

4. **`PLATE_PREFIX_CODES` sert à deux usages incompatibles.** Allowlist de partitionnement sur 2 caractères (`plate.get(..2)`) dans `cache_partition_for_plate`, et liste d'énumération batch dans le binaire sync. Conséquence : les entrées 3 caractères `CMD` et `CPC` ne matchent jamais, et toutes les plaques routées vers la partition `others` (militaires, État, temporaires, transit…) ne sont **jamais** récupérées par le sync — donc jamais servables depuis le cache pendant une panne.

5. **Noms de variables d'environnement divergents.** Le binaire sync lit `EXTERNAL_API_HEADER_*` alors que tout le reste du projet utilise `EXTERNAL_API_*`. `docker-compose.yml` masque le bug en définissant les deux orthographes à `""` pour le mock. Pointé sur le registre réel, le service enverrait des en-têtes d'authentification vides.

6. **`S3_CACHE_KMS_KEY_ID` vs `S3_CACHE_SSE_KMS_KEY_ID`.** `config.rs` lit le premier, l'ancre compose du backend exporte le second : **le backend ne reçoit jamais de clé KMS**. Le service sync fait le mapping correctement.

7. **Quatre valeurs par défaut différentes** pour `S3_CACHE_BUCKET` réparties sur cinq emplacements.

8. **Fichiers orphelins** : `src/feature_flags.rs` (plus déclaré dans `lib.rs`), `src/services/vehicle_client_service.rs` (tombstone d'une ligne).

---

## 3. Décisions et raisonnement

### 3.1 Sonde de santé — la question la plus longue à trancher

Le point difficile : comment le service sync sait-il que l'API externe est « up » ?

**Options écartées, et pourquoi :**

- **ICMP (`ping` réseau)** — nécessite `CAP_NET_RAW` dans le conteneur, et ICMP est très souvent filtré devant un endpoint HTTPS sur `:8443`. Un hôte qui ne répond pas à ICMP mais sert correctement du TLS serait vu comme définitivement en panne.
- **Connexion TCP sur `host:8443`** — un socket ouvert ne dit rien de l'application derrière.
- **`GET` HTTP sur `base_url`** — d'abord retenu comme bon compromis (n'importe quel statut HTTP = serveur vivant, TLS négocié, certificat épinglé validé ; et simuler une panne en dev revient à arrêter le mock). **Écarté après une objection décisive** : le registre externe interroge lui-même une base de données avant de répondre. Le serveur HTTP peut être parfaitement debout alors que sa base est inaccessible — les requêtes n'aboutiront pas et un `GET` sur la racine ne verrait rien.

**Décision : sonde = `POST /query` avec plaque sentinelle.** C'est le seul test qui traverse le chemin applicatif complet, base de données comprise.

Vérification faite sur [`iviss-mock-ext-api/src/routes/query.rs`](../iviss-mock-ext-api/src/routes/query.rs) : le mock reproduit **déjà exactement** ce comportement, sans code de simulation à ajouter.

| Situation | Réponse API | Côté client Rust |
|---|---|---|
| DB OK, plaque présente | 200 `{"data":"<html>IMMAT:…"}` | `Ok(_)` |
| DB OK, plaque absente | 200 `{"data":"…Service indisponible…"}` | `Err(NotFound)` |
| **DB en erreur** | **500** | `Err(Other)` |

Règle de la sonde : `Ok(_)` → up · `Err(NotFound)` → up · `Err(Other)` → down.

C'est **la même partition** que celle déjà utilisée par `search_vehicle` pour décider du repli cache : sonde et chemin agent restent cohérents par construction.

### 3.2 L'ambiguïté « Service indisponible »

Payload réel renvoyé par le registre quand la donnée n'existe pas mais que le service est up :

```json
{
  "data": "\n---------\nService distant:\n---------\nCE 546 LR\n--> Service indisponible\n06-08-2026 11:58:46"
}
```

**Analyse : ce payload ne permet pas de lever l'ambiguïté.** Le gabarit se lit littéralement « *Service distant* … → *Service indisponible* » — c'est un rapport sur le service distant que le registre interroge lui-même, pas un message « cette plaque n'est pas immatriculée ». Seuls la plaque et l'horodatage varient : aucun code d'erreur, aucun libellé distinct entre « plaque absente de la base » et « l'amont du registre ne répond pas ».

Ce qu'on en tire de fiable : la plaque est **renvoyée en écho** et **horodatée**, donc le registre a bien reçu, parsé et traité la requête. Suffisant comme signal de vie ; insuffisant pour conclure que le véhicule n'existe pas.

**Conséquence connue et acceptée** : `search_vehicle` traite `NotFound` comme une certitude et renvoie un 404 sans consulter le cache. Une vraie panne amont est donc présentée à l'agent comme « véhicule inexistant », et l'agent part filmer une carte grise pour un véhicule peut-être déjà en cache.

**Décision : ne rien changer à `search_vehicle` sur `NotFound`.** Conforme à l'architecture voulue (API up + donnée absente → flux carte grise). Le risque est identifié et documenté ; à rouvrir si l'ambiguïté se confirme en production.

### 3.3 Choix de la plaque sentinelle

Deux propositions successives :

1. *Utiliser la première clé de `retry-queue/` comme sonde* — élégant (pas de constante à maintenir, appel non gaspillé puisque son résultat est classé immédiatement), mais vulnérable au **poison pill** : une plaque déclenchant une erreur propre bloquerait la détection du « up » pendant toute la fenêtre d'1 h.

2. **Retenu — une constante en dur.** Même si l'enregistrement venait à être supprimé du registre, la requête renverra au moins un 200 si le service est up ; sinon le service renvoie autre chose, généralement après un timeout (comportement observé en production). Le problème du poison pill disparaît entièrement, la sonde étant découplée de la file.

**Correction de valeur** : `CD128AB`, proposé initialement, **n'est pas valide** selon [`plate_format.rs`](../iviss-backend/src/utils/plate_format.rs). `DIPLOMATIC_RE` exige `CD` + 2-3 chiffres + `RC` + chiffres, ou `CD` + chiffres uniquement ; `CD128AB` ne matche ni l'un ni l'autre, ni aucun autre gabarit. Retenu : **`CE128BC`** (format CivilCemac, celui des tests existants), verrouillé par un `assert!(plate_format::is_valid(HEALTH_PROBE_PLATE))` en test unitaire.

### 3.4 Autres décisions

| Sujet | Décision | Raison |
|---|---|---|
| **Write-through** | `tokio::spawn` détaché, échec loggé uniquement | La réponse agent part immédiatement, jamais bloquée ni dégradée par S3. Conforme au « async, non bloquant » du diagramme. |
| **Accès `unregistered/`** | Fusionné dans `GET /api/v1/admin/submissions` | Écarté : lecture S3 directe depuis le frontend — imposerait des credentials AWS dans le bundle navigateur, donnant accès à l'intégralité du cache véhicule. Une page, un appel, aucune credential côté client. |
| **`fetch_batch` / `GET /batch`** | Supprimés | La `retry-queue` devient la seule source de travail du sync. Résout au passage le double usage bogué de `PLATE_PREFIX_CODES`. |
| **Marqueurs de file** | JSON en clair, non chiffrés | Le corps ne contient que plaque + horodatage, et la plaque figure déjà en clair dans le nom de la clé (convention existante de `vehicle-cache/`). Chiffrer n'ajouterait rien tout en imposant un déchiffrement par objet au listing backoffice. Seul `vehicle-cache/` reste chiffré : lui porte des données personnelles. |
| **Préfixes plats** | `retry-queue/{PLATE}.json`, `unregistered/{PLATE}.json` | Énumérés en entier à chaque drain : un `ListObjectsV2` paginé suffit, là où un partitionnement imposerait 20 listes. `vehicle-cache/` conserve son partitionnement régional déjà testé. |
| **Ordre de suppression** | Marqueur supprimé **après** confirmation de l'écriture destination | Un crash entre les deux rejoue la plaque au cycle suivant. At-least-once, idempotent puisque la clé est déterministe. |
| **Garde-fou drain** | Abandon après `MAX_CONSECUTIVE_FAILURES` échecs consécutifs | Couvre le cas « serveur joignable mais `/query` cassé » que la sonde seule ne détecte pas. Les plaques restent en file, rien n'est perdu. |

---

## 4. Points signalés, non traités

Trois problèmes identifiés pendant l'exploration, hors périmètre de ce ticket et **délibérément non corrigés** :

1. **Isolation tenant — sévérité haute.** [`get_pending_submissions`](../iviss-backend/src/queries/submission_queries.rs) ne filtre par **aucune organisation** : les deux variantes SQL sélectionnent toutes les lignes de `pending_submissions`. Tout utilisateur autorisé sur cet endpoint voit les soumissions de tous les tenants. Antérieur à ce ticket, mais l'étape 4 du plan ajoute des données à cette même réponse — à traiter en priorité dans un ticket dédié.

2. **Secrets dans `.env`.** Identifiants de production réels présents (mot de passe de l'API externe, URL du registre, certificat TLS, clé de chiffrement du cache). À confirmer : `.env` bien ignoré par git, et aucune de ces valeurs jamais commitée. Un hook gitleaks existe dans `.githooks/`.

3. **Divergence doc/code.** [`IVISS_Sync_Architecture.md`](IVISS_Sync_Architecture.md) décrit le service de sync comme un proxy HTTP à la demande (`GET /fetch?plate=…`) avec un préfixe plat `vehicles/`, l'application n'ayant aucune credential S3. L'implémentation est l'inverse : service sans surface HTTP, préfixe partitionné `vehicle-cache/{REGION}/`, et l'API lit S3 directement. Le plan s'aligne sur le code ; le document devra être mis à jour.

---

## 5. Reprise du travail

Le plan est validé sur les décisions d'architecture mais **n'a pas été approuvé pour exécution** — aucun code n'a été modifié dans cette session.

Ordre d'exécution : étape 0 (réparer le fichier cassé) est bloquante pour tout le reste, car ni `cargo check` ni les tests ne passent tant qu'elle n'est pas close. Les étapes 1 à 5 suivent l'ordre du plan.

> **Note** : ces deux documents versionnés portent le raisonnement et le plan, pas la session Claude Code elle-même. Pour reprendre la conversation avec son contexte complet, il faut copier le transcript local (`~/.claude/projects/<projet>/<session-id>.jsonl` et son dossier `subagents/`) puis lancer `claude --resume <session-id>`. Le nom du dossier projet est le chemin absolu du dépôt avec les `/` remplacés par des `-`, et doit donc être renommé pour correspondre au chemin de la machine cible.
