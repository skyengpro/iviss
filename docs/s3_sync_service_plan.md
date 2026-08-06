# Plan d'implémentation — Service de synchronisation du cache S3

> **Statut** : plan validé sur les décisions d'architecture, implémentation non commencée.
> **Branche** : `perf/crop-dimensions-for-square-plate-format`
> **Date** : 2026-08-06

## Contexte

Aujourd'hui, une recherche de plaque par un agent ([`search_vehicle`](../iviss-backend/src/handlers/search_vehicle.rs)) interroge l'API intermédiaire externe et **ne conserve rien**. Le cache S3 n'est alimenté que par un binaire séparé ([`s3-cache-sync.rs`](../iviss-backend/src/bin/s3-cache-sync.rs)) qui balaie l'API par préfixe de plaque toutes les 5 min via `GET /batch`. Deux conséquences : les recherches réellement effectuées sur le terrain n'enrichissent jamais le cache, et une recherche qui échoue pendant une panne de l'API externe est définitivement perdue.

L'objectif est qu'**aucune recherche agent n'échappe au stockage S3**, afin d'assurer la continuité de service quand l'API intermédiaire est indisponible. On remplace le balayage par préfixe par une architecture pilotée par la demande réelle : write-through sur succès, file d'attente `retry-queue/` sur échec, et un service de synchronisation qui draine cette file pendant des fenêtres planifiées.

Un blocage préalable : [`services/vehicle_data_cache.rs`](../iviss-backend/src/services/vehicle_data_cache.rs) est commité **dans un état qui ne compile pas** (refactor à moitié appliqué : imports manquants, variable `client` jamais liée, `store_vehicle_data` hors du trait, champ `dedup_cache` inexistant, signature à 2 arguments appelée avec 1). Rien ne peut être vérifié tant que ce n'est pas réparé.

## Architecture cible

```mermaid
---
config:
  layout: fixed
---
flowchart TB
 subgraph S3["Bucket S3 — préfixes isolés"]
        Cache[("vehicle-cache/")]
        Queue[("retry-queue/")]
        Unreg[("unregistered/")]
  end
 subgraph B["Backend B — service de synchronisation séparé"]
        Drain["Drain planifié<br>fenêtre 1h / cycle 3h"]
        Ping["Sonde santé<br>toutes les 5 min"]
  end
    Agent(["Agent terrain"]) --> A["Backend A — API"]
    A <-- POST /query --> Ext[["API intermédiaire externe"]]
    A -- succès --> WT["Write-through<br>async, non bloquant"] & RespOK(["Réponse agent"])
    A -- échec transitoire --> RD{"Lecture cache S3"}
    RD -- hit --> RespCache(["Réponse dégradée<br>cached_at exposé"])
    RD -- miss --> RespErr(["Erreur agent"]) & PutQ["Écrit marqueur retry"]
    WT --> Cache
    RD -. get .-> Cache
    PutQ --> Queue
    Ping -- API up --> Drain
    Queue -- list + get --> Drain
    Drain -- query_plate --> Ext
    Drain -- trouvé --> Cache
    Drain -- NotFound --> Unreg
    Drain -- marqueur résolu/classé --> Queue
    Office(["Backoffice"]) -- liste --> Unreg
```

## Décisions actées

| Sujet | Décision |
|---|---|
| Sonde de santé | `POST /query` avec plaque sentinelle constante. `Ok` **ou** `NotFound` = up ; `Other` (transport, TLS, 5xx, timeout) = down |
| Plaque sentinelle | Constante `CE128BC` — `CD128AB` ne passe pas `plate_format::is_valid` |
| `search_vehicle` sur `NotFound` | **Inchangé** : 404 sec, cache non consulté → flux carte grise |
| Write-through | `tokio::spawn` détaché, échec loggé uniquement, jamais bloquant pour l'agent |
| `unregistered/` au backoffice | Fusionné dans `GET /api/v1/admin/submissions` existant |
| `fetch_batch` / `GET /batch` | Supprimé |

Justification de la sonde : le registre renvoie `{"data":"…Service distant… --> Service indisponible… <date>"}` avec HTTP 200 quand la donnée est absente. Le gabarit est identique qu'il s'agisse d'une plaque non immatriculée ou d'une panne amont — il ne permet donc pas de distinguer les deux cas, mais l'écho de la plaque et l'horodatage prouvent que le service applicatif a traité la requête. C'est suffisant comme signal de vie, et c'est la même partition `Ok`/`NotFound` vs `Other` que celle déjà utilisée par `search_vehicle` pour décider du repli cache : sonde et chemin agent restent cohérents par construction.

