# Plan de refactoring — Recadrage du backend par domaine

> **Date** : 2026-08-07 · **Branche cible** : `refactor/backend-domain-layout`, clonée depuis `298-implement-s3-cache-layer-provisioning-service`
> **Compte-rendu de session** : [`backend_domain_refactor_session.md`](backend_domain_refactor_session.md)
> **Plan S3 associé** : [`../s3_sync_service_plan.md`](../s3_sync_service_plan.md)
> **Portée** : plan uniquement — **aucun code modifié à ce stade**.

---

## Context

La mission de la branche `298-implement-s3-cache-layer-provisioning-service` est d'implémenter le
service de synchronisation du cache S3 ([`../s3_sync_service_plan.md`](../s3_sync_service_plan.md)).

Avant cette implémentation, on recadre la structure du backend pour que le nouveau service s'inscrive
dans une architecture propre — **sans** payer le coût d'un passage complet en monolithe modulaire.
Le refactoring passe donc avant toute nouvelle implémentation ; le plan S3 sera réappliqué ensuite
sur la structure recadrée.

### Décisions actées

| Sujet | Décision |
|---|---|
| Architecture cible | **Package-by-layer-then-feature** : on garde les 4 couches, on les sous-divise par domaine |
| `routes.rs` | ✅ **Éclaté** (étape 6). Guards RBAC toujours composés centralement |
| `api_doc.rs` | ❌ **Non touché** cette itération (ticket séparé) |
| Services externes | **Véhicule uniquement en réel.** Assurance et visite technique restent **mockés comme aujourd'hui** |
| Disposition des clés S3 | **Inchangée** — `vehicle-cache/{PARTITION}/{PLATE}.json`, `retry-queue/` et `unregistered/` plats |
| Branche | Clonée depuis `298-…`, mergée **dans** `298-…`. `298` → `dev` seulement après validation complète |

---

# PARTIE 1 — Évaluation de l'existant

**Note globale : 6/10.** Monolithe en couches honnête et bien tenu, **pas cassé**.

| Axe | Note | | Axe | Note |
|---|---|---|---|---|
| Organisation / cohésion | 5/10 | | Contrat OpenAPI | 7/10 |
| Couplage / dépendances | 7/10 | | Passage à l'échelle équipe | **4/10** |
| Testabilité | 5/10 | | Boucle build/feedback | 5/10 |
| Sécurité multi-tenant | 6/10 | | Observabilité / ops | **8/10** |

**Volumétrie** : 106 fichiers Rust · 25 427 LOC (16 120 production + 9 307 tests) · 24 migrations ·
1 256 commits · 5-6 contributeurs actifs.

## 1.1 Forces à préserver

