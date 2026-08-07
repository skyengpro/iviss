# Compte-rendu de session — Évaluation d'architecture et recadrage du backend par domaine

> **Date** : 2026-08-07 · **Branche** : `298-implement-s3-cache-layer-provisioning-service`
> **Plan produit** : [`backend_domain_refactor_plan.md`](backend_domain_refactor_plan.md)
> **Session amont** : [`../s3_sync_service_session.md`](../s3_sync_service_session.md) (conception du service de sync S3)
> **Portée** : session d'analyse et de conception uniquement — **aucun code modifié**.

Ce document rejoue le raisonnement de la session : ce qui a été mesuré dans la codebase, les
propositions successives, et **pourquoi** chaque arbitrage a été tranché. Il existe pour qu'une
reprise du travail ne reparte pas de zéro sur des décisions déjà prises.

---

## 1. Demande initiale

Deux questions, dans cet ordre imposé :

1. **Évaluer et noter l'architecture actuelle telle quelle** (avantages et inconvénients), afin de
   décider s'il est nécessaire de refactorer ou non.
2. Si oui : rechercher l'état de l'art du **monolithe modulaire de production**, parcourir
   intégralement la structure du backend, et proposer une arborescence de refactoring adaptée à
   IVISS — en soulignant les contraintes, les avantages et les difficultés.

L'esquisse initiale envisageait des modules indépendants contenant chacun toute sa logique
(`dto`, `router`, `query`, `model`, `service`, `unit_tests`) : `auth [otp, jwt]`,
`users [admins, agents, ocr [photo, scan]]`, `organisation`, `stats`,
`control [search vehicle, list control]`, `audit`,
`external services [vehicle_client, insurance_client, technical_visit_client]`,
`notification [email, sms]`.

---

## 2. État des lieux mesuré

Exploration exhaustive de `iviss-backend/src/` — 106 fichiers Rust, 25 427 LOC.

### 2.1 Structure réelle

Organisation en **package-by-layer** : `handlers/ services/ queries/ dto/ middleware/ models/`,
plus des fichiers racine centralisateurs (`routes.rs`, `api_doc.rs`, `app_state.rs`, `config.rs`).

| Répartition | LOC |
|---|---|
| Production | 16 120 |
| Tests (`src/tests/`, 17 fichiers) | 9 307 (37 %) |
| **Total** | **25 427** |

Domaines identifiés depuis `routes.rs` : `auth`, `users`, `user_management`, `organizations`,
`controls`, `vehicles/search`, `pending_submissions`, `scan`+`photo` (OCR), `stats`, `audit`, `health`.

### 2.2 Ce qui est bon

- **Aucun cycle de dépendance** sur les 106 fichiers. `queries/` ne dépend que de `dto` + `errors` ;
  `services/` ne dépend jamais de `handlers/`. Les couches sont respectées vers le bas.
- **`vehicle_client/` et `s3_cache_layer/` sont déjà des tranches verticales propres** — `mod.rs`
  avec surface publique explicite et re-exports. L'équipe sait donc déjà produire ce modèle.
- **RBAC centralisé et serveur** : les guards sont posés **par groupe de routes** dans `routes.rs`
  (lignes 103-104 et 136-137). La matrice de sécurité se lit d'un seul coup d'œil. C'est une vraie
  force du god-router, pas un défaut.
- **Observabilité niveau production** : OTel + Prometheus + port métriques séparé (9091, hors ingress
  public) + graceful shutdown + flush télémétrie.
- **Découpage `feature = "api"` déjà pensé** : le binaire `s3-cache-sync` réutilise
  `dto`/`vehicle_client`/`s3_cache_layer` sans tirer axum/sqlx/leptess.
- **9 307 LOC de tests** sur testcontainers (Postgres + MinIO réels). C'est le filet de sécurité qui
  rend n'importe quel refactoring faisable.

### 2.3 Ce qui coûte — chiffres, pas impressions