---

## Étape 0 — Réparer `services/vehicle_data_cache.rs`

Fichier : [`iviss-backend/src/services/vehicle_data_cache.rs`](../iviss-backend/src/services/vehicle_data_cache.rs)

Le réécrire en délégation pure vers `s3_cache_layer` (pas de duplication de la logique AWS/crypto, qui existe déjà dans [`s3_cache_layer/config.rs`](../iviss-backend/src/s3_cache_layer/config.rs), [`s3_writer.rs`](../iviss-backend/src/s3_cache_layer/s3_writer.rs), [`s3_reader.rs`](../iviss-backend/src/s3_cache_layer/s3_reader.rs)) :

```rust
#[async_trait]
pub trait VehicleDataCache: Send + Sync {
    async fn get_vehicle_data(&self, plate: &str) -> Result<Option<CachedVehicleData>>;
    async fn store_vehicle_data(&self, plate: &str, vehicle: &VehicleInfo) -> Result<()>;
    async fn enqueue_retry(&self, plate: &str) -> Result<()>;
    async fn list_unregistered(&self) -> Result<Vec<UnregisteredPlate>>;
}
```

- `from_config(&S3CacheConfig)` → **un seul argument**, délègue à `s3_cache_layer::build_s3_client`. Supprime toute trace de `dedup_cache` (le champ `vehicle_dedup` a déjà été retiré de `AppCache` sur cette branche).
- Les 4 méthodes délèguent aux fonctions libres de `s3_cache_layer`.
- Corrige les appels existants : `main.rs:56` et `tests/vehicle_data_cache_tests.rs:157` passent déjà un seul argument — ils redeviennent corrects sans modification.

---

## Étape 1 — Étendre `s3_cache_layer` aux nouveaux préfixes

Fichier : [`iviss-backend/src/s3_cache_layer/types.rs`](../iviss-backend/src/s3_cache_layer/types.rs)

```rust
pub const S3_CACHE_PREFIX: &str = "vehicle-cache/";   // existant, inchangé
pub const RETRY_QUEUE_PREFIX: &str = "retry-queue/";
pub const UNREGISTERED_PREFIX: &str = "unregistered/";
```

Disposition des clés : `vehicle-cache/` **conserve** son partitionnement régional existant (`vehicle-cache/{PARTITION}/{PLATE}.json`, déjà testé). `retry-queue/` et `unregistered/` sont **plats** — `retry-queue/{PLATE}.json` — parce qu'ils sont énumérés en entier à chaque drain : un `ListObjectsV2` paginé sur un préfixe plat suffit, alors qu'un partitionnement imposerait 20 listes.