| # | Point fort | Preuve |
|---|---|---|
| 1 | **Aucun cycle de dépendance** sur 106 fichiers | `queries/` ne dépend que de `dto` + `errors` ; `services/` ne dépend jamais de `handlers/` |
| 2 | **Convention prévisible** | Faible coût d'onboarding : un dev sait où va une requête ou un DTO |
| 3 | **Deux tranches verticales déjà en place** | [`vehicle_client/mod.rs`](../../iviss-backend/src/vehicle_client/mod.rs) et [`s3_cache_layer/mod.rs`](../../iviss-backend/src/s3_cache_layer/mod.rs) ont une surface publique explicite avec re-exports — l'équipe sait déjà faire |
| 4 | **RBAC centralisé et serveur** | [`routes.rs:103-137`](../../iviss-backend/src/routes.rs#L103-L137) : guards posés par groupe, matrice lisible d'un coup d'œil. **Vraie force du god-router** |
| 5 | **Contrat OpenAPI complet et unique** | [`api_doc.rs`](../../iviss-backend/src/api_doc.rs) garantit qu'aucun endpoint n'échappe au codegen frontend (`frontend/src/openapi-rq`, `@hey-api/openapi-ts`) |
| 6 | **Contrat d'erreur uniforme** | [`errors.rs:104-181`](../../iviss-backend/src/errors.rs#L104-L181) — `AppError` → `IntoResponse` unique, aucune fuite d'internes |
| 7 | **Observabilité niveau production** | OTel + Prometheus + port métriques séparé (9091, hors ingress public) + graceful shutdown + flush télémétrie |
| 8 | **Découpage `feature = "api"` déjà pensé** | Le binaire `s3-cache-sync` réutilise `dto`/`vehicle_client`/`s3_cache_layer` sans tirer axum/sqlx/leptess |
| 9 | **Couverture de test substantielle** | 9 307 LOC, testcontainers avec vrai Postgres + MinIO. **C'est le filet qui rend le refactoring faisable** |

## 1.2 Les 10 problèmes mesurés

| | Problème | Preuve |
|---|---|---|
| **P1** | God-files à fort churn | Churn 6 mois : `routes.rs` **65 commits**, `main.rs` 52, `api_doc.rs` **40**, `app_state.rs` 30, `services/mod.rs` 23, `tests/mod.rs` 22 — × 6 devs. Ajouter un endpoint touche 6-8 fichiers dans 5 répertoires |
| **P2** | Aucune couche de service métier | `services/` = adaptateurs techniques seuls (jwt, otp, email, sms, ocr, cache S3). La logique vit dans les handlers : [`handlers/auth.rs`](../../iviss-backend/src/handlers/auth.rs) = **1 457 lignes** |
| **P3** | 27 violations de couche | `sqlx::query` brut hors `queries/` : `auth.rs` **×18**, `user_management.rs` ×5, `pending_submission.rs` ×2, `search_vehicle.rs` ×1, **[`middleware/rbac.rs:45-51`](../../iviss-backend/src/middleware/rbac.rs#L45-L51) ×1** |
| **P4** | Localité cognitive faible | « daily login » = `dto/auth.rs` + `handlers/auth.rs` + `queries/auth_queries.rs` + `services/otp_service.rs` + `app_cache.rs` + `middleware/auth.rs` + `routes.rs` + `api_doc.rs` |
| **P5** | `AppState` god-object (11 champs) | Le handler OCR (aucun besoin DB) reçoit `db`, `jwt_svc`, `email_svc`… Aucune expression compile-time du moindre privilège |
| **P6** | Isolation tenant = convention | `organization_id` filtré dans 6 des 10 modules `queries/`. **Rien n'empêche d'écrire une requête sans le filtre.** Risque n°1 sur un produit forces de l'ordre |
| **P7** | Tests = seau plat intra-crate | 9 307 LOC (37 % du code) compilés dans le binaire de test de la lib → toute modif recompile tout |
| **P8** | 0 vérification SQL compile-time | **0** macro `sqlx::query!`/`query_as!` contre **208** requêtes runtime. `.sqlx/` = 1 entrée (cache offline inutilisé) |
| **P9** | Code mort | `feature_flags.rs` (44 l., jamais déclaré dans `lib.rs`), `services/vehicle_client_service.rs` (commenté dans `services/mod.rs:9`) |
| **P10** | `#[cfg(feature = "api")]` dispersé | `lib.rs`, `dto/mod.rs`, `utils/mod.rs` — un problème de **frontière de crate** résolu au préprocesseur |

**Bonus** — 7 structs DTO déclarées **dans les handlers** :
[`stats.rs:36,70,103,133`](../../iviss-backend/src/handlers/stats.rs#L36) (`ActivityQuery`,
`TopAgentsQuery`, `ActivityFeedQuery`, `RecentAlertsQuery`) et
[`auth.rs:940,946,954`](../../iviss-backend/src/handlers/auth.rs#L940) (`RefreshChallengeResponse`,
`VerifyRefreshRequest`, `VerifyRefreshResponse`), d'où les références `crate::handlers::auth::…`
dans [`api_doc.rs:202-204`](../../iviss-backend/src/api_doc.rs#L202-L204).

## 1.3 Verdict

L'architecture actuelle est **au-dessus de la moyenne** pour son âge et n'est pas cassée. Mais à
16k LOC de production, 6 devs concurrents et une roadmap qui ajoute 3-4 domaines, le package-by-layer
a franchi le point où il **coûte plus qu'il ne rapporte** : coût marginal d'un endpoint = 6-8 fichiers
et 3 conflits potentiels.

**Le vrai gain n'est pas l'arborescence** — c'est (a) sortir la logique de `handlers/auth.rs` et
(b) transformer l'isolation tenant en type. Les dossiers sont le véhicule, pas la destination.

---

# PARTIE 2 — Pourquoi ce périmètre plutôt qu'un monolithe modulaire complet

## 2.1 Propriété décisive : ce n'est pas une impasse

Package-by-layer-then-feature est l'intermédiaire canonique entre package-by-layer (actuel) et
package-by-feature (monolithe modulaire). Si les répertoires de domaine portent le **même nom dans les
quatre couches**, la bascule ultérieure est un pur `git mv`, sans une ligne de logique modifiée :

```bash
git mv src/handlers/auth  src/modules/auth/api
git mv src/services/auth  src/modules/auth/service
git mv src/queries/auth   src/modules/auth/infra
git mv src/dto/auth.rs    src/modules/auth/api/dto.rs
```

👉 **Règle n°1 non négociable : nomenclature identique sur les 4 couches.**
`auth`, `users`, `organizations`, `controls`, `vehicles`, `submissions`, `ocr`, `stats`, `audit`,
`notifications`. Pas de `users_query/` d'un côté et `user_management/` de l'autre.

## 2.2 Coût comparé

| | Ce plan | Monolithe modulaire complet |
|---|---|---|
| Fichiers déplacés | ~60 (`git mv` + `mod.rs`) | ~106 + réorganisation des tests |
| Logique réécrite | 27 SQL extraits + 2 gros handlers découpés | idem + couche service complète + `AppState`→`FromRef` |
| Signatures modifiées | **0** | ~35 handlers |
| Contrat OpenAPI | diff attendu = **0** | refonte de `api_doc.rs`, golden-file obligatoire |
| Composition RBAC | **préservée** (guards centraux) | recomposition complète (risque n°1) |
| Suite de tests | mise à jour des chemins `use` | déplacement vers `tests/` + réécriture du harnais |
| **Effort** | **~11 j-dev ≈ 2 à 2,5 semaines** | **6-8 semaines** |
| **Risque** | 🟢 faible | 🔴 élevé |
| **Valeur captée** | **~65 %** | 100 % |

## 2.3 Couverture des problèmes

| | Ce plan | Modular monolith |
|---|---|---|
| **P1** god-files | 🟢 `routes.rs` 222 → ~45 l. · `api_doc.rs` différé | ✅ |
| **P2** pas de couche service | 🟡 `auth.rs` 1457 → 6 fichiers ; logique non extraite (assumé, §4.6) | ✅ |
| **P3** 27 violations de couche | ✅ **corrigé** | ✅ |
| **P4** localité cognitive | 🟢 | ✅ |
| **P5** `AppState` god-object | ❌ différé | ✅ |
| **P6** isolation tenant | 🟡 le seam est créé | 🟡 ticket dédié dans les deux cas |
| **P7** tests seau plat | ❌ différé | ✅ |
| **P8** 0 `sqlx::query!` | ❌ orthogonal à toute structure | ❌ |
| **P9** code mort | ✅ | ✅ |
| **P10** `cfg(feature="api")` | 🟡 clarifié par `external_services/` | ✅ (si workspace) |

---

# PARTIE 3 — Arborescence cible

```
iviss-backend/src/
├── main.rs
├── lib.rs
├── routes.rs                    ★ 222 → ~45 lignes (§4.1)
├── api_doc.rs                   ❌ NON TOUCHÉ cette itération (§4.5)
├── app_state.rs   app_cache.rs   config.rs   errors.rs   telemetry.rs
│
├── external_services/           ★ NOUVEAU — ports vers les partenaires
│   ├── mod.rs                   ★ trait ExternalDataSource (§4.2)
│   ├── vehicle_client/          ← git mv depuis src/vehicle_client/   [RÉEL]
│   │   └── mod.rs  client.rs  parser.rs  types.rs
│   ├── insurance_client/        [MOCK — transport non décidé]
│   │   └── mod.rs               pending_insurance_status()  ← extrait de vehicle_status_service.rs
│   └── technical_inspection_client/   [MOCK — transport non décidé]
│       └── mod.rs               pending_technical_status()  ← extrait de vehicle_status_service.rs
│
├── s3_cache_layer/              ⚠ NE PAS DÉPLACER (§4.4)
│   └── config.rs  crypto.rs  s3_reader.rs  s3_writer.rs  types.rs
│
├── handlers/                    ★ sous-divisé par domaine
│   ├── mod.rs
│   ├── auth/
│   │   ├── mod.rs               pub use login::login; …  → chemins externes INCHANGÉS (§3.3)
│   │   ├── login.rs             ← auth.rs:41-209
│   │   ├── logout.rs            ← auth.rs:212-310
│   │   ├── activate.rs          ← auth.rs:312-526
│   │   ├── daily_login.rs       ← auth.rs:528-930
│   │   ├── refresh.rs           ← auth.rs:931-1377
│   │   ├── change_password.rs   ← auth.rs:1379-1457
│   │   └── router.rs            ★ public_routes() / web_auth_routes()
│   ├── users/
│   │   ├── mod.rs  router.rs
│   │   ├── profile.rs           ← users.rs (get_user_profile)
│   │   ├── location.rs          ← users.rs (update_location)
│   │   ├── provisioning.rs      ← user_management.rs (provision, list, get, update, delete, org)
│   │   ├── sessions.rs          ← user_management.rs (terminate_session, restart_session)
│   │   └── activation.rs        ← user_management.rs (resend_activation_code, resend_org_admin_password)
│   ├── organizations/  mod.rs router.rs crud.rs      ← organization_management.rs
│   ├── controls/       mod.rs router.rs             ← list_control.rs
│   ├── vehicles/       mod.rs router.rs search.rs   ← search_vehicle.rs
│   ├── submissions/    mod.rs router.rs             ← pending_submission.rs
│   ├── ocr/            mod.rs router.rs scan.rs photo.rs
│   ├── stats/          mod.rs router.rs admin.rs org.rs agent.rs
│   ├── audit/          mod.rs router.rs             ← audit.rs
│   └── health.rs                                    (1 endpoint — reste un fichier)
│
├── services/                    ★ sous-divisé par domaine
│   ├── mod.rs
│   ├── auth/            mod.rs  jwt.rs  otp.rs
│   ├── notifications/   mod.rs  email_provider.rs  email_service.rs  sms_provider.rs   (§4.3)
│   ├── ocr/             mod.rs  engine.rs  scan.rs photo.rs  timings.rs
│   └── vehicles/        mod.rs  status.rs  data_cache.rs
│
├── queries/                     ★ suffixes retirés + destination des 27 SQL extraits
│   ├── mod.rs
│   ├── auth/            mod.rs  credentials.rs  devices.rs  refresh_tokens.rs  sessions.rs
│   │                    ← auth_queries.rs + session_queries.rs + 18 SQL de handlers/auth.rs
│   │                      + 1 SQL de middleware/rbac.rs
│   ├── users/           mod.rs  profile.rs  provisioning.rs  location.rs
│   │                    ← user_queries.rs + location_queries.rs + 5 SQL de user_management.rs
│   ├── organizations.rs ← organization_queries.rs      (499 l., reste un fichier)
│   ├── controls.rs      ← control_queries.rs           (499 l.)
│   ├── vehicles.rs      ← vehicle_queries.rs + 1 SQL de search_vehicle.rs
│   ├── submissions.rs   ← submission_queries.rs + 2 SQL de pending_submission.rs
│   ├── stats.rs         ← stats_queries.rs             (570 l.)
│   └── audit.rs         ← audit_queries.rs
│
├── dto/                         noms déjà alignés — on récupère les 7 DTO égarés
│   ├── auth.rs                  + RefreshChallengeResponse, VerifyRefreshRequest, VerifyRefreshResponse
│   ├── stats.rs                 + ActivityQuery, TopAgentsQuery, ActivityFeedQuery, RecentAlertsQuery
│   └── controls.rs              ← fusion de create_control.rs + list_control.rs
│
├── middleware/  auth.rs  rbac.rs  cors.rs  metrics.rs   (le SQL de rbac.rs part dans queries/auth/)
└── models/   utils/   db/   bin/   tests/                inchangés cette itération
```

## 3.1 Règle de promotion fichier → répertoire

`queries/organizations.rs` et `queries/organizations/mod.rs` sont **équivalents pour l'appelant** :
`use crate::queries::organizations::…` fonctionne dans les deux cas. La promotion est **non cassante,
réversible et invisible**.

👉 **Répertoire seulement si > 400 LOC ou > 1 fichier** après absorption des SQL extraits. Aujourd'hui
cela ne concerne que `queries/auth/` (366 + 18 blocs + 95 ≈ 600 l.) et `queries/users/`
(436 + 228 + 5 blocs ≈ 700 l.). Créer 8 répertoires à un seul fichier serait de la cérémonie pure.

## 3.2 Suppression du suffixe `_queries`

`queries/auth_queries.rs` → `queries/auth.rs`. Le suffixe est redondant dans `queries/` et casse
l'alignement 1:1 entre couches — condition de la réversibilité du §2.1.

## 3.3 ★ Re-exports : le chemin externe ne bouge pas

`handlers/auth.rs` devient `handlers/auth/mod.rs` avec :

```rust
mod login; mod logout; mod activate; mod daily_login; mod refresh; mod change_password;
pub mod router;
pub use login::login;
pub use logout::logout;
pub use activate::activate;
pub use daily_login::{request_daily_login, verify_daily_login};
pub use refresh::{request_refresh, verify_refresh};
pub use change_password::change_password;
```

`crate::handlers::auth::login` reste valide → **`api_doc.rs` (churn 40, critique pour le contrat
frontend) et `routes.rs` ne changent pas d'une ligne** pendant les étapes 2-3. Diff OpenAPI = **zéro**.

C'est la mesure de réduction de risque la plus importante du plan : elle découple totalement le
découpage des fichiers du contrat d'API.

---

# PARTIE 4 — Décisions de conception

## 4.1 ★ Éclatement de `routes.rs` — la sécurité RBAC préservée par construction

`routes.rs` est le fichier **n°2 en churn** (65 commits/6 mois, 6 devs).

**Le point critique** : aujourd'hui les guards sont posés **par groupe**. Si chaque domaine posait ses
propres layers, une route pourrait silencieusement perdre son `require_admin` → escalade de
privilèges. **On ne fait donc PAS ça.** Les domaines exposent des routers **non protégés** ;
l'assemblage central applique les layers :

```rust
// handlers/stats/router.rs — AUCUN layer ici
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/admin/stats", get(super::admin::get_dashboard_stats))
}
pub fn org_admin_routes() -> Router<Arc<AppState>> { … }
pub fn agent_routes()     -> Router<Arc<AppState>> { … }

// routes.rs — la matrice RBAC reste ICI, lisible d'un coup d'œil
let admin_routes = Router::new()
    .merge(handlers::submissions::router::admin_routes())
    .merge(handlers::users::router::admin_routes())
    .merge(handlers::organizations::router::admin_routes())
    .merge(handlers::controls::router::admin_routes())
    .merge(handlers::audit::router::admin_routes())
    .merge(handlers::stats::router::admin_routes())
    .layer(from_fn_with_state(state.clone(), rbac::require_admin))
    .layer(from_fn_with_state(state.clone(), rbac::require_auth_web));
```

`routes.rs` : 222 → ~45 lignes. Ajouter un endpoint ne le touche plus. **Filet obligatoire** :
`tests/contract_rbac.rs`, écrit **avant** le découpage sur la structure actuelle.

## 4.2 ★★ `external_services/` : trait `ExternalDataSource` + périmètre réel restreint

Le mode d'accès à `insurance` et `technical_inspection` n'est pas décidé (API HTTP ou accès direct à
leur base). C'est le cas d'usage d'un **port** : on fige le contrat maintenant, on décide du transport
plus tard, sans rien réécrire en amont.

```rust
// external_services/mod.rs — PAS de #[cfg(feature = "api")] : le binaire sync en dépend
#[async_trait]
pub trait ExternalDataSource: Send + Sync {
    /// Identifiant stable — label de métrique et, plus tard, préfixe S3.
    fn service_id(&self) -> &'static str;

    async fn fetch(&self, plate: &str) -> Result<PartnerPayload, ExternalServiceError>;

    /// Sonde de vie. Le HEALTH_PROBE_PLATE du plan S3 devient un détail
    /// interne à vehicle_client au lieu d'une constante du binaire sync.
    async fn health_probe(&self) -> HealthStatus;
}

pub enum ExternalServiceError { NotFound, Unavailable(String), Protocol(String) }
```

**Périmètre réel de cette itération — décision actée** : seul `vehicle_client` implémente le trait.
`insurance_client/` et `technical_inspection_client/` existent comme répertoires mais ne contiennent
que **le mock actuel, déplacé sans changement de comportement** : les constructeurs
`InsuranceStatus { status: Pending, notes: "No insurance data available for the moment", … }` et
`TechnicalStatus { … }` aujourd'hui codés en dur dans
[`vehicle_status_service.rs:53-90`](../../iviss-backend/src/services/vehicle_status_service.rs#L53-L90)
en sortent pour devenir `insurance_client::pending_insurance_status()` et
`technical_inspection_client::pending_technical_status()`.

Ils restent **synchrones** et n'implémentent pas encore le trait — le faire imposerait de rendre
`build_status_results_from_api` async, ce qui remonterait jusqu'à `search_vehicle` : un ripple non
justifié pour du mock. Le jour où le transport réel est décidé, ces deux fichiers implémentent
`ExternalDataSource` et `services/vehicles/status.rs` devient async — un changement localisé.

Bénéfice immédiat : le répertoire dit la vérité sur l'intention, les valeurs mockées ne sont plus
noyées dans la logique d'agrégation, et il n'y a **aucun code mort**.

`PoliceStatus` (wanted/stolen) **reste** dans `services/vehicles/status.rs` : ce n'est pas un service
externe mais une fonctionnalité v2 du rôle `supervisor`. Même chose pour `CustomsStatus`, dérivé de la
réponse du registre ([`vehicle_status_service.rs:94`](../../iviss-backend/src/services/vehicle_status_service.rs#L94)).

## 4.3 `services/auth/` ne doit PAS contenir email et SMS

Ces transports sont utilisés par **deux** domaines : `auth` (OTP de connexion journalière) **et**
`users` ([`user_management.rs:339`](../../iviss-backend/src/handlers/user_management.rs#L339)
`resend_activation_code`, [`:476`](../../iviss-backend/src/handlers/user_management.rs#L476)
`resend_org_admin_password`).

Sous `auth/`, `handlers/users/` devrait importer `services/auth/email` pour renvoyer un code
d'activation — un couplage inversé sans justification.

👉 `services/notifications/{email_provider, email_service, sms_provider}` + `services/auth/{jwt, otp}`.
L'OTP appartient bien à `auth` ; il **utilise** les transports de notification.

## 4.4 Ne pas déplacer `s3_cache_layer/`

1. Ce n'est pas un service **externe** (partenaire tiers), c'est **votre** infrastructure de stockage.
2. Il est déjà propre — `mod.rs` avec surface publique explicite, le modèle à répliquer.
3. **Il est sur le chemin critique de la branche en cours** : le plan S3 y ajoute `s3_queue.rs`,
   `retry_queue_key`, `QueueMarker`. Le déplacer ferait entrer en collision refactoring et
   implémentation. *Principe du diff minimal : on ne déplace pas ce qui est déjà bon et en travaux.*

## 4.5 Différer l'éclatement de `api_doc.rs` — assumé

- Le décentraliser impose `utoipa::openapi::OpenApi::merge()` par domaine, dont la sémantique sur
  `components.schemas` et les `modifiers` (`SecurityAddon`) doit être vérifiée. `utoipa-axum` /
  `OpenApiRouter` exigerait **utoipa 5** — une montée de version qui, seule, risque de faire dériver
  le contrat consommé par `frontend/src/openapi-rq`.
- Grâce au §3.3, ce refactoring produit un **diff OpenAPI nul**. Y toucher casserait cette propriété.

👉 Ticket séparé, après stabilisation.

## 4.6 Ce qu'on ne fait **pas** cette itération

| Différé | Raison |
|---|---|
| Extraire la logique métier vers `services/` (**P2**) | Découper `auth.rs` 1457 → 6 fichiers de ~200 l. capture l'essentiel du gain de lisibilité **sans réécriture**. L'extraction complète est un travail à part, domaine par domaine. |
| `AppState` → `FromRef` (**P5**) | ~35 signatures pour un gain réel mais non urgent. Mauvais ratio ici. |
| Réorganisation des tests (**P7**) | 9 307 LOC ; on met à jour les chemins `use`, rien de plus. |
| Isolation tenant typée (**P6**) | 🔴 **Sujet de sécurité, ticket dédié et prioritaire** — voir §7. |
| Workspace multi-crates (**P10**) | Hors sujet à ce stade. |

---

# PARTIE 5 — Impact sur le plan S3

**Le plan [`../s3_sync_service_plan.md`](../s3_sync_service_plan.md) reste valide sans amendement
fonctionnel.** Périmètre confirmé : véhicule seul en réel, assurance et visite technique mockés,
**disposition des clés inchangée** (`vehicle-cache/{PARTITION}/{PLATE}.json` partitionné,
`retry-queue/` et `unregistered/` plats).

Seuls trois points d'adaptation, purement mécaniques :

1. **Chemins d'import** : `crate::vehicle_client::…` → `crate::external_services::vehicle_client::…`
   dans `bin/s3-cache-sync.rs`, `app_state.rs`, `config.rs`, `handlers/vehicles/search.rs`.
2. **Sonde de santé** : `HEALTH_PROBE_PLATE = "CE128BC"` devient une constante privée de
   `vehicle_client`, exposée via `ExternalDataSource::health_probe()`. Le binaire sync n'a plus
   connaissance d'une plaque sentinelle. Le test `assert!(plate_format::is_valid(HEALTH_PROBE_PLATE))`
   déménage avec la constante.
3. **Emplacements cibles** : l'étape 0 du plan S3 (réparer `services/vehicle_data_cache.rs`, qui ne
   compile pas) s'applique désormais à `services/vehicles/data_cache.rs` ; l'étape 2 (write-through)
   à `handlers/vehicles/search.rs` ; l'étape 4 (fusion `unregistered/`) à `handlers/submissions/`.

Restent valides tels quels : write-through détaché sur succès · `NotFound` → 404 sec sans cache ·
suppression de `fetch_batch`/`GET /batch` · fusion `unregistered/` dans `GET /api/v1/admin/submissions` ·
marqueur supprimé après confirmation d'écriture (at-least-once idempotent) · corrections de config
(`S3_CACHE_SSE_KMS_KEY_ID`, `EXTERNAL_API_*`, valeur unique de `S3_CACHE_BUCKET`).

---

# PARTIE 6 — Plan d'exécution

**Branche** : `refactor/backend-domain-layout`, clonée depuis
`298-implement-s3-cache-layer-provisioning-service`, mergée **dans** cette même branche.
`298` → `dev` seulement après validation complète.

Chaque étape est **mergeable indépendamment** et laisse la codebase verte.

| # | Étape | Effort | Risque |
|---|---|---|---|
| **0** | **Filets de sécurité** : `tests/contract_openapi.rs` (golden-file JSON normalisé) + `tests/contract_rbac.rs` (matrice route × rôle → 401/403/2xx), écrits sur la structure **actuelle**. Supprimer `feature_flags.rs` et `services/vehicle_client_service.rs` (**P9**). | 1 j | 🟢 |
| **1** | **`external_services/`** : `git mv src/vehicle_client external_services/vehicle_client` ; créer `external_services/mod.rs` avec le trait `ExternalDataSource` ; l'implémenter sur `VehicleApiService`. Créer `insurance_client/mod.rs` et `technical_inspection_client/mod.rs` avec les constructeurs de statut mockés extraits de `vehicle_status_service.rs` (§4.2) — **valeurs identiques au bit près**. Vérifier qu'aucun de ces modules n'est gaté par `feature = "api"`. | 1,5 j | 🟢 |
| **2** | **`services/`** : sous-répertoires `auth/`, `notifications/`, `ocr/`, `vehicles/` (§4.3). Renommages : `jwt_service.rs`→`auth/jwt.rs`, `otp_service.rs`→`auth/otp.rs`, `ocr_service.rs`→`ocr/engine.rs`, `photo_ocr_service.rs`→`ocr/photo.rs`, `ocr_timings.rs`→`ocr/timings.rs`, `vehicle_status_service.rs`→`vehicles/status.rs`, `vehicle_data_cache.rs`→`vehicles/data_cache.rs`. | 1 j | 🟢 |
| **3** | **`handlers/`** : sous-répertoires par domaine + découpage de `auth.rs` (1457→6) et `user_management.rs` (664→3). **`mod.rs` re-exporte tout** (§3.3) → `routes.rs` et `api_doc.rs` intouchés. | 2,5 j | 🟢 |
| **4** | **`queries/` + extraction des 27 SQL** (**P3**) : suffixes `_queries` retirés ; `queries/auth/` et `queries/users/` promus en répertoires. Sortir les 18 SQL de `handlers/auth`, 5 de `handlers/users`, 2 de `submissions`, 1 de `vehicles`, **1 de `middleware/rbac.rs`**. ⚠ Déplacer les requêtes **à l'identique**, ne jamais les réécrire au passage (aucune vérification SQL au compile-time, **P8** — les tests d'intégration sont le seul filet). | 2,5 j | 🟠 |
| **5** | **DTO égarés** : les 7 structs des handlers → `dto/auth.rs` et `dto/stats.rs` ; fusion `create_control.rs` + `list_control.rs` → `dto/controls.rs`. Mettre à jour `api_doc.rs:202-204`. ⚠ utoipa nomme les schémas d'après le **nom de struct**, pas le chemin de module → diff OpenAPI attendu = 0, **à valider par le golden-file**. | 0,5 j | 🟠 |
| **6** | **Éclater `routes.rs`** (§4.1, **P1**) : `handlers/<domaine>/router.rs` non protégés, guards composés dans `routes.rs`. Valider par `contract_rbac.rs`. | 2 j | 🟠 |
| **7** | **Réappliquer le plan S3** avec les adaptations du §5. | *(plan séparé)* | 🟠 |

**Total étapes 0-6 : ~11 jours-dev ≈ 2 à 2,5 semaines.**

**Ordre imposé** : 0 avant tout · 4 après 3 (les SQL doivent d'abord être dans leur fichier de
destination) · 6 après 3 · 7 en dernier.

## Fichiers critiques

[`handlers/auth.rs`](../../iviss-backend/src/handlers/auth.rs) (1457 l., 18 SQL, churn 84) ·
[`routes.rs`](../../iviss-backend/src/routes.rs) (churn 65) ·
[`api_doc.rs`](../../iviss-backend/src/api_doc.rs) (churn 40, **ne pas restructurer**) ·
[`handlers/user_management.rs`](../../iviss-backend/src/handlers/user_management.rs) (664 l., 5 SQL) ·
[`middleware/rbac.rs`](../../iviss-backend/src/middleware/rbac.rs) (1 SQL) ·
[`services/vehicle_status_service.rs`](../../iviss-backend/src/services/vehicle_status_service.rs) (extraction des mocks) ·
[`lib.rs`](../../iviss-backend/src/lib.rs) (déclarations + `cfg(feature)`) ·
[`src/tests/mod.rs`](../../iviss-backend/src/tests/mod.rs) (chemins `use` à mettre à jour partout).

## Code à réutiliser tel quel — ne pas réécrire

[`vehicle_client/mod.rs`](../../iviss-backend/src/vehicle_client/mod.rs) et
[`s3_cache_layer/mod.rs`](../../iviss-backend/src/s3_cache_layer/mod.rs) — modèle de surface publique
à répliquer dans chaque `mod.rs` de domaine ·
[`errors.rs`](../../iviss-backend/src/errors.rs) ·
`setup_test_app()` dans [`src/tests/stats_handler_tests.rs:28`](../../iviss-backend/src/tests/stats_handler_tests.rs#L28)
(à factoriser plutôt qu'à dupliquer dans les nouveaux tests de contrat).

## Contraintes transverses

- **`git mv` systématique** (jamais delete + create) pour préserver `git log --follow`.
- Rebaser toute branche ouverte sur `298` avant l'étape 3 — ~60 déplacements génèrent des conflits massifs.
- `external_services/` et `s3_cache_layer/` restent **hors** de `#[cfg(feature = "api")]` : le binaire
  `s3-cache-sync` compile avec `--no-default-features`.
- Les migrations restent globales (`sqlx::migrate!` ne prend qu'un répertoire).

---

# PARTIE 7 — Points signalés, hors périmètre

1. 🔴 **Isolation tenant (P6)** — `get_pending_submissions`
   ([`queries/submission_queries.rs`](../../iviss-backend/src/queries/submission_queries.rs)) ne filtre
   par **aucune** organisation : les deux variantes SQL sélectionnent toutes les lignes de
   `pending_submissions`. Tout utilisateur autorisé sur cet endpoint voit les soumissions de tous les
   tenants. Antérieur à ce refactoring, déjà relevé dans le plan S3. **Ticket dédié prioritaire.**
   Ce refactoring crée le seam (`queries/<domaine>/`) où le corriger proprement.

2. 🟠 **Absence de vérification SQL au compile-time (P8)** — 0 macro `sqlx::query!` contre 208
   requêtes runtime, `.sqlx/` avec une seule entrée. Migrer progressivement vers les macros
   compile-checked donnerait un filet que ce refactoring n'a pas. Orthogonal à toute structure.

3. 🟠 **`api_doc.rs` reste un god-file** (churn 40) — voir §4.5. À traiter après stabilisation,
   avec le golden-file comme préalable et une décision explicite sur utoipa 4 vs 5.

4. 🟢 **Divergence doc/code** — [`../IVISS_Sync_Architecture.md`](../IVISS_Sync_Architecture.md)
   décrit le service de sync comme un proxy HTTP à la demande avec un préfixe plat `vehicles/`, alors
   que le code implémente un service sans surface HTTP et un préfixe partitionné
   `vehicle-cache/{REGION}/`. À mettre à jour.

---

# Vérification

Après **chaque** étape, sans exception :

```bash
cd iviss-backend
cargo fmt --check
cargo check                                             # profil api (défaut)
cargo check --no-default-features --bin s3-cache-sync   # ★ contrainte du binaire sync
cargo clippy --all-targets -- -D warnings

# Contrat frontend — doit produire ZÉRO diff sur les étapes 1 à 6
cargo run --bin export_openapi > /tmp/openapi.after.json
diff <(jq -S . /tmp/openapi.before.json) <(jq -S . /tmp/openapi.after.json)

cargo test --test contract_openapi      # golden-file (étape 0)
cargo test --test contract_rbac         # matrice route × rôle (étape 0)
cargo test                              # suite complète — Docker requis (testcontainers PG + MinIO)
```

Contrôle spécifique à l'étape 1 (extraction des mocks assurance / visite technique) :

```bash
cargo test vehicle_status          # StatusResults doit être identique avant/après
```

→ ajouter un test verrouillant les valeurs mockées : `status == Pending`, `notes ==
"No insurance data available for the moment"` / `"No technical inspection data available for the moment"`,
et `overall_status == Pending` sur un véhicule au dédouanement valide.

Vérification bout-en-bout, après l'étape 6 :

```bash
docker compose up -d                                   # backend + postgres + minio
cd frontend && npm run predev && npm run codegen
git diff --stat frontend/src/openapi-rq/               # ★ doit être VIDE
npm run build && npm run test
```

Contrôles fonctionnels manuels (tous les chemins RBAC ne sont pas couverts par les tests) :
login admin → `/api/v1/admin/stats` (200) · même token sur `/api/v1/org-admin/stats` (403) ·
token agent sur `/api/v1/admin/users` (403) · flux daily-login OTP complet ·
`POST /api/v1/scan/plate` · `POST /api/v1/vehicles/search` sur une plaque de `seeds/vehicles.sql`
(les statuts assurance/technique doivent rester `pending` avec les mêmes libellés qu'avant).