| Mesure | Commande / preuve |
|---|---|
| **Churn 6 mois** : `routes.rs` 65 commits, `main.rs` 52, `api_doc.rs` 40, `app_state.rs` 30, `services/mod.rs` 23, `tests/mod.rs` 22 | `git log --since="6 months ago" --name-only --pretty=format: -- iviss-backend/src \| sort \| uniq -c \| sort -rn` |
| **5-6 contributeurs actifs** sur 1 256 commits | `git shortlog -sn --all` |
| **27 `sqlx::query` bruts hors `queries/`** : `handlers/auth.rs` ×18, `user_management.rs` ×5, `pending_submission.rs` ×2, `search_vehicle.rs` ×1, **`middleware/rbac.rs` ×1** | `grep -rn "sqlx::query" src/handlers/ src/middleware/` |
| **`handlers/auth.rs` = 1 457 lignes** (churn 84 commits) | `wc -l` |
| **0 macro `sqlx::query!` contre 208 requêtes runtime**, `.sqlx/` avec 1 seule entrée | `grep -rn "sqlx::query!" src \| wc -l` |
| **`organization_id` filtré dans 6 des 10 modules `queries/`** | `grep -rln organization_id src/queries/` |
| **Code mort** : `feature_flags.rs` (44 l.) jamais déclaré dans `lib.rs` ; `services/vehicle_client_service.rs` commenté dans `services/mod.rs:9` | `grep -rn feature_flags src` → vide |
| **7 structs DTO déclarées dans les handlers** : `stats.rs:36,70,103,133` et `auth.rs:940,946,954` | `grep -n "pub struct" src/handlers/*.rs` |
| **`AppState` = 11 champs** | lecture de `app_state.rs` |

### 2.4 Note attribuée

| Axe | Note | Justification |
|---|---|---|
| Organisation / cohésion | 5/10 | Cohésion technique forte, cohésion métier nulle |
| Couplage / sens des dépendances | 7/10 | Aucun cycle, couches respectées — mais 27 court-circuits |
| Testabilité | 5/10 | Volume excellent, mais presque tout exige Docker+Postgres ; rien d'unitaire côté métier |
| Sécurité multi-tenant | 6/10 | RBAC centralisé serveur ✅ ; isolation tenant par convention ⚠ |
| Contrat OpenAPI / frontend | 7/10 | Complet et codegen'd, mais god-file |
| Passage à l'échelle de l'équipe | 4/10 | Churn hotspots × 6 devs |
| Boucle de feedback / build | 5/10 | Mono-crate 25k LOC, tests in-crate, gymnastique de features |
| Observabilité / ops | 8/10 | Rien à redire |
| **GLOBAL** | **6/10** | |

**Verdict rendu** : architecture honnête et bien tenue, **pas cassée**, mais qui a franchi le point où
package-by-layer coûte plus qu'il ne rapporte. Refactoring **justifié mais non urgent**, à faire de
façon incrémentale et mécanique — jamais en réécriture.

**Point d'architecte souligné** : le vrai gain n'est pas l'arborescence, c'est (a) sortir la logique
de `handlers/auth.rs` et (b) transformer l'isolation tenant en type. Un déplacement purement
cosmétique de 106 fichiers apporterait ~30 % de la valeur pour ~70 % du risque.

---

## 3. Recherche — état de l'art du monolithe modulaire

