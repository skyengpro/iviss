# Tenant isolation for pending submissions (DB) + unregistered plates (S3)

## Context

The admin submissions list is currently cross-tenant on **both** of its data sources.

1. **DB half.** [`list_pending_submissions`](iviss-backend/src/handlers/submissions/submissions.rs#L108) takes only `State` and `Query` — it never reads the `AuthenticatedAdmin` extension that `require_auth_web` already injects ([rbac.rs:75-80](iviss-backend/src/middleware/rbac.rs#L75-L80)). [`get_pending_submissions`](iviss-backend/src/queries/submissions.rs#L53) has no `org_id` parameter. Because [`require_admin`](iviss-backend/src/middleware/rbac.rs#L101) admits `org_admin` as well as `admin`, an org_admin of org A reads every org's submissions. Same gap on [`get_pending_submission`](iviss-backend/src/handlers/submissions/submissions.rs#L179) and [`get_submission_audit_log`](iviss-backend/src/handlers/submissions/submissions.rs#L205) — plain IDOR on any UUID, including the gray-card images.

2. **S3 half.** `unregistered/` markers are written with a flat, org-less key ([`unregistered_key`](iviss-backend/src/s3_cache_layer/types.rs#L82)) and listed in full with `usize::MAX` ([data_cache.rs:71-83](iviss-backend/src/services/vehicles/data_cache.rs#L71-L83)), then folded into the same list. Every org sees every other org's unregistered plates — i.e. which plates each tenant searched and failed to find.

3. **Two supporting defects that make the fix meaningless if left alone.** `submit_vehicle` takes `agent_id` from the request body, and [`resolve_agent_id`](iviss-backend/src/queries/submissions.rs#L413) falls back to `SELECT id FROM users ORDER BY created_at ASC LIMIT 1` — so a submission can be attributed to an arbitrary user in another org, which would poison the join we are about to filter on. And `search_vehicle` reads `organization_id` from the client payload with `unwrap_or_else(Uuid::new_v4)` ([search.rs:219](iviss-backend/src/handlers/vehicles/search.rs#L219)) — untrusted, and garbage when absent.

**Outcome:** each organization sees only its own pending submissions and its own unregistered plates; the platform `admin` role keeps cross-tenant oversight.

## Decisions taken

| Decision | Choice |
| --- | --- |
| S3 layout | Partition **both** marker prefixes by org UUID: `retry-queue/{org_id}/{PLATE}.json`, `unregistered/{org_id}/{PLATE}.json` |
| `vehicle-cache/` | **Unchanged and shared** — registry data is public-registry fact, not tenant data. Partitioning it would multiply storage and external API load per org. |
| DB scoping | Join through `users.organization_id` (the pattern already in [queries/stats.rs:324-337](iviss-backend/src/queries/stats.rs#L324-L337)). No migration. |
| Role scope | `admin` (org = NULL) sees all tenants; `org_admin` sees only its own org |
| Legacy flat keys (`unregistered/{PLATE}.json`, `retry-queue/{PLATE}.json`) | **Fully superseded.** The system is pre-deployment — no production objects exist under the old flat layout. All key builders now require `org_id: Uuid`. Any residual flat objects in local MinIO can be deleted manually (`mc rm --recursive`). No legacy fallback code is needed. |
| IAM | Harden now — see the caveat below |

### IAM caveat — read this before the infra step

Per-**org** IAM scoping is **not achievable** in this change. Backend and sync share one static IAM credential pair (`AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`, [.env.example:144-146](.env.example#L144-L146)), and both services legitimately touch every org's prefix. An `s3:prefix` condition cannot separate tenants without per-tenant credentials, which do not exist. Prefix partitioning is the *prerequisite* for that; it is not itself the boundary. **Tenant isolation here is enforced in application code.**

What is achievable now, and what the infra step delivers: **least privilege between the two services**, plus getting the bucket under Terraform at all (today it exists only as a MinIO `mc mb` in [docker-compose.yml:98](docker-compose.yml#L98) and a bucket name in env — there is no Terraform for it; [oidc.tf:59-66](infra/terraform/oidc.tf#L59-L66) covers only the tfstate bucket).

## Implementation

### 1. S3 key layer — `iviss-backend/src/s3_cache_layer/types.rs`

Key builders take a **typed `Uuid`**, never a `&str` — this is what prevents prefix injection / traversal via the org segment, and mirrors the existing `ensure_valid_plate` guard at [types.rs:63](iviss-backend/src/s3_cache_layer/types.rs#L63).

```rust
pub fn retry_queue_key(org_id: Uuid, plate: &str) -> Result<String>   // retry-queue/{org_id}/{plate}.json
pub fn unregistered_key(org_id: Uuid, plate: &str) -> Result<String>  // unregistered/{org_id}/{plate}.json

pub fn org_prefix(prefix: &str, org_id: Uuid) -> String               // "{prefix}{org_id}/"

/// Parses `{prefix}{uuid}/{PLATE}.json`. Rejects non-UUID segments and
/// anything with extra path segments.
pub fn org_plate_from_key(key: &str, prefix: &str) -> Option<(Uuid, String)>
```

The old flat `plate_from_key` function is **removed** — there are no legacy flat objects to parse (system is pre-deployment). `object_key` (for `vehicle-cache/`) is untouched.

Tests to add here: round-trip `retry_queue_key` → `org_plate_from_key`; round-trip `unregistered_key` → `org_plate_from_key`; `org_plate_from_key` rejects `unregistered/not-a-uuid/X.json`, rejects a traversal-shaped segment, and returns `None` on a flat key without UUID segment.

### 2. S3 queue layer — `iviss-backend/src/s3_cache_layer/s3_queue.rs`

- `enqueue_plate(client, bucket, org_id, plate)` and `mark_unregistered(client, bucket, org_id, plate)` take `org_id: Uuid`.
- `ListedMarker` gains `organization_id: Uuid` (always present — no legacy flat markers exist).
- `list_markers` gains a parse mode so callers can list either one org's prefix or the whole prefix across tenants:
  - `list_unregistered_markers(client, bucket, org_id: Uuid, max)` → lists `org_prefix(UNREGISTERED_PREFIX, org_id)` only.
  - `list_all_unregistered_markers(client, bucket, max)` → lists `UNREGISTERED_PREFIX`, parsing each key with `org_plate_from_key`. Keys that don't match the org-partitioned format are skipped (should not exist).
  - `list_queued_markers(client, bucket, max)` (drain-side) → returns `(Uuid, plate)` pairs across the whole retry queue, parsed via `org_plate_from_key`.
- `remove_marker` takes `org_id: Uuid` and reconstructs the full key from `retry_queue_key(org_id, plate)`. Keep the `debug_assert_eq!(prefix, RETRY_QUEUE_PREFIX)` guard at [s3_queue.rs:142](iviss-backend/src/s3_cache_layer/s3_queue.rs#L142).

`org_id` is a required `Uuid` parameter everywhere — forgetting it is a compile error, not a silent full-bucket list.

### 3. Cache service trait — `iviss-backend/src/services/vehicles/data_cache.rs`

```rust
pub enum UnregisteredScope { Organization(Uuid), AllTenants }

async fn enqueue_retry(&self, org_id: Uuid, plate: &str) -> Result<()>;
async fn list_unregistered(&self, scope: UnregisteredScope) -> Result<Vec<UnregisteredPlate>>;
```

`get_vehicle_data` / `store_vehicle_data` keep their plate-only signatures (shared cache). `UnregisteredPlate` gains `organization_id: Uuid` — always known since all markers are org-partitioned.

### 4. Trusted org on the agent path

`AuthenticatedUser` ([middleware/auth.rs:14-19](iviss-backend/src/middleware/auth.rs#L14-L19)) gains `organization_id: Option<Uuid>`. Resolve it **without an extra round trip** by adding `u.organization_id` to the existing `get_auth_validation_context` query ([queries/auth/credentials.rs:30-66](iviss-backend/src/queries/auth/credentials.rs#L30-L66)) and its `AuthValidationContext` struct; `require_auth` then builds the extension from claims + context instead of `From<&AccessTokenClaims>`.

**`iviss-backend/src/handlers/vehicles/search.rs`:**
- `search_vehicle` / `search_vehicle_v1` add `Extension(user): Extension<AuthenticatedUser>`.
- `spawn_enqueue_retry(state, org_id, plate)` — retry markers now carry the tenant.
- If the caller has no `organization_id` (platform admin doing a search): **skip the enqueue** with a `tracing::warn!` rather than inventing a partition. The search itself still works; only the retry marker is skipped.
- `record_vehicle_search_control` uses the token's `organization_id` and `user_id` instead of `payload.organization_id` / `payload.agent_id` with their `unwrap_or_else(Uuid::new_v4)` fallbacks. Leave `VehicleSearchRequest` fields in place (ignored) so the OpenAPI contract does not change.

**`iviss-backend/src/handlers/submissions/submissions.rs` — `submit_vehicle`:**
- Add `Extension(user): Extension<AuthenticatedUser>`; use `user.user_id` as the agent.
- Delete the `resolve_agent_id` fallback in [queries/submissions.rs:413-434](iviss-backend/src/queries/submissions.rs#L413-L434) (or reduce it to a strict existence check that errors rather than substituting a stranger).
- Leave `CreatePendingSubmissionRequest.agent_id` in the DTO but ignored, so no OpenAPI regeneration is required. Mark it deprecated in the doc comment.

### 5. Sync worker — `iviss-backend/src/bin/s3-cache-sync.rs`

`drain_queue` lists `(Uuid, plate)` pairs — org is always present since all retry markers are org-partitioned. Before fetching, **group the page by plate**: two orgs queuing the same plate must produce one external API call, not two — then write one `unregistered/{org}/{PLATE}.json` per org on `NotFound`, and one shared `vehicle-cache/` entry on a hit. This keeps the fan-out cost of partitioning at zero on the external API.

Retry-marker deletion after success/NotFound uses `retry_queue_key(org_id, plate)` to reconstruct the exact key.

The worker still needs no DB connection and no org lookup — org comes straight out of the key.

### 6. Handlers — `iviss-backend/src/handlers/submissions/submissions.rs`

All three read handlers add `Extension(admin): Extension<AuthenticatedAdmin>` and derive scope from the role, following [handlers/stats/org.rs:28-37](iviss-backend/src/handlers/stats/org.rs#L28-L37):

```rust
let org_scope: Option<Uuid> = match admin.role.as_str() {
    "admin" => None,                       // platform admin: all tenants
    _ => Some(admin.organization_id
        .ok_or_else(|| AppError::forbidden("Org admin must belong to an organization"))?),
};
```

- `list_pending_submissions` passes `org_scope` to the query, and picks `UnregisteredScope::Organization(id)` vs `AllTenants` for the S3 half.
- `get_pending_submission` / `get_submission_audit_log` pass `org_scope` into the query and return **404, not 403**, when the row belongs to another org — do not confirm existence across tenants.
- `unregistered_to_list_item` unchanged (`PendingSubmissionListItem` keeps its shape → **no frontend client regeneration needed**).

### 7. Queries — `iviss-backend/src/queries/submissions.rs`

`get_pending_submissions`, `get_submission_by_id`, `get_submission_audit_log` each take `org_id: Option<Uuid>`. The `users` join already exists on the first two, so the predicate is one line:

```sql
FROM pending_submissions s
LEFT JOIN users u ON s.agent_id = u.id
WHERE ($1::uuid IS NULL OR u.organization_id = $1)
```

The `LEFT JOIN` must become an **`INNER JOIN` when `org_id` is `Some`** — a `LEFT JOIN` plus a `WHERE` on the right table already achieves this, but the orphan case (submission whose agent row was deleted) must not leak: with the predicate above, a NULL `u.organization_id` fails the comparison and the row drops out of org-scoped results, which is the desired behavior. `get_submission_audit_log` has no join today and needs one added back through `pending_submissions → users`.

Consider adding an index in a migration if the list gets slow: `pending_submissions(agent_id)` — there is currently none, and every scoped list now joins on it. Measure before adding.

### 8. Infra — new `infra/terraform/s3_cache.tf` ⚠️ *DevOps team responsibility — not implemented in this PR*

The following is the specification to forward to the DevOps team:

- `aws_s3_bucket` (or a `data` source if the bucket predates Terraform — **check before applying**, and import rather than recreate), versioning, public-access block, default encryption.
- Two IAM policies, split by service, both scoped to this bucket only:
  - **backend**: `GetObject`/`PutObject` on `vehicle-cache/*`, `PutObject` on `retry-queue/*`, `ListBucket` limited by `s3:prefix` to `unregistered/*`. No `DeleteObject` anywhere.
  - **sync worker**: `ListBucket` + `GetObject` + `DeleteObject` on `retry-queue/*`, `PutObject` on `vehicle-cache/*` and `unregistered/*`. No delete on the cache — this is the invariant the comment at [s3_queue.rs:135](iviss-backend/src/s3_cache_layer/s3_queue.rs#L135) already assumes.
- Split the credentials in `.env.example` / compose so the two services stop sharing one key pair. Dev MinIO keeps the single root credential — no behavior change locally.

## Files touched

**Backend — core**
- `iviss-backend/src/s3_cache_layer/types.rs` — org-aware key builders + parser (+ tests)
- `iviss-backend/src/s3_cache_layer/s3_queue.rs` — org-aware write/list/delete (+ tests)
- `iviss-backend/src/s3_cache_layer/mod.rs` — re-exports
- `iviss-backend/src/services/vehicles/data_cache.rs` — trait signatures, `UnregisteredScope`
- `iviss-backend/src/bin/s3-cache-sync.rs` — drain with org from key, per-plate grouping
- `iviss-backend/src/handlers/submissions/submissions.rs` — auth extension on all four handlers
- `iviss-backend/src/queries/submissions.rs` — `org_id` on three read queries, `resolve_agent_id` removal
- `iviss-backend/src/handlers/vehicles/search.rs` — trusted org for retry marker + control record
- `iviss-backend/src/middleware/auth.rs` — `AuthenticatedUser.organization_id`
- `iviss-backend/src/queries/auth/credentials.rs` — `organization_id` in the existing validation query

**Backend — likely follow-on edits from the signature changes**
- `iviss-backend/src/dto/pending_submission.rs`, `iviss-backend/src/dto/search_vehicle.rs` — doc comments marking the client-supplied `agent_id` / `organization_id` as ignored
- any other implementor/caller of `VehicleDataCache` surfaced by `cargo check`

**Infra / config**
- `infra/terraform/s3_cache.tf` (new), `infra/terraform/outputs.tf`
- `docker-compose.yml`, `iviss-backend/docker-compose.yml`, `.env.example`, `iviss-backend/.env.example`

**Not touched:** `frontend/**` — no DTO shape changes, so the generated client stays valid. If that turns out false during implementation, flag it and regenerate rather than hand-editing.

## Verification

1. `cargo check`, `cargo fmt`, `cargo clippy` (per the project Rust rules — not `cargo test` as a delivery gate, but do run the new unit tests below since they are fast and pure).
2. `cargo test s3_cache_layer` — key builder/parser round-trips, UUID rejection.
3. Local end-to-end with `docker compose --profile dev up`:
   - Clean local MinIO bucket first: `mc rm --recursive --force myminio/iviss-cache/retry-queue/` and `mc rm --recursive --force myminio/iviss-cache/unregistered/` to remove any pre-existing flat keys.
   - Seed two orgs, one org_admin + one agent each.
   - As agent A, `POST /api/v1/vehicles/pending` → row appears for org A's admin, **absent** for org B's admin, present for the platform admin.
   - Hit `GET /api/v1/admin/submissions/{id}` with org B's org_admin token → expect **404**.
   - With `ENABLE_VEHICLE_API=false`, search an unknown plate as an agent of each org → `mc ls` the MinIO bucket and confirm `retry-queue/{orgA}/…` and `retry-queue/{orgB}/…` exist as separate objects.
   - Point the sync worker at the mock external API returning 404 for that plate → confirm `unregistered/{orgA}/PLATE.json` and `unregistered/{orgB}/PLATE.json`, that the retry markers were deleted, and that the external API was called **once** for the shared plate.
4. Infra step (Terraform) — to be verified by the DevOps team separately.

## Residual risks

- Prefix partitioning is enforced in application code, not by IAM (see the caveat above). A bug in key construction silently re-merges tenants; the key-builder tests are the guard.
- Local MinIO may contain residual flat-key objects from development. Run `mc rm --recursive` on `retry-queue/` and `unregistered/` before testing with the new code to avoid stale data.
- There is still no delete path for `unregistered/` markers ([s3_queue.rs:136](iviss-backend/src/s3_cache_layer/s3_queue.rs#L136) hard-asserts retry-queue only), so a plate later registered upstream keeps showing in the admin list. Pre-existing, out of scope, worth a follow-up ticket.
- `approve_submission` / `reject_submission` exist in the query layer but are wired to no route. They also have no org checks. Out of scope here — but they must not be wired up without the same scoping.