Ajouter, sur le modèle de `object_key` (qui rejette déjà tout caractère non `ascii_alphanumeric` — protection contre l'injection de clé, à conserver et réutiliser telle quelle) :

- `retry_queue_key(plate) -> Result<String>` et `unregistered_key(plate) -> Result<String>`, factorisées avec `object_key` via un helper commun de validation.
- `plate_from_key(key, prefix) -> Option<String>` pour le chemin retour du listing.
- Struct `QueueMarker { plate_number: String, queued_at: String }` (RFC3339), sérialisée en JSON.

Nouveau fichier : `iviss-backend/src/s3_cache_layer/s3_queue.rs`

```rust
pub async fn enqueue_plate(client, bucket, plate) -> Result<()>
pub async fn list_queued_plates(client, bucket, prefix, max: usize) -> Result<Vec<String>>  // ListObjectsV2 paginé
pub async fn remove_marker(client, bucket, prefix, plate) -> Result<()>
pub async fn mark_unregistered(client, bucket, plate) -> Result<()>
```

Les marqueurs sont écrits **en clair** (JSON, pas de chiffrement AES) : le corps ne contient que la plaque et un horodatage, et la plaque figure déjà en clair dans le nom de la clé — comme c'est déjà le cas pour `vehicle-cache/{PARTITION}/{PLATE}.json`. Chiffrer le corps n'ajouterait aucune protection réelle tout en imposant un déchiffrement par objet au listing du backoffice. Seul `vehicle-cache/` reste chiffré, car lui porte des données personnelles (nom, adresse, pièce d'identité du propriétaire).

**Attention IAM** : `remove_marker` introduit un besoin de `s3:DeleteObject`, que [`IVISS_Sync_Architecture.md`](IVISS_Sync_Architecture.md) exclut explicitement de la politique de moindre privilège. À restreindre par condition sur `retry-queue/*` uniquement, jamais sur `vehicle-cache/*`.

---

## Étape 2 — Backend A : write-through et mise en file

Fichier : [`iviss-backend/src/handlers/search_vehicle.rs`](../iviss-backend/src/handlers/search_vehicle.rs)

**Branche `Ok(api_response)`** (ligne 56) — après `build_search_result`, avant de répondre :

```rust
if let Some(cache) = &state.s3_data_cache {
    let (cache, plate, vehicle) = (cache.clone(), plate.clone(), response.vehicle.clone());
    tokio::spawn(async move {
        if let Err(e) = cache.store_vehicle_data(&plate, &vehicle).await {
            tracing::warn!(error = %e, "write-through S3 échoué");
        }
    });
}
```

Détaché : la réponse agent part immédiatement et n'est jamais dégradée par S3. Ne pas logger la plaque avec les données propriétaire.

**Branche `Err(other)`** (ligne 70) — la lecture cache existante avec `S3_CACHE_READ_TIMEOUT` (3 s) est conservée telle quelle. Sur cache **miss**, erreur de lecture ou timeout, avant le `Err(AppError::external_api_failure)` final : `tokio::spawn` d'un `cache.enqueue_retry(&plate)`.

**Branche `Err(NotFound)`** (ligne 63) : inchangée.

Sur un hit cache, la réponse expose aujourd'hui `confidence: Some(1.0)` et `IdentificationMode::Manual` sans aucun indicateur de fraîcheur — l'agent ne peut pas distinguer une donnée live d'une donnée de cache. Le diagramme prévoit `cached_at exposé`. Ajouter à `VehicleSearchResult` : `source: Option<VehicleDataSource>` (`Live` | `Cache`) et `cached_at: Option<String>`, tous deux `Option` pour rester rétrocompatibles.

> **Changement de contrat OpenAPI** — impose la régénération du client frontend (`npm run codegen`, voir étape 4).

---

## Étape 3 — Backend B : réécriture du service de synchronisation

Fichier : [`iviss-backend/src/bin/s3-cache-sync.rs`](../iviss-backend/src/bin/s3-cache-sync.rs) — remplacer entièrement la boucle par préfixe.

Constantes nommées, chacune surchargeable par variable d'environnement (indispensable : personne ne peut tester un cycle de 3 h à la main) :

```rust
const HEALTH_PROBE_PLATE: &str = "CE128BC";
const DRAIN_WINDOW: Duration         = Duration::from_secs(60 * 60);      // SYNC_WINDOW_SECS
const IDLE_BETWEEN_WINDOWS: Duration = Duration::from_secs(2 * 60 * 60);  // SYNC_IDLE_SECS
const PING_INTERVAL: Duration        = Duration::from_secs(5 * 60);       // SYNC_PING_INTERVAL_SECS
const MAX_CONSECUTIVE_FAILURES: u32  = 5;                                 // SYNC_MAX_CONSECUTIVE_FAILURES
```

Boucle principale — cycle de 3 h :

1. **Fenêtre de drain**, `DRAIN_WINDOW` (1 h) : toutes les `PING_INTERVAL` (5 min),
   - `list_queued_plates(retry-queue/, max=1)` → si **vide**, ne rien faire, attendre le ping suivant. Aucune sonde n'est émise quand il n'y a rien à drainer.
   - Sinon, sonde : `query_plate(HEALTH_PROBE_PLATE)`.
     - `Err(Other)` → API down, on log et on attend le ping suivant.
     - `Ok(_) | Err(NotFound)` → API up, on draine.
2. **Drain** : `list_queued_plates` paginé, puis pour chaque plaque :
   - `Ok(resp)` → `write_vehicle_data` dans `vehicle-cache/`, puis `remove_marker`.
   - `Err(NotFound)` → `mark_unregistered` dans `unregistered/`, puis `remove_marker`.
   - `Err(Other)` → **marqueur laissé en place**, incrémente le compteur d'échecs consécutifs ; à `MAX_CONSECUTIVE_FAILURES` on avorte le drain du cycle et on repasse en mode ping. Garde-fou contre le cas « serveur joignable mais `/query` cassé ».
   - Le compteur est remis à zéro sur tout succès.
   - Le marqueur n'est supprimé **qu'après** confirmation de l'écriture destination : un crash entre les deux rejoue la plaque au cycle suivant (at-least-once, idempotent puisque la clé est déterministe).
3. **Repos** : `IDLE_BETWEEN_WINDOWS` (2 h).

Suppressions associées :

- `fetch_batch` dans [`vehicle_client/client.rs`](../iviss-backend/src/vehicle_client/client.rs) (ligne 100) et le type `ExternalVehicle` ([`vehicle_client/types.rs`](../iviss-backend/src/vehicle_client/types.rs), + son export dans `mod.rs`).
- Route `GET /batch` du mock : `iviss-mock-ext-api/src/routes/batch.rs`, sa déclaration dans `routes/mod.rs` et `main.rs`, ainsi que `db::find_by_prefix`.
- `PLATE_PREFIX_CODES` cesse d'être une liste d'énumération et redevient uniquement l'allowlist de partitionnement de `cache_partition_for_plate`. Cela résout au passage le double usage bogué : les entrées 3 caractères `CMD` et `CPC` ne pouvaient jamais matcher `plate.get(..2)`.
- Fichiers orphelins : `src/feature_flags.rs` (plus déclaré dans `lib.rs`), `src/services/vehicle_client_service.rs` (tombstone d'une ligne).

Le binaire reste construit avec `--no-default-features` (pas de `sqlx`, pas d'axum) : le drain n'a besoin que de S3 et du client HTTP.

**Test unitaire** : `assert!(plate_format::is_valid(HEALTH_PROBE_PLATE))` — verrouille la constante contre une future dérive de format.

---

## Étape 4 — Backoffice : fusion des plaques `unregistered/`

**Backend** — [`handlers/pending_submission.rs`](../iviss-backend/src/handlers/pending_submission.rs), `list_pending_submissions` : après `get_pending_submissions`, si `state.s3_data_cache` est présent, appeler `list_unregistered()` et concaténer les entrées converties en `PendingSubmissionListItem`, triées par date décroissante sur l'ensemble.

Adapter [`dto/pending_submission.rs`](../iviss-backend/src/dto/pending_submission.rs) :

```rust
pub struct PendingSubmissionListItem {
    pub id: Option<Uuid>,         // None pour une entrée S3 — pas de ligne en base
    pub plate_number: String,
    pub agent_name: Option<String>,
    pub status: SubmissionStatus,
    pub submitted_at: String,
    pub source: SubmissionSource, // Submission | S3Unregistered
}
```

`source` permet au frontend de savoir qu'il n'y a ni images ni détail à charger. Un échec de lecture S3 est loggé et n'invalide pas la réponse : la liste des soumissions en base reste servie (dégradation gracieuse).

> **Changement de contrat OpenAPI** — `id` devient nullable et `source` est ajouté.

**Frontend** — [`PendingVehicles.tsx`](../frontend/src/pages/backoffice/PendingVehicles.tsx) : masquer les vignettes carte grise et désactiver l'ouverture du détail quand `source === 's3_unregistered'`. Ajouter les clés i18n correspondantes dans `frontend/src/i18n/locales/{en,fr}.json`.

Cette page est le seul consommateur d'API du frontend qui **n'utilise pas** le client généré : elle passe par [`services/api/submissionService.ts`](../frontend/src/services/api/submissionService.ts) avec des interfaces TS maintenues à la main (tous les autres hooks importent depuis `src/openapi-rq/queries/queries`). Il faut donc mettre à jour ces types manuellement **en plus** de régénérer le client. Aligner cette page sur le client généré serait cohérent avec le reste du code, mais c'est un refactor à part — hors périmètre ici.

---

## Étape 5 — Configuration et déploiement

- [`config.rs`](../iviss-backend/src/config.rs) : `get_s3_cache_config` lit `S3_CACHE_KMS_KEY_ID`, mais `docker-compose.yml` (ligne 54) exporte `S3_CACHE_SSE_KMS_KEY_ID` pour le backend — **le backend ne reçoit jamais de clé KMS**. Le service `s3-cache-sync` fait le mapping correctement (ligne 186). Corriger l'ancre partagée.
- Le binaire sync duplique le chargement d'env (`load_vehicle_api_credentials`, `load_s3_cache_config`, lignes 139-199) et lit `EXTERNAL_API_HEADER_*` là où tout le reste du projet utilise `EXTERNAL_API_*`. Compose masque le bug en définissant les deux orthographes à `""`. **Pointé sur le registre réel, le service enverrait des en-têtes vides.** Aligner sur les noms `EXTERNAL_API_*` et retirer les doublons de compose.
- `S3_CACHE_BUCKET` a **quatre valeurs par défaut différentes** selon l'endroit (`vehicle-data-cache`, `iviss-vehicle-cache`…). Unifier sur une seule valeur dans `.env.example` et `docker-compose.yml` (services `minio-init`, `s3-cache-sync`, `backend`).
- Remplacer `SYNC_INTERVAL_SECS` par les nouvelles variables de fenêtre dans le bloc `s3-cache-sync` de `docker-compose.yml`.

---

## Vérification

1. `cargo check` (défaut) puis `cargo check --no-default-features --bin s3-cache-sync` — les deux profils de compilation doivent passer. C'est la garantie que l'étape 0 est réellement close.
2. `cargo fmt` et `cargo clippy` sur les deux profils.
3. `cargo test s3_cache_layer` et `cargo test vehicle_data_cache` — les tests MinIO via testcontainers existants ([`tests/vehicle_data_cache_tests.rs`](../iviss-backend/src/tests/vehicle_data_cache_tests.rs)) doivent rester verts ; y ajouter la couverture `enqueue_retry` → `list_queued_plates` → `remove_marker` et le round-trip `mark_unregistered` → `list_unregistered`.
4. Bout en bout en local :
   - `cargo run` dans `iviss-mock-ext-api/`, `docker compose --profile dev up minio minio-init s3-cache-sync backend`.
   - **Write-through** : rechercher une plaque de `seeds/vehicles.sql` → `mc ls local/<bucket>/vehicle-cache/` doit montrer l'objet.
   - **Panne + hit cache** : arrêter le mock, rechercher la même plaque → réponse 200 avec `source: "cache"` et `cached_at`.
   - **Panne + miss** : arrêter le mock, rechercher une plaque jamais vue → erreur agent, et `mc ls local/<bucket>/retry-queue/` montre le marqueur.
   - **Drain** : redémarrer le mock, `SYNC_WINDOW_SECS=120 SYNC_PING_INTERVAL_SECS=10 SYNC_IDLE_SECS=30` → le marqueur disparaît, l'objet apparaît dans `vehicle-cache/` (plaque connue) ou `unregistered/` (plaque inconnue).
   - **Sonde down** : arrêter Postgres sous le mock → `/query` renvoie 500 → le sync doit rester en ping et **ne pas** vider la file.
   - **Backoffice** : la page Pending Validation liste l'entrée `unregistered/` sans image ni détail cliquable.
5. `npm run codegen` dans `frontend/` après démarrage du backend, puis `npm run test` (vitest).

## Points signalés, hors périmètre

- **Isolation tenant** : `get_pending_submissions` ([`queries/submission_queries.rs`](../iviss-backend/src/queries/submission_queries.rs)) ne filtre **par aucune organisation** — les deux variantes SQL sélectionnent toutes les lignes de `pending_submissions`. Tout utilisateur autorisé sur cet endpoint voit les soumissions de tous les tenants. C'est antérieur à ce ticket et n'est pas corrigé ici, mais l'étape 4 ajoute des données à cette même réponse : à traiter en priorité dans un ticket dédié. À noter que les entrées `unregistered/` sont par nature globales (le bucket n'est pas partitionné par organisation) — décider si elles doivent l'être.
- **Secrets dans `.env`** : le fichier contient des identifiants de production réels (mot de passe de l'API externe, URL du registre, certificat TLS, clé de chiffrement du cache). Confirmer que `.env` est bien ignoré par git et qu'aucune de ces valeurs n'a jamais été commitée.
- **Divergence doc/code** : [`IVISS_Sync_Architecture.md`](IVISS_Sync_Architecture.md) décrit le service de sync comme un proxy HTTP à la demande (`GET /fetch?plate=…`) avec un préfixe plat `vehicles/`, alors que le code implémente un service sans surface HTTP et un préfixe partitionné `vehicle-cache/{REGION}/`. Le présent plan s'aligne sur le code ; le document devra être mis à jour.