| Source | Apport retenu |
|---|---|
| [Milan Jovanović — Modular Monolith Architecture](https://milanjovanovic.tech/modular-monolith-architecture) | Modules à frontières explicites, chacun exposant une **API publique** ; toute communication inter-module passe par elle |
| [Internal vs. Public APIs in Modular Monoliths](https://www.milanjovanovic.tech/blog/internal-vs-public-apis-in-modular-monoliths) | Rendre le couplage **explicite et contrôlé** plutôt que de l'interdire |
| [Where Vertical Slices Fit Inside the Modular Monolith](https://milanjovanovic.tech/blog/where-vertical-slices-fit-inside-the-modular-monolith-architecture) | Modular monolith pour les frontières, vertical slice à l'intérieur — les deux se combinent |
| [Rust at scale: packages, crates, and modules](https://mmapped.blog/posts/03-rust-packages-crates-modules) | ⚠ Dans un crate unique, **aucune contrainte compile-time** n'empêche un module d'en importer un autre. Le workspace multi-crates donne cette garantie + le parallélisme de compilation, au prix d'un `Cargo.toml` par crate |
| [Rust Project Primer — Organization](https://rustprojectprimer.com/organization/index.html) | Ajouter un crate est nettement plus coûteux qu'ajouter un module |
| [Building Modular Web APIs with Axum](https://leapcell.io/blog/building-modular-web-apis-with-axum-in-rust) · [axum discussion #2152](https://github.com/tokio-rs/axum/discussions/2152) | Séparer le routage par module évite que `main.rs`/`routes.rs` devienne monolithique et réduit les conflits de merge |

**Enseignement transposé** : la règle « architecture rules checks at build time » (type ArchUnit) n'a
pas d'équivalent natif en Rust dans un crate unique → il faut un lint CI si l'on veut des frontières
tenues.

---

## 4. Première proposition (monolithe modulaire complet) — écartée

Structure en 3 zones + racine de composition : `core/` (noyau partagé) ← `platform/` (infra) ←
`modules/` (tranches métier) ← `app/` (composition), avec 11 étapes de migration en strangler et une
option de bascule en workspace 4 crates.

**Écartée après contre-proposition** : coût estimé 6-8 semaines en incrémental, risque élevé sur
l'éclatement de `api_doc.rs` (contrat frontend), la recomposition RBAC et le module `auth`.

---

## 5. Contre-proposition retenue — package-by-layer-then-feature

Proposition formulée en séance : garder les 4 couches horizontales et **sous-diviser chacune par
domaine**, avec en plus un répertoire `external_services/` pour les clients partenaires et le
rapatriement de toutes les requêtes SQL dispersées dans `queries/`.

### 5.1 Pourquoi c'est le bon appel

**Ce n'est pas une impasse : c'est un préfixe strict du monolithe modulaire.** Si les répertoires de
domaine portent le même nom sur les 4 couches, la bascule ultérieure est un pur `git mv` :

```bash
git mv src/handlers/auth  src/modules/auth/api
git mv src/services/auth  src/modules/auth/service
git mv src/queries/auth   src/modules/auth/infra
```

D'où la **règle n°1 non négociable : nomenclature identique sur les 4 couches.**

### 5.2 Coût comparé

| | Proposition retenue | Monolithe modulaire complet |
|---|---|---|
| Signatures modifiées | **0** | ~35 handlers |
| Diff OpenAPI attendu | **0** | refonte de `api_doc.rs` |
| Composition RBAC | préservée | recomposition (risque n°1) |
| **Effort** | **~11 j-dev ≈ 2 à 2,5 sem.** | **6-8 semaines** |
| **Risque** | 🟢 faible | 🔴 élevé |
| **Valeur captée** | **~65 %** | 100 % |

### 5.3 Le levier de réduction de risque — l'astuce des re-exports

`handlers/auth.rs` → `handlers/auth/mod.rs` avec `pub use login::login;` etc. →
`crate::handlers::auth::login` **reste valide**. Conséquence : `api_doc.rs` (churn 40, critique pour
le contrat frontend) et `routes.rs` **ne changent pas d'une ligne** pendant le découpage des
handlers. Le diff OpenAPI attendu est **zéro**.

C'est ce qui transforme une opération potentiellement risquée en opération mécanique vérifiable.

---

## 6. Arbitrages tranchés

### 6.1 Corrections apportées à l'esquisse initiale

| Esquisse | Retenu | Pourquoi |
|---|---|---|
| `email` et `sms` sous `services/auth/` | `services/notifications/` + `services/auth/{jwt, otp}` | Ces transports servent **deux** domaines : `auth` (OTP) et `users` (`resend_activation_code` à `user_management.rs:339`, `resend_org_admin_password` à `:476`). Sous `auth/`, `handlers/users/` devrait importer `services/auth/email` — couplage inversé |
| `queries/users_query/`, `control_query/` | `queries/users/`, `queries/controls.rs` | Le suffixe est redondant dans `queries/` et **casse l'alignement 1:1 entre couches**, condition de la réversibilité (§5.1) |
| Un répertoire par domaine dans `queries/` | **Répertoire seulement si > 400 LOC ou > 1 fichier** | `queries/x.rs` et `queries/x/mod.rs` sont équivalents pour l'appelant : la promotion est non cassante, réversible et invisible. Seuls `auth/` (~600 l.) et `users/` (~700 l.) la justifient aujourd'hui. 8 répertoires à un fichier = cérémonie pure |
| `ocr` sous `users` | `handlers/ocr/` autonome | L'OCR n'a aucune table, aucun tenant, aucune dépendance à `users`. Le placer sous `users` créerait un couplage fictif |
| `admins`/`agents` comme sous-modules de `users` | un seul domaine `users` | Ce sont des **rôles** (`UserRole`), pas des agrégats. Même table, mêmes requêtes, même cycle de vie |
| `s3_cache_layer/` déplacé dans `external_services/` | **laissé où il est** | Ce n'est pas un partenaire tiers mais l'infrastructure de stockage interne ; il est déjà propre ; et il est **sur le chemin critique de la branche en cours** (le plan S3 y ajoute `s3_queue.rs`). Principe du diff minimal |

### 6.2 Décisions prises en séance

| Question | Décision | Raison |
|---|---|---|
| **Éclater `routes.rs` ?** (hors périmètre initial, +2 j) | ✅ **Oui** | Fichier n°2 en churn (65 commits/6 mois × 6 devs). 222 → ~45 lignes. Naturellement débloqué par `handlers/<domaine>/router.rs` |
| **Éclater `api_doc.rs` ?** | ❌ **Non, ticket séparé** | Impose `OpenApi::merge()` par domaine (sémantique à vérifier sur `components.schemas` et le `SecurityAddon`) ; `utoipa-axum`/`OpenApiRouter` exigerait **utoipa 5**, dont la montée risque seule de faire dériver le contrat. Y toucher casserait la propriété « diff OpenAPI nul » |
| **Disposition des clés S3 avec 3 services ?** | **Inchangée** | Périmètre confirmé : véhicule seul en réel, assurance et visite technique **mockés comme aujourd'hui**. Le plan S3 reste valide sans amendement fonctionnel |
| **Branche ?** | `refactor/backend-domain-layout`, clonée depuis `298-…` et mergée **dans** `298-…` | `dev` est la version en ligne ; `298` contient déjà `dev`. Merge vers `dev` seulement après validation complète |

### 6.3 Le point de conception le plus structurant — le trait `ExternalDataSource`

**Problème posé** : le mode d'accès aux services assurance et visite technique n'est pas décidé (API
HTTP ou accès direct à leur base de données), et ces services connaîtront le même problème
d'indisponibilité que l'API véhicule actuelle — le service de sync devra donc pouvoir les contacter.

**Réponse** : un **port**. On fige le contrat maintenant, on décide du transport plus tard.

```rust
#[async_trait]
pub trait ExternalDataSource: Send + Sync {
    fn service_id(&self) -> &'static str;
    async fn fetch(&self, plate: &str) -> Result<PartnerPayload, ExternalServiceError>;
    async fn health_probe(&self) -> HealthStatus;
}
```

Le transport (HTTP ou `PgPool`) devient un détail privé de chaque implémentation. Le service de sync
itère `Vec<Arc<dyn ExternalDataSource>>` sans rien savoir des partenaires : ajouter un 4ᵉ service =
1 fichier + 1 ligne d'enregistrement. La sonde `HEALTH_PROBE_PLATE = "CE128BC"` du plan S3 devient un
détail interne à `vehicle_client`, exposé via `health_probe()`.

**Périmètre restreint décidé en séance** : seul `vehicle_client` implémente le trait.
`insurance_client/` et `technical_inspection_client/` existent comme répertoires mais ne contiennent
que **le mock actuel déplacé sans changement de comportement** — les `InsuranceStatus`/`TechnicalStatus`
en `Pending` aujourd'hui codés en dur dans `vehicle_status_service.rs:53-90` en sortent pour devenir
`pending_insurance_status()` et `pending_technical_status()`.

Ils restent **synchrones** et n'implémentent pas encore le trait : le faire imposerait de rendre
`build_status_results_from_api` async, ce qui remonterait jusqu'à `search_vehicle` — un ripple non
justifié pour du mock. Bénéfice quand même réel : le répertoire dit la vérité sur l'intention, les
valeurs mockées sortent de la logique d'agrégation, et il n'y a **aucun code mort**.

**Précisions apportées en séance** : `Registration` et `Customs` ne sont pas des services
indépendants — ils viennent du même appel registre que les données véhicule. `WantedStatus` est une
fonctionnalité v2 (rôle `supervisor`, signalement de véhicule recherché/volé depuis son tableau de
bord), d'où son `Pending` actuel. `PoliceStatus` et `CustomsStatus` **restent** donc dans
`services/vehicles/status.rs`.

### 6.4 Ce qui est explicitement différé

| Différé | Raison |
|---|---|
| Extraire la logique métier des handlers vers `services/` | Découper `auth.rs` 1457 → 6 fichiers de ~200 l. capture l'essentiel du gain de lisibilité **sans réécriture** |
| `AppState` → `FromRef` par domaine | ~35 signatures pour un gain réel mais non urgent |
| Réorganisation des 9 307 LOC de tests | On met à jour les chemins `use`, rien de plus |
| Isolation tenant portée par le type | Sujet de sécurité, ticket dédié — le refactoring crée seulement le seam |
| Workspace multi-crates | Hors sujet à ce stade |

---

## 7. Points signalés, hors périmètre

1. 🔴 **Isolation tenant** — `get_pending_submissions` (`queries/submission_queries.rs`) ne filtre par
   **aucune** organisation : les deux variantes SQL sélectionnent toutes les lignes de
   `pending_submissions`. Tout utilisateur autorisé sur cet endpoint voit les soumissions de tous les
   tenants. Antérieur à ce travail, déjà relevé dans la session S3. **Ticket dédié prioritaire.**

2. 🟠 **Aucune vérification SQL au compile-time** — 0 macro `sqlx::query!` contre 208 requêtes
   runtime, `.sqlx/` avec une seule entrée. Conséquence directe pour l'étape 4 du plan : les 27 SQL
   extraits doivent être déplacés **à l'identique**, jamais réécrits au passage — les tests
   d'intégration sont le seul filet.

3. 🟠 **Outillage indisponible** — la vérification de la sémantique `utoipa::openapi::OpenApi::merge()`
   via Context7, prescrite par le `CLAUDE.md` du projet, n'a pas pu être faite : les connecteurs
   claude.ai (Context7, Gmail, Google Calendar) demandent une autorisation depuis les réglages de
   connecteurs claude.ai, non réalisable depuis une session non interactive. C'est l'une des raisons
   pour lesquelles l'éclatement de `api_doc.rs` est différé.

4. 🟢 **Divergence doc/code** — `docs/IVISS_Sync_Architecture.md` décrit le service de sync comme un
   proxy HTTP à la demande avec un préfixe plat `vehicles/`, alors que le code implémente un service
   sans surface HTTP et un préfixe partitionné `vehicle-cache/{REGION}/`. À mettre à jour.

---

## 8. Résultat

Plan d'exécution en 7 étapes, chacune mergeable indépendamment et laissant la codebase verte :
filets de sécurité → `external_services/` → `services/` → `handlers/` → `queries/` + extraction des
27 SQL → DTO égarés → éclatement de `routes.rs` → réapplication du plan S3.

**~11 jours-dev**, diff OpenAPI attendu nul, composition RBAC préservée.

Détail complet : [`backend_domain_refactor_plan.md`](backend_domain_refactor_plan.md).
