# Graph Report - iviss  (2026-08-10)

## Corpus Check
- 450 files · ~274,033 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 3139 nodes · 6495 edges · 209 communities (193 shown, 16 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 220 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f0c3e93b`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- login
- cn
- control_queries_tests.rs
- get_user_profile
- Button
- storeKey.ts
- services/ocr/photo.rs
- users.rs
- AppError
- UserManagement.tsx
- submissions/mod.rs
- users_and_user_manager_tests.rs
- useAuth
- search.rs
- mockControls.ts
- batch_by_prefix
- PendingVehicles.tsx
- Step 2 — Populate `s3_cache_layer` shared module + simplify `vehicle_data_cache`
- s3_cache_layer/types.rs
- update_organization
- routes.ts
- AuthenticatedAdmin
- sidebar.tsx
- request_refresh
- Config
- stats_queries_tests.rs
- AppCache
- stats_handler_tests.rs
- StageTimings
- list_audit_logs
- vehicle_data_cache_tests.rs
- IVISS Back Office Desktop Dashboard
- sms_provider.rs
- admin_login_and_refresh_token_tests.rs
- plate_format.rs
- button.tsx
- vehicles.rs
- parser.rs
- deviceId.ts
- MobileCarteGrise.tsx
- sms_provider_tests.rs
- Backend CI Report Job
- setup_test_infrastructure
- admin_logout_tests.rs
- Shared backend-environment Anchor
- email_provider.rs
- hcloud_server.control_plane
- extract_image_field
- captureFrame.ts
- src/config.rs
- compilerOptions
- OpenApi
- submissions.rs
- users table
- components.json
- crypto.rs
- Status
- middleware/auth.rs
- use-proactive-refresh.test.ts
- tokenManager.ts
- Docker Rules Skill
- Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time)
- keyManagement.ts
- JwtService (RS256 access token issuance)
- queries/organizations.rs
- var.project_name
- App.tsx
- metricsCollector.ts
- compilerOptions
- jwt.rs
- ocr_service_tests.rs
- VehicleStatusRow
- seed_admin.rs
- .mcp.json
- update_location
- chart.tsx
- ImageProcessor
- logout
- IVISS Platform
- AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret)
- OpenAPI Frontend Codegen (openapi-rq queries/requests)
- change_password
- IVISS Docker Compose Stack
- API Route Groups (public, web-auth, agent-protected, admin, org-admin)
- metrics-server.js
- mockExternalAPIs.ts
- preprocess
- request_daily_login
- ControlRecord Interface
- Prometheus Metrics Server (port 9091)
- control_records table
- compilerOptions
- dto/auth.rs
- IVISS Frontend Architecture
- IVISS Developer Documentation Index
- send_otp HTTP handler (POST /otp)
- SmsProvider trait
- html_builder.rs
- hash_password
- VehicleApiService
- Partner API Integration Flow
- search_vehicle
- Project Graphify Integration Rules
- OtpService
- mockAuth.ts
- authInterceptor.ts
- Plan d'implémentation — Service de synchronisation du cache S3
- Semantic Release Automation
- PendingSubmission
- String
- Install Banner Component
- IVISS Visual Identity: Vehicle-in-Shield on Navy
- test_flow.sh
- Compte-rendu de session — Conception du service de synchronisation S3
- metrics.rs
- ScanResultData
- require_auth_web
- Graphify Knowledge Graph System
- backend_domain_refactor_plan.md
- gh-artifacts.sh
- try_seed
- Compte-rendu de session — Évaluation d'architecture et recadrage du backend par domaine
- S3VehicleDataCache
- fetch_openapi.mjs
- cors_layer
- coverage.sh
- queries/users/location.rs
- IVISS README (English)
- Organization Work Window (start_work_time/end_work_time, UTC+1)
- db.rs
- jwt_service_tests.rs
- setup-remote-state.sh
- main
- authInterceptor.test.ts
- PARTIE 4 — Décisions de conception
- FalkorDB Cypher Export
- init_metrics Custom Metric Registration
- phone_imei Uniqueness Conflict Check
- routes.rs
- Extraction Subagent Prompt
- activate
- Graphify Pipeline
- .fetch
- pre-commit
- engine.rs
- init_db.sh
- health.rs
- S3CacheConfig
- remote_setup.sh
- write_vehicle_data
- errors.rs
- Incremental Update Flow
- Sync Server (Rust/Axum intermediary binary)
- viewfinder.ts
- 2. État des lieux mesuré
- AppState
- useCamera.ts
- VehicleSearchControlRecordInsert
- 6. Arbitrages tranchés
- useCaptureCoaching.test.ts
- PARTIE 1 — Évaluation de l'existant
- PARTIE 2 — Pourquoi ce périmètre plutôt qu'un monolithe modulaire complet
- PARTIE 3 — Arborescence cible
- PARTIE 6 — Plan d'exécution
- useScanPlate.ts
- otp_service_tests.rs
- Admin Terminate Session Endpoint (/api/v1/admin/terminate-session)
- graphify-out/wiki/index.md navigation
- usePWA.ts
- GRAPH_REPORT.md (fallback for broad architecture review)
- graphify query/path/explain commands
- graphify update . (AST-only refresh)
- Conventional Commits + Semantic Release
- usePhotoCapture.ts
- VehicleApiCredentials
- restart_user_session
- EmailProvider
- initialize_pool
- handlers/auth/mod.rs

## God Nodes (most connected - your core abstractions)
1. `AppError` - 196 edges
2. `cn()` - 80 edges
3. `useAuth()` - 43 edges
4. `Button` - 37 edges
5. `setup_test_app()` - 32 edges
6. `Config` - 26 edges
7. `setup_test_app()` - 25 edges
8. `setup_test_infrastructure()` - 25 edges
9. `AppCache` - 24 edges
10. `setup_test_infrastructure()` - 23 edges

## Surprising Connections (you probably didn't know these)
- `IVISS Tech Stack (React/Vite, Rust/Axum, PostgreSQL 15)` --semantically_similar_to--> `Backend Technology Stack Decisions`  [INFERRED] [semantically similar]
  IVISS-README.en.pdf → iviss-backend/docs/stack_documentaion.md
- `No Secrets in Image Layers` --semantically_similar_to--> `Gitleaks Secret Scan Job`  [INFERRED] [semantically similar]
  .claude/skills/docker/SKILL.md → .github/workflows/backend-ci.yml
- `Local Development Setup (.env + compose dev profile)` --conceptually_related_to--> `IVISS Docker Compose Stack`  [INFERRED]
  IVISS-README.en.pdf → docs/docker_setup.md
- `Multi-Tenancy & Security Model` --conceptually_related_to--> `Admin-Only RBAC`  [INFERRED]
  IVISS-README.en.pdf → docs/fe_admin_rbac.md
- `OrangeSmsProvider::normalize_msisdn (tel:+237 normalization)` --semantically_similar_to--> `isValidPlate Validation Utility`  [INFERRED] [semantically similar]
  iviss-backend/docs/message (1).txt → docs/manual_plate_entry.md

## Import Cycles
- 2-file cycle: `iviss-backend/src/external_services/vehicle_client/client.rs -> iviss-backend/src/external_services/vehicle_client/parser.rs -> iviss-backend/src/external_services/vehicle_client/client.rs`

## Hyperedges (group relationships)
- **Graphify Extraction Pipeline** — _claude_skills_graphify_skill_corpus_detection, _claude_skills_graphify_skill_ast_structural_extraction, _claude_skills_graphify_skill_semantic_extraction, _claude_skills_graphify_skill_extraction_cache, _claude_skills_graphify_references_extraction_spec_subagent_prompt, _claude_skills_graphify_skill_community_detection [EXTRACTED 1.00]
- **Rust Backend Quality Gate** — _github_workflows_backend_ci_backend_build, _github_workflows_backend_ci_backend_fmt, _github_workflows_backend_ci_backend_clippy, _github_workflows_backend_ci_backend_audit, _github_workflows_backend_ci_backend_coverage, _claude_rules_rust_rules_testing_and_quality [INFERRED 0.85]
- **Local Docker Compose Dev Stack** — docker_compose_db, docker_compose_backend, docker_compose_frontend, docker_compose_minio, docker_compose_minio_init, docker_compose_adminer, docker_compose_iviss_network [EXTRACTED 1.00]
- **IVISS Agent Authentication Lifecycle (provisioning → bootstrap → activation → daily OTP → refresh → revocation)** — docs_user_registration_admin_controlled_provisioning, docs_user_registration_device_bootstrap, docs_user_registration_agent_activation, docs_user_registration_daily_otp_enforcement, docs_auto_refresh_signature_two_call_refresh_cycle, docs_daily_opertational_login_flow_admin_terminate_session, docs_daily_opertational_login_flow_end_of_shift_revocation [EXTRACTED 1.00]
- **OpenAPI Contract Pipeline (backend annotations → spec export → frontend codegen → CI checks)** — docs_developer_api_contract_first_workflow, docs_developer_api_openapi_codegen, docs_fe_ci_fix_codegen_migration, docs_fe_ci_fix_openapi_sync_workflow, docs_developer_getting_started_predev_openapi_fallback, docs_developer_debugging_openapi_codegen_drift, docs_developer_project_structure_generated_artifacts [INFERRED 0.85]
- **External Registry Availability Mitigation (sync server, S3 cache, versioning, lifecycle, cold start)** — docs_iviss_sync_architecture_sync_server, docs_iviss_sync_architecture_s3_fallback_strategy, docs_iviss_sync_architecture_plate_indexed_json, docs_iviss_sync_architecture_s3_versioning_audit_trail, docs_iviss_sync_architecture_lifecycle_policy, docs_iviss_sync_architecture_cold_start_challenge [EXTRACTED 1.00]
- **Admin-Only RBAC Enforcement Chain (login, guard, sidebar, middleware, bootstrap)** — docs_fe_admin_rbac_admin_login_flow, docs_fe_admin_rbac_requireauth_route_guard, docs_fe_admin_rbac_backoffice_sidebar_gating, docs_fe_admin_rbac_require_auth_middleware, docs_fe_admin_rbac_admin_bootstrap, docs_manual_rbac_testing_backend_enforcement_matrix [EXTRACTED 1.00]
- **SmsProvider Trait Implementations** — iviss_backend_docs_message__1__smsprovider, iviss_backend_docs_message__1__consolesmsprovider, iviss_backend_docs_message__1__snssmsprovider, iviss_backend_docs_message__1__apismsprovider, iviss_backend_docs_message__1__avlytextsmsprovider, iviss_backend_docs_message__1__orangesmsprovider [EXTRACTED 1.00]
- **Metrics-to-Grafana Observability Pipeline** — docs_monitoring_metrics_middleware, docs_monitoring_prometheus_metrics, docs_monitoring_servicemonitor_scrape, monitoring_prometheus_prometheus_frontend_metrics_scrape_job, monitoring_grafana_provisioning_datasources_datasource_prometheus_datasource, monitoring_grafana_provisioning_dashboards_dashboard_dashboard_provider [INFERRED 0.85]
- **PWA / Multi-Platform App Icon Set (favicon, apple-touch, maskable, Safari mask)** — frontend_public_favicon_ivissshieldmark, frontend_public_apple_touch_icon_ivissshieldmark, frontend_public_maskable_icon_512x512_ivissshieldmark, frontend_public_mask_icon_shieldcheckglyph [INFERRED 0.85]
- **Vehicle Control Lifecycle: Field Capture to Back Office Validation** — frontend_public_screenshot_mobile_new_control_capture_modes, frontend_public_screenshot_mobile_gps_location_tracking, frontend_public_screenshot_desktop_pending_validation_workflow, frontend_public_screenshot_desktop_live_control_map, frontend_public_screenshot_desktop_recent_alerts_panel, frontend_public_screenshot_desktop_audit_logs_view [INFERRED 0.85]
- **Real-Time Dashboard Widget Family (Live-Polled Surfaces)** — frontend_public_screenshot_desktop_kpi_stat_card_row, frontend_public_screenshot_desktop_live_control_map, frontend_public_screenshot_desktop_recent_alerts_panel, frontend_public_screenshot_desktop_control_activity_chart, frontend_public_screenshot_desktop_top_agents_today, frontend_public_screenshot_mobile_todays_activity_summary [INFERRED 0.85]
- **PWA Installable Asset Set (Icons + Store Screenshots)** — frontend_public_pwa_64x64_pwa_icon_64, frontend_public_pwa_192x192_pwa_icon_192, frontend_public_pwa_512x512_pwa_icon_512, frontend_public_screenshot_desktop_back_office_dashboard, frontend_public_screenshot_mobile_agent_mobile_dashboard [INFERRED 0.95]

## Communities (209 total, 16 thin omitted)

### Community 0 - "login"
Cohesion: 0.29
Nodes (7): login(), AppState, Arc, IntoResponse, Json, Result, State

### Community 1 - "cn"
Cohesion: 0.04
Nodes (56): BackOfficeHeader(), BackOfficeHeaderProps, BackOfficeLayoutInner(), BackOfficeLayoutProps, BackOfficeSidebar(), NavLink(), MobileHeader(), MobileHeaderProps (+48 more)

### Community 2 - "control_queries_tests.rs"
Cohesion: 0.08
Nodes (66): ControlAction, IdentificationMode, Option, String, SubmissionLocation, ActionType, ControlAction, ControlListQuery (+58 more)

### Community 3 - "get_user_profile"
Cohesion: 0.14
Nodes (12): From, get_user_profile(), AppState, Arc, Extension, IntoResponse, Result, State (+4 more)

### Community 4 - "Button"
Cohesion: 0.09
Nodes (29): ControlActivityChart(), ControlActivityChartProps, agentIcon, DEFAULT_CENTER, leafletIconProto, LiveControlMap(), LiveControlMapProps, BackOfficeLayout() (+21 more)

### Community 5 - "storeKey.ts"
Cohesion: 0.25
Nodes (5): decryptPrivateKey(), encryptPrivateKey(), generateKeyPair(), PasswordManager, storeKeyPair()

### Community 6 - "services/ocr/photo.rs"
Cohesion: 0.13
Nodes (21): color_adaptive_crop(), enhance_photo_result(), estimate_plate_trapezoid(), extract_plate_strict(), fit_edge(), is_orange_plate_pixel(), perspective_rectify_color_crop(), photo_plate() (+13 more)

### Community 7 - "users.rs"
Cohesion: 0.06
Nodes (76): DeviceStatus, ProvisionUserRequest, ProvisionUserResponse, ResendActivationRequest, ResendActivationResponse, ResendOrgAdminPasswordRequest, ResendOrgAdminPasswordResponse, RestartSessionRequest (+68 more)

### Community 8 - "AppError"
Cohesion: 0.15
Nodes (51): UserStatus, AppError, IntoResponse, ActivationUserRow, ActiveDeviceKeyMetadata, AdminAuthRow, AdminRefreshRow, AuthValidationContext (+43 more)

### Community 9 - "UserManagement.tsx"
Cohesion: 0.06
Nodes (63): formSchema, FormValues, minutesToTimeValue(), OrganizationForm(), OrganizationFormProps, timeValueToMinutes(), FormMode, formSchema (+55 more)

### Community 10 - "submissions/mod.rs"
Cohesion: 0.11
Nodes (36): CreatePendingSubmissionRequest, DataEntryResponse, PendingSubmissionDetail, PendingSubmissionListItem, ReviewSubmissionRequest, ReviewSubmissionResponse, Option, Self (+28 more)

### Community 11 - "users_and_user_manager_tests.rs"
Cohesion: 0.13
Nodes (44): create_test_organization(), create_test_user(), generate_test_public_key_base64(), generate_test_rsa_keypair_pem(), hash_otp_code(), issue_admin_token(), Arc, Box (+36 more)

### Community 12 - "useAuth"
Cohesion: 0.10
Nodes (24): StatCard(), StatCardProps, statCardVariants, isValidPlate(), PlateInput(), PlateInputProps, useControls(), useAuth() (+16 more)

### Community 13 - "search.rs"
Cohesion: 0.18
Nodes (19): String, test_validate_plate_format_diplomatic(), test_validate_plate_format_invalid_all_letters(), test_validate_plate_format_invalid_empty(), test_validate_plate_format_invalid_special_chars(), test_validate_plate_format_invalid_too_long(), test_validate_plate_format_invalid_too_short(), test_validate_plate_format_invalid_wrong_pattern() (+11 more)

### Community 14 - "mockControls.ts"
Cohesion: 0.14
Nodes (14): ControlAction, ControlRecord, ControlStats, ControlStatus, mockControls, Translatable, mockVehicles, mockVehicleService (+6 more)

### Community 15 - "batch_by_prefix"
Cohesion: 0.06
Nodes (32): AsRef, Bytes, FromRequestParts, ApiCredentials, AuthError, IntoResponse, Response, Result (+24 more)

### Community 16 - "PendingVehicles.tsx"
Cohesion: 0.06
Nodes (46): Toast, ToastAction, ToastActionElement, ToastClose, ToastDescription, ToastProps, ToastTitle, toastVariants (+38 more)

### Community 17 - "Step 2 — Populate `s3_cache_layer` shared module + simplify `vehicle_data_cache`"
Cohesion: 0.05
Nodes (38): Automated Tests, [DELETE] [vehicle_client_service.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/services/vehicle_client_service.rs), Key Design Decisions (Confirmed), Manual Verification, [MODIFY] [app_cache.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/app_cache.rs), [MODIFY] [app_state.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/app_state.rs), [MODIFY] [bin/s3-cache-sync.rs](file:///home/lonsti-ws/Documents/iviss/iviss-backend/src/bin/s3-cache-sync.rs), [MODIFY] [Cargo.toml](file:///home/lonsti-ws/Documents/iviss/iviss-backend/Cargo.toml) (+30 more)

### Community 18 - "s3_cache_layer/types.rs"
Cohesion: 0.17
Nodes (12): read_vehicle_data(), Client, Option, Result, cache_partition_for_plate(), CachedEntry, CachedVehicleData, object_key() (+4 more)

### Community 19 - "update_organization"
Cohesion: 0.41
Nodes (12): create_organization(), delete_organization(), get_organization(), AppState, Arc, IntoResponse, Json, Path (+4 more)

### Community 20 - "routes.ts"
Cohesion: 0.07
Nodes (33): ProtectedRoute(), ProtectedRouteProps, Activate, AdminLogin, AppRoute, AuditLogPage, BackOfficeDashboard, BackOfficeReports (+25 more)

### Community 21 - "AuthenticatedAdmin"
Cohesion: 0.12
Nodes (56): ActivityData, ActivityFeedItemDto, ActivityFeedQuery, ActivityFeedResponse, ActivityQuery, AgentLocationDto, ControlActivityPoint, ControlActivityResponse (+48 more)

### Community 22 - "sidebar.tsx"
Cohesion: 0.05
Nodes (35): SheetContent, SheetContentProps, SheetDescription, SheetFooter(), SheetHeader(), SheetOverlay, SheetTitle, sheetVariants (+27 more)

### Community 23 - "request_refresh"
Cohesion: 0.35
Nodes (13): request_refresh(), request_refresh_admin(), request_refresh_agent(), AppState, Arc, IntoResponse, Json, Response (+5 more)

### Community 24 - "Config"
Cohesion: 0.19
Nodes (9): Config, Environment, Option, Result, Self, String, Vec, EmailProviderCredentials (+1 more)

### Community 25 - "stats_queries_tests.rs"
Cohesion: 0.23
Nodes (32): ContainerAsync, PgPool, Postgres, Uuid, seed_agent(), seed_agent_location(), seed_control_record(), seed_organization() (+24 more)

### Community 26 - "AppCache"
Cohesion: 0.05
Nodes (68): Default, Expiry, AppCache, OtpEntry, OtpExpiry, Cache, Duration, Instant (+60 more)

### Community 27 - "stats_handler_tests.rs"
Cohesion: 0.15
Nodes (35): contract_rbac_route_role_matrix(), get(), RbacCase, Option, Router, StatusCode, generate_test_rsa_keypair_pem(), Arc (+27 more)

### Community 28 - "StageTimings"
Cohesion: 0.16
Nodes (11): is_budget_exceeded(), OcrBudget, OcrBudgetExceeded, Duration, FnOnce, Instant, Result, Self (+3 more)

### Community 29 - "list_audit_logs"
Cohesion: 0.11
Nodes (25): AuditAction, AuditLogEntry, AuditLogQuery, Err, FromStr, Option, Result, Self (+17 more)

### Community 30 - "vehicle_data_cache_tests.rs"
Cohesion: 0.32
Nodes (10): get_missing_plate_returns_none(), make_test_vehicle(), ContainerAsync, FnOnce, start_minio_cache(), store_and_get_no_encryption(), store_and_get_option_e_client_layer_verified(), store_and_get_with_client_side_encryption() (+2 more)

### Community 31 - "IVISS Back Office Desktop Dashboard"
Cohesion: 0.12
Nodes (25): IVISS PWA App Icon 192x192, IVISS Brand Identity (Shield + Vehicle + Licence Plate), Navy / Teal Law-Enforcement Colour Palette, IVISS PWA App Icon 512x512, Multi-Resolution PWA Icon Set, IVISS PWA Favicon 64x64, Audit Logs View, IVISS Back Office Desktop Dashboard (+17 more)

### Community 32 - "sms_provider.rs"
Cohesion: 0.15
Nodes (16): MockSmsProvider, OrangeSmsProvider, OrangeTokenResponse, Arc, Cache, Client, Instant, Result (+8 more)

### Community 33 - "admin_login_and_refresh_token_tests.rs"
Cohesion: 0.07
Nodes (49): init_metrics(), init_telemetry(), init_tracer_provider(), Arc, Option, Result, Self, String (+41 more)

### Community 34 - "plate_format.rs"
Cohesion: 0.14
Nodes (16): classify(), classify_compact(), correct_digit(), correct_letter(), correct_with_mask(), extract_first(), find_candidate(), format_display() (+8 more)

### Community 35 - "button.tsx"
Cohesion: 0.06
Nodes (25): MobileLayout(), VehicleActionFooterProps, VehicleHeader(), VehicleHeaderProps, VehicleImageCollapsible(), VehicleImageCollapsibleProps, VehicleNotFound(), VehicleNotFoundProps (+17 more)

### Community 36 - "vehicles.rs"
Cohesion: 0.12
Nodes (9): Option, String, VehicleRow, create_test_status_row(), create_test_vehicle_row(), test_vehicle_row_debug_format(), test_vehicle_row_structure(), test_vehicle_status_debug_format() (+1 more)

### Community 37 - "parser.rs"
Cohesion: 0.23
Nodes (12): HashMap, clean_value(), decode_basic_html_entities(), html_to_text(), make_service(), parse_inline_customs_status(), parse_label_value_lines(), Option (+4 more)

### Community 38 - "deviceId.ts"
Cohesion: 0.15
Nodes (9): getDeviceId(), resetDeviceId(), clearAllStoredData(), MyDatabase, SimpleStorage, storage, dummyDB, mockIDBOpenDBRequest (+1 more)

### Community 39 - "MobileCarteGrise.tsx"
Cohesion: 0.13
Nodes (13): Textarea, TextareaProps, mockGeolocation, useUser(), useUsers(), GeoLocation, getBrowserLocation(), reverseGeocode() (+5 more)

### Community 40 - "sms_provider_tests.rs"
Cohesion: 0.18
Nodes (14): Result, test_twilio_authentication_headers(), test_twilio_form_parameters(), test_twilio_sms_provider_authentication_error(), test_twilio_sms_provider_empty_message(), test_twilio_sms_provider_invalid_phone_error(), test_twilio_sms_provider_network_timeout(), test_twilio_sms_provider_rate_limit_error() (+6 more)

### Community 41 - "Backend CI Report Job"
Cohesion: 0.13
Nodes (20): General Coding Guidelines, Modular Design Principle, Preserve Repository Architecture, Security-First Approach, Test Coverage Requirement, Rust Testing and Quality Gate, graph.json Shrink Guard, Backend Security Audit Job (+12 more)

### Community 42 - "setup_test_infrastructure"
Cohesion: 0.25
Nodes (16): ec_public_key_to_b64_jwk(), generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, Router, String, Uuid (+8 more)

### Community 43 - "admin_logout_tests.rs"
Cohesion: 0.23
Nodes (21): create_admin_user(), create_refresh_token(), generate_access_token(), generate_test_rsa_keypair_pem(), hash_password(), ContainerAsync, Option, PgPool (+13 more)

### Community 44 - "Shared backend-environment Anchor"
Cohesion: 0.12
Nodes (19): Image Version Pinning, SHA-Pinned GitHub Actions, Build & Push Docker Images Job, GHCR Image Registry, Determine Release Version Job, Semantic Release Action, Dual-Key OIDC Trust Hardening, PEM Key and Secret Parsing Hardening (+11 more)

### Community 45 - "email_provider.rs"
Cohesion: 0.19
Nodes (10): AsyncSmtpTransport, LettreEmailProvider, MockEmailProvider, ResendEmailProvider, Arc, Client, Result, Self (+2 more)

### Community 46 - "hcloud_server.control_plane"
Cohesion: 0.16
Nodes (23): hcloud_firewall.k3s, hcloud_server.control_plane, hcloud_server.worker, hcloud_ssh_key.k3s, output.cluster_name, output.control_plane_ips, output.kubeconfig, output.private_key_openssh (+15 more)

### Community 47 - "extract_image_field"
Cohesion: 0.27
Nodes (13): error_response(), error_response_tuple(), extract_image_field(), IntoResponse, Json, Multipart, Option, Result (+5 more)

### Community 48 - "captureFrame.ts"
Cohesion: 0.18
Nodes (10): bitmapToDataUrl(), blobToDataUrl(), captureFrame(), getImageCaptureCtor(), ImageCapture, ImageCaptureConstructor, Window, withTimeout() (+2 more)

### Community 49 - "src/config.rs"
Cohesion: 0.16
Nodes (10): LogLevel, mock_vehicle_api_credentials(), FromStr, test_config_helpers(), test_parse_allowed_origins_accepts_explicit_origin_list(), test_parse_allowed_origins_rejects_path_or_trailing_slash(), test_parse_allowed_origins_rejects_wildcard(), test_s3_cache_requires_bucket_when_enabled() (+2 more)

### Community 50 - "compilerOptions"
Cohesion: 0.06
Nodes (34): compilerOptions, allowImportingTsExtensions, baseUrl, isolatedModules, jsx, lib, module, moduleDetection (+26 more)

### Community 51 - "OpenApi"
Cohesion: 0.17
Nodes (4): ApiDoc, SecurityAddon, Modify, OpenApi

### Community 52 - "submissions.rs"
Cohesion: 0.23
Nodes (18): approve_submission(), create_pending_submission(), get_pending_submissions(), get_submission_audit_log(), get_submission_by_id(), reject_submission(), resolve_agent_id(), Option (+10 more)

### Community 53 - "users table"
Cohesion: 0.11
Nodes (21): Environment-Seeded Admin Bootstrap, Back-Office Sidebar Admin Nav Gating, Role Matrix (admin / agent / supervisor), Admin Bootstrap Idempotency Verification, PENDING_ACTIVATION Manual Password Workaround, Pre-Seeded Test Users (seed_data.sql), Gray-Card Submission Workflow, access_token_blacklist table (+13 more)

### Community 54 - "components.json"
Cohesion: 0.12
Nodes (16): aliases, components, hooks, lib, ui, utils, rsc, $schema (+8 more)

### Community 55 - "crypto.rs"
Cohesion: 0.50
Nodes (7): decrypt(), decrypt_rejects_short_payload(), decrypt_with_wrong_key_fails(), encrypt(), encrypt_decrypt_round_trip(), Result, Vec

### Community 56 - "Status"
Cohesion: 0.42
Nodes (13): Status, CustomsStatus, InsuranceStatus, OwnerInfo, PoliceStatus, Option, String, Vec (+5 more)

### Community 57 - "middleware/auth.rs"
Cohesion: 0.17
Nodes (18): HeaderValue, decode_access_token_rs256(), extract_bearer_token(), extracts_bearer_token(), is_user_status_allowed(), rejects_non_bearer_header(), require_auth(), AppState (+10 more)

### Community 59 - "tokenManager.ts"
Cohesion: 0.16
Nodes (19): applyAuthTokenToApiClient(), AuthProvider(), humanizeActivationError(), requiresDeviceReactivation(), AuthContext, AuthContextType, defaultFocusSetup(), useProactiveRefresh() (+11 more)

### Community 60 - "Docker Rules Skill"
Cohesion: 0.21
Nodes (15): Custom Bridge Network, Docker Rules Skill, HEALTHCHECK on All Services, Named Volumes for Persistence, Non-Root Container User, No Secrets in Image Layers, Log to stdout/stderr Only, adminer Service (+7 more)

### Community 61 - "Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time)"
Cohesion: 0.14
Nodes (15): Activation vs Daily Login Distinction, Testing Guide: Admin Session Management & Authentication, Browser-Persisted Device ID (iviss_device_id in localStorage), BE-08 Daily Login Flow (Agent), Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time), Redis-to-Moka In-Process Cache Migration, Backend Startup Sequence (config → pool → migrations → bootstrap seed → dev seed → AppCache), IVISS Database Development Guide (+7 more)

### Community 62 - "keyManagement.ts"
Cohesion: 0.29
Nodes (6): AppInitializer(), AppInitializerProps, tryInitializeKeys(), mockedKeyManagement, checkKeyPairExists(), KeyManagement()

### Community 63 - "JwtService (RS256 access token issuance)"
Cohesion: 0.14
Nodes (15): Testing Guide: Admin Session Termination & Auth Fixes, Forced Logout with EventKeyStorage IndexedDB Wipe, JWT RSA Key Pair Generation and .env Setup, Access & Refresh Tokens (Backend), AccessTokenClaims (sub, device_id, role, exp, jti), AuthUser Extractor (FromRequestParts bearer verification), JwtService (RS256 access token issuance), Stateless Access Token (never stored server-side) (+7 more)

### Community 64 - "queries/organizations.rs"
Cohesion: 0.24
Nodes (22): CreateOrganizationRequest, Organization, OrganizationDetails, OrganizationType, Option, String, Uuid, UpdateOrganizationRequest (+14 more)

### Community 65 - "var.project_name"
Cohesion: 0.18
Nodes (15): aws_iam_openid_connect_provider.github_actions, aws_iam_role.github_actions_deploy, aws_iam_role_policy.deploy_permissions, aws_secretsmanager_secret.app_secrets, aws_secretsmanager_secret.provider_keys, aws_secretsmanager_secret.vehicle_api_keys, aws_secretsmanager_secret_version.app_secrets, aws_secretsmanager_secret_version.provider_keys (+7 more)

### Community 66 - "App.tsx"
Cohesion: 0.11
Nodes (11): App(), queryClient, ErrorFallback(), ErrorBoundary, Props, State, BeforeInstallPromptEvent, PWAInstallPrompt() (+3 more)

### Community 67 - "metricsCollector.ts"
Cohesion: 0.31
Nodes (12): AppInner(), useMetrics(), collectPageLoadDuration(), destroyMetrics(), generateSessionId(), initMetrics(), observeCLS(), observeFID() (+4 more)

### Community 68 - "compilerOptions"
Cohesion: 0.11
Nodes (17): compilerOptions, allowImportingTsExtensions, isolatedModules, lib, module, moduleDetection, moduleResolution, noEmit (+9 more)

### Community 69 - "jwt.rs"
Cohesion: 0.29
Nodes (12): EncodingKey, AccessTokenClaims, generate_test_keys(), JwtService, Result, Self, String, UserRole (+4 more)

### Community 70 - "ocr_service_tests.rs"
Cohesion: 0.11
Nodes (13): pick_best_ensemble(), Vec, sauvola_threshold(), dark_on_light(), deskew_on_a_straight_binary_plate_picks_zero(), deskew_output_is_strictly_bilevel(), finalize_never_rewrites_confidence(), pick_best_ensemble_falls_back_to_textual_candidates_when_nothing_has_a_plate() (+5 more)

### Community 71 - "VehicleStatusRow"
Cohesion: 0.20
Nodes (8): Date, pending_insurance_status(), pending_technical_status(), VehicleStatusRow, api_status_results_preserve_pending_partner_placeholders(), Option, VehicleInfo, VehicleService

### Community 72 - "seed_admin.rs"
Cohesion: 0.30
Nodes (13): BootstrapResult, create_test_config(), ContainerAsync, PgPool, Postgres, Result, String, run_bootstrap_seed() (+5 more)

### Community 73 - ".mcp.json"
Cohesion: 0.18
Nodes (13): DATABASE_URI, npx, uvx, context7, filesystem, git, postgres, sequential-thinking (+5 more)

### Community 74 - "update_location"
Cohesion: 0.15
Nodes (11): String, UpdateLocationRequest, UpdateLocationResponse, AppState, Arc, Extension, IntoResponse, Json (+3 more)

### Community 75 - "chart.tsx"
Cohesion: 0.18
Nodes (7): ChartConfig, ChartContainer, ChartContext, ChartContextProps, ChartLegendContent, ChartTooltipContent, THEMES

### Community 76 - "ImageProcessor"
Cohesion: 0.29
Nodes (4): usePhotoCapture(), ImageProcessor, PLATE_PATTERNS, ViewfinderBox

### Community 77 - "logout"
Cohesion: 0.25
Nodes (8): logout(), AppState, Arc, Body, IntoResponse, Request, Result, State

### Community 78 - "IVISS Platform"
Cohesion: 0.17
Nodes (13): Robust Error Handling and Logging, Concurrency and Async Discipline, Rust Error Handling (thiserror/anyhow), Ownership and Type Modeling, Rust General Rules, Backend Build Job, metrics Service, Backend Architecture (Rust/Axum/SQLx) (+5 more)

### Community 79 - "AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret)"
Cohesion: 0.15
Nodes (13): Mock SMS Provider (OTP retrieved from backend logs), AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret), Temporary Debug SSH Profile (port 22 open to 0.0.0.0/0), infra/scripts/deploy.sh Deployment Pipeline, Cost-Aware Edge-and-Origin Architecture (CloudFront → Lightsail), Edge Lockdown (CloudFront CIDR restriction, default enabled), IVISS Master Deployment & Infrastructure Guide (v3.3), SMS/Email Provider Configuration (mock, twilio, vonage, orange, resend, lettre) (+5 more)

### Community 80 - "OpenAPI Frontend Codegen (openapi-rq queries/requests)"
Cohesion: 0.17
Nodes (13): API Gateway (JWT + Rate Limit + CORS), IVISS WebService (Rust + Axum + Tokio), Endpoint Change Order (DTO → handler → query → route → api_doc → tests → codegen), OpenAPI Frontend Codegen (openapi-rq queries/requests), Generated Artifacts Must Not Be Hand-Edited, OpenAPI and Codegen Drift Symptom/Fix, predev OpenAPI Fetch with Local Snapshot Fallback, Backend Layering (main/routes/handlers/services/queries/dto/models/middleware/db) (+5 more)

### Community 81 - "change_password"
Cohesion: 0.25
Nodes (8): change_password(), AppState, Arc, Extension, IntoResponse, Json, Result, State

### Community 82 - "IVISS Docker Compose Stack"
Cohesion: 0.14
Nodes (14): Development Hot-Reload Services, IVISS Docker Compose Stack, Admin Login Flow (email + password), Admin-Only RBAC, require_auth JWT Middleware, RequireAuth Route Guard (allowedRoles), Backend RBAC Enforcement Matrix (401/403/200), Manual RBAC Test Plan (+6 more)

### Community 83 - "API Route Groups (public, web-auth, agent-protected, admin, org-admin)"
Cohesion: 0.14
Nodes (15): IVISS Technical Architecture & System Design, Multi-Tenant Organization Hierarchy (Super Admin → Org → Admin → Supervisor → Agent), RBAC Role Definitions (super_admin, admin, supervisor, agent), Auto-Refresh Signature Mechanism, Two-Call Refresh Cycle (/auth/refresh + /auth/refresh/verify), User Interface (frontend), IVISS API Development Guide, Backend-First API Contract Workflow (+7 more)

### Community 84 - "metrics-server.js"
Cohesion: 0.15
Nodes (11): activeSessions, app, clsGauge, errorsTotal, fidGauge, frontendUp, lcpGauge, pageLoadDuration (+3 more)

### Community 85 - "mockExternalAPIs.ts"
Cohesion: 0.15
Nodes (10): AggregatedVehicleStatus, APIResponse, APIStatus, CustomsResult, InsuranceResult, mockAPIData, mockExternalAPIService, PoliceResult (+2 more)

### Community 86 - "preprocess"
Cohesion: 0.23
Nodes (15): add_border(), contrast_stretch_percentile(), deskew(), estimate_skew_angle(), invert_image(), is_light_on_dark(), morphology_open(), preprocess() (+7 more)

### Community 87 - "request_daily_login"
Cohesion: 0.39
Nodes (8): request_daily_login(), AppState, Arc, IntoResponse, Json, Result, State, verify_daily_login()

### Community 88 - "ControlRecord Interface"
Cohesion: 0.18
Nodes (12): Partner Compliance APIs (Insurance, Customs, Inspection, Wanted), ControlRecord Interface, VehicleStatus Enum (valid/warning/critical/pending), Core Database Tables and Domains, Query Layer Modules (auth/user/organization/vehicle/control/submission/stats/audit/location/session), Mock External Providers in Tests (no real SMS/email/OCR calls), Control Logging (frontend feature), Mock API Services Layer (mockAuth, mockVehicles, mockControls, mockExternalAPIs) (+4 more)

### Community 89 - "Prometheus Metrics Server (port 9091)"
Cohesion: 0.18
Nodes (12): OpenTelemetry Distributed Tracing (OTLP/HTTP to Alloy), Metrics Port Isolation from Public Ingress, IVISS Observability Stack, Prometheus Metrics Server (port 9091), kube-prometheus-stack ServiceMonitor Scrape Config, Layered Structured Logging (fmt + OTel), TelemetryHandle::shutdown Graceful Flush, Traces-Logs-Metrics Correlation in Grafana (+4 more)

### Community 90 - "control_records table"
Cohesion: 0.17
Nodes (12): Vehicle Check Workflow, Offline Cache Fallback Behaviour, Silent Service Worker Auto-Update, control_actions table, control_records table, Authority & Precision Color Palette, Micro-Gestalt Interactive Mechanics, Premium Layer Design Tokens (glassmorphism, gradients, shadows) (+4 more)

### Community 91 - "compilerOptions"
Cohesion: 0.17
Nodes (11): compilerOptions, allowJs, baseUrl, noImplicitAny, noUnusedLocals, noUnusedParameters, paths, skipLibCheck (+3 more)

### Community 92 - "dto/auth.rs"
Cohesion: 0.23
Nodes (18): ActivateRequest, ActivateResponse, AuthResponse, ChangePasswordRequest, ChangePasswordResponse, LoginRequest, LogoutRequestHeaders, RefreshChallengeResponse (+10 more)

### Community 93 - "IVISS Frontend Architecture"
Cohesion: 0.20
Nodes (10): Identification Modes (manual/photo/live), Frontend Layering (main/App/router/pages/components/hooks/services/openapi-rq/i18n), Frontend Test Helpers (setup.ts, createQueryWrapper, MemoryRouter), AuthProvider (React Auth Context), IVISS Design System (CSS Variables, Navy/Teal, status colors), IVISS Frontend Architecture, License Plate OCR Pipeline (react-webcam + ImageProcessor + Tesseract.js), ProtectedRoute / RequireAuth Guard (+2 more)

### Community 94 - "IVISS Developer Documentation Index"
Cohesion: 0.22
Nodes (10): Deployment Modes A/B/C (local .env, Secrets Manager, CI/CD OIDC), GitHub Actions OIDC Assume Role (no static AWS keys), IVISS Coding Standards, IVISS Debugging Guide, IVISS Developer Getting Started, IVISS Project Structure, IVISS Developer Documentation Index, CI Pipeline Jobs (backend coverage/clippy/audit, frontend codegen/lint/build/Sonar) (+2 more)

### Community 95 - "send_otp HTTP handler (POST /otp)"
Cohesion: 0.27
Nodes (10): Partner Timeout / Unavailable Graceful Degradation, is_permanent_error, is_transient_error, NotificationJob (Otp / MagicEmail), process_notification_job, send_otp_with_retry (exponential backoff), Transient vs Permanent SMS Error Taxonomy, Permanent-to-400 / Transient-to-502 Status Mapping (+2 more)

### Community 96 - "SmsProvider trait"
Cohesion: 0.33
Nodes (10): ApiSmsProvider (generic third-party API), AvlytextSmsProvider, ConsoleSmsProvider, OrangeSmsProvider, SmsProvider trait, SnsSmsProvider (AWS SNS), create_sms_provider factory, GatewayState (Arc<dyn SmsProvider>) (+2 more)

### Community 97 - "html_builder.rs"
Cohesion: 0.32
Nodes (14): build_found_response(), build_html(), build_not_found_response(), cleared_banner_present(), current_timestamp(), immat_always_present(), labeled_field(), mark_and_type_uses_html_entity() (+6 more)

### Community 98 - "hash_password"
Cohesion: 0.38
Nodes (9): hash_password(), Result, String, test_hash_is_different_each_time(), test_hash_password_generates_valid_hash(), test_verify_password_correct_password(), test_verify_password_malformed_hash(), test_verify_password_wrong_password() (+1 more)

### Community 99 - "VehicleApiService"
Cohesion: 0.18
Nodes (8): Client, Result, Self, Vec, VehicleApiService, is_vehicle_not_found_response(), Error, VehicleApiError

### Community 100 - "Partner API Integration Flow"
Cohesion: 0.13
Nodes (17): Cameroon License Plate Formats, isValidPlate Validation Utility, PlateInput Flexible Auto-Formatting, vehicle_owners table, vehicle_statuses table, vehicles table, Agent Login and Vehicle Lookup Flow, API Gateway (JWT extraction and claim forwarding) (+9 more)

### Community 101 - "search_vehicle"
Cohesion: 0.28
Nodes (13): Uuid, VehicleSearchRequest, log_search_location(), record_vehicle_search_control(), AppState, Arc, IntoResponse, Json (+5 more)

### Community 102 - "Project Graphify Integration Rules"
Cohesion: 0.28
Nodes (9): Deterministic Node ID Format, Native CLAUDE.md Integration, BFS Traversal Mode, Explain Node Flow, Inline NetworkX Traversal Fallback, Constrained Query Expansion, Shortest Path Between Concepts, Graph Health Check (+1 more)

### Community 103 - "OtpService"
Cohesion: 0.32
Nodes (6): OtpService, Arc, Result, Self, String, Uuid

### Community 104 - "mockAuth.ts"
Cohesion: 0.25
Nodes (5): AuthSession, mockAuthService, mockUsers, User, UserRole

### Community 105 - "authInterceptor.ts"
Cohesion: 0.21
Nodes (16): AuthErrorCategory, classifyAuthError(), classifyAuthErrorMessage(), extractErrorCode(), extractErrorMessage(), HeyApiClient, isAdminSession(), isDeviceReactivationMessage() (+8 more)

### Community 106 - "Plan d'implémentation — Service de synchronisation du cache S3"
Cohesion: 0.17
Nodes (12): Architecture cible, Contexte, Décisions actées, Plan d'implémentation — Service de synchronisation du cache S3, Points signalés, hors périmètre, Vérification, Étape 0 — Réparer `services/vehicle_data_cache.rs`, Étape 1 — Étendre `s3_cache_layer` aux nouveaux préfixes (+4 more)

### Community 107 - "Semantic Release Automation"
Cohesion: 0.25
Nodes (8): GHCR Image Publishing via GitHub Actions, Prod Compose Profile (backend-prod / frontend-prod), Conventional Commit Prefixes, dev-Branch Release Trigger, Semantic Release Automation, Semantic Versioning (MAJOR.MINOR.PATCH), Version Tag Reset Procedure, Terraform + Ansible + GitHub Actions Deployment to AWS Lightsail

### Community 108 - "PendingSubmission"
Cohesion: 0.50
Nodes (7): PendingSubmission, OffsetDateTime, Option, String, Uuid, Value, SubmissionAuditLogRow

### Community 109 - "String"
Cohesion: 0.35
Nodes (4): Into, Error, Self, String

### Community 110 - "Install Banner Component"
Cohesion: 0.29
Nodes (7): Install Banner Component, No localStorage Persistence for Banner Dismissal, PWA Installation Testing Guide, display-mode: standalone Installed Detection, PWA Meta Tags and Icons (index.html), SPA Root Mount (#root + /src/main.tsx), Crawler Allow-All Policy

### Community 111 - "IVISS Visual Identity: Vehicle-in-Shield on Navy"
Cohesion: 0.43
Nodes (7): IVISS Shield Mark (Apple Touch Icon), IVISS Shield Mark (Browser Favicon), Monochrome Shield-with-Checkmark Glyph (Safari Mask Icon), IVISS Shield Mark (Maskable PWA Icon 512x512), IVISS Visual Identity: Vehicle-in-Shield on Navy, Maskable Icon Safe-Zone Padding, Neutral Missing-Image Placeholder Graphic

### Community 112 - "test_flow.sh"
Cohesion: 0.67
Nodes (5): ensure_cargo_llvm_cov(), print_error(), print_status(), print_warning(), test_flow.sh script

### Community 113 - "Compte-rendu de session — Conception du service de synchronisation S3"
Cohesion: 0.17
Nodes (12): 1. Demande initiale, 2. État des lieux du code existant, 3.1 Sonde de santé — la question la plus longue à trancher, 3.2 L'ambiguïté « Service indisponible », 3.3 Choix de la plaque sentinelle, 3.4 Autres décisions, 3. Décisions et raisonnement, 4. Points signalés, non traités (+4 more)

### Community 114 - "metrics.rs"
Cohesion: 0.60
Nodes (5): Body, Next, Request, Response, track_metrics()

### Community 115 - "ScanResultData"
Cohesion: 0.15
Nodes (22): Eq, ImageUploadRequest, Option, Self, String, ScanErrorData, ScanPlateResponse, ScanResultData (+14 more)

### Community 116 - "require_auth_web"
Cohesion: 0.35
Nodes (11): require_admin(), require_auth_web(), require_org_admin(), AppState, Arc, Next, Request, Response (+3 more)

### Community 118 - "backend_domain_refactor_plan.md"
Cohesion: 0.22
Nodes (6): Context, Décisions actées, PARTIE 5 — Impact sur le plan S3, PARTIE 7 — Points signalés, hors périmètre, Plan de refactoring — Recadrage du backend par domaine, Vérification

### Community 119 - "gh-artifacts.sh"
Cohesion: 0.70
Nodes (4): delete_all_artifacts(), delete_old_artifacts(), list_artifacts(), gh-artifacts.sh script

### Community 120 - "try_seed"
Cohesion: 0.70
Nodes (4): PgPool, Result, run_seed_data(), try_seed()

### Community 121 - "Compte-rendu de session — Évaluation d'architecture et recadrage du backend par domaine"
Cohesion: 0.20
Nodes (10): 1. Demande initiale, 3. Recherche — état de l'art du monolithe modulaire, 4. Première proposition (monolithe modulaire complet) — écartée, 5.1 Pourquoi c'est le bon appel, 5.2 Coût comparé, 5.3 Le levier de réduction de risque — l'astuce des re-exports, 5. Contre-proposition retenue — package-by-layer-then-feature, 7. Points signalés, hors périmètre (+2 more)

### Community 122 - "S3VehicleDataCache"
Cohesion: 0.27
Nodes (6): Client, Option, Result, Self, String, S3VehicleDataCache

### Community 123 - "fetch_openapi.mjs"
Cohesion: 0.60
Nodes (4): __dirname, fallbackToLocal(), fetchFromBackend(), main()

### Community 124 - "cors_layer"
Cohesion: 0.50
Nodes (3): CorsLayer, cors_layer(), String

### Community 125 - "coverage.sh"
Cohesion: 0.50
Nodes (3): LLVM_PROFILE_FILE, RUSTFLAGS, coverage.sh script

### Community 126 - "queries/users/location.rs"
Cohesion: 0.36
Nodes (13): create_test_user(), ContainerAsync, PgPool, Postgres, Result, Uuid, setup_test_db(), test_update_agent_location_handles_boundary_coordinates() (+5 more)

### Community 127 - "IVISS README (English)"
Cohesion: 0.15
Nodes (16): IVISS Platform Overview, Multi-Tenant Data Isolation by Organization, Retention and Archival Policy, IVISS PostgreSQL Schema, organizations table, Soft-Delete via deleted_at, SMS Gateway Service (binary), Axum Web Framework (+8 more)

### Community 128 - "Organization Work Window (start_work_time/end_work_time, UTC+1)"
Cohesion: 0.67
Nodes (3): Organization Work Window (start_work_time/end_work_time, UTC+1), Shift Bounds Embedded in Access Token and devices.metadata, Org-Level Shift Windows Supersede SHIFT_START_HOUR/SHIFT_END_HOUR

### Community 129 - "db.rs"
Cohesion: 0.38
Nodes (9): find_by_plate(), find_by_prefix(), init(), Option, PgPool, Result, String, Vec (+1 more)

### Community 130 - "jwt_service_tests.rs"
Cohesion: 0.56
Nodes (8): decode_access_claims(), make_jwt_service(), test_access_token_contains_correct_sub_and_device_id(), test_access_token_contains_shift_bounds(), test_access_token_has_unique_jti(), test_access_token_role_is_correct(), test_access_token_with_shift_custom_bounds(), test_access_token_with_shift_different_roles()

### Community 133 - "authInterceptor.test.ts"
Cohesion: 0.17
Nodes (7): signNonce(), mockedGetDeviceId, mockedRequestRefresh, mockedSignNonce, mockedVerifyRefresh, retrieveKeyPair(), mockedRetrieveKeyPair

### Community 134 - "PARTIE 4 — Décisions de conception"
Cohesion: 0.29
Nodes (7): 4.1 ★ Éclatement de `routes.rs` — la sécurité RBAC préservée par construction, 4.2 ★★ `external_services/` : trait `ExternalDataSource` + périmètre réel restreint, 4.3 `services/auth/` ne doit PAS contenir email et SMS, 4.4 Ne pas déplacer `s3_cache_layer/`, 4.5 Différer l'éclatement de `api_doc.rs` — assumé, 4.6 Ce qu'on ne fait **pas** cette itération, PARTIE 4 — Décisions de conception

### Community 138 - "routes.rs"
Cohesion: 0.67
Nodes (5): assembly(), metrics_router(), AppState, Arc, Router

### Community 139 - "Extraction Subagent Prompt"
Cohesion: 0.15
Nodes (14): Discrete Confidence Score Rubric, Hyperedge Extraction Rule, Extraction Subagent Prompt, EXTRACTED/INFERRED/AMBIGUOUS Audit Trail, Cumulative Token Cost Tracker, Semantic Extraction Cache, Graphify Honesty Rules, No API Key Required Policy (+6 more)

### Community 140 - "activate"
Cohesion: 0.25
Nodes (7): activate(), AppState, Arc, IntoResponse, Json, Result, State

### Community 141 - "Graphify Pipeline"
Cohesion: 0.21
Nodes (13): Graphify Skill Trigger, Folder Watch Auto-Rebuild, Graphify MCP Stdio Server, Token Reduction Benchmark, Cross-Repo Graph Merge, GitHub Repo Clone, Post-Commit Auto-Rebuild Hook, Code-Only Change Shortcut (+5 more)

### Community 142 - ".fetch"
Cohesion: 0.24
Nodes (9): ExternalDataSource, ExternalServiceError, HealthStatus, PartnerPayload, Option, Send, String, Sync (+1 more)

### Community 145 - "engine.rs"
Cohesion: 0.12
Nodes (30): Deref, DerefMut, Drop, acquire_ocr_permit(), configure_tesseract(), decode_image(), encode_bmp(), extract_plate_fuzzy() (+22 more)

### Community 147 - "health.rs"
Cohesion: 0.52
Nodes (6): health_check(), metrics_export(), AppState, Arc, IntoResponse, State

### Community 148 - "S3CacheConfig"
Cohesion: 0.38
Nodes (6): build_s3_client(), Client, Option, Result, String, S3CacheConfig

### Community 150 - "write_vehicle_data"
Cohesion: 0.21
Nodes (10): load_s3_cache_config(), load_vehicle_api_credentials(), main(), Result, Client, Option, Result, String (+2 more)

### Community 169 - "errors.rs"
Cohesion: 0.29
Nodes (12): AppErrorResponse, ErrorCode, get_body_json(), Response, Value, test_bad_request_response(), test_database_error_response(), test_external_api_failure_response() (+4 more)

### Community 170 - "Incremental Update Flow"
Cohesion: 0.18
Nodes (12): URL Ingestion into Corpus, Agent-Crawlable Wiki Export, Verbatim source_file Rule, save-result Work Memory Loop, Whisper Domain Hint Prompt, Whisper Video/Audio Transcription, build_merge Replace-on-Re-extract, Cluster-Only Rebuild (+4 more)

### Community 171 - "Sync Server (Rust/Axum intermediary binary)"
Cohesion: 0.25
Nodes (8): Data Schema & Constants, Vehicle Interface, Cold Start Challenge (plate never cached in S3), Plate-Indexed JSON Object Model (vehicles/{PLATE}.json), Responsibility Boundary (IVISS App never touches External DB or S3), S3 Fallback Cache Strategy, Sync Server (Rust/Axum intermediary binary), IVISS Sync Server Architecture

### Community 172 - "viewfinder.ts"
Cohesion: 0.17
Nodes (6): ScanViewfinder(), ScanViewfinderProps, expectedCropOutput(), mockT, computeViewfinderCrop(), ViewfinderCrop

### Community 173 - "2. État des lieux mesuré"
Cohesion: 0.40
Nodes (5): 2.1 Structure réelle, 2.2 Ce qui est bon, 2.3 Ce qui coûte — chiffres, pas impressions, 2.4 Note attribuée, 2. État des lieux mesuré

### Community 174 - "AppState"
Cohesion: 0.24
Nodes (11): AppState, Arc, DbPool, Option, Result, Self, String, Vec (+3 more)

### Community 175 - "useCamera.ts"
Cohesion: 0.20
Nodes (6): FacingMode, MediaTrackCapabilities, MediaTrackConstraintSet, useCamera(), UseCameraProps, MobileScan()

### Community 176 - "VehicleSearchControlRecordInsert"
Cohesion: 0.25
Nodes (11): get_vehicle_status_by_plate(), get_vehicle_with_owner_by_plate(), insert_control_record_for_vehicle_search(), OffsetDateTime, Option, PgPool, Result, String (+3 more)

### Community 177 - "6. Arbitrages tranchés"
Cohesion: 0.40
Nodes (5): 6.1 Corrections apportées à l'esquisse initiale, 6.2 Décisions prises en séance, 6.3 Le point de conception le plus structurant — le trait `ExternalDataSource`, 6.4 Ce qui est explicitement différé, 6. Arbitrages tranchés

### Community 178 - "useCaptureCoaching.test.ts"
Cohesion: 0.38
Nodes (5): useCaptureCoaching(), UseCaptureCoachingProps, Mode, PhotoState, setup()

### Community 181 - "PARTIE 1 — Évaluation de l'existant"
Cohesion: 0.50
Nodes (4): 1.1 Forces à préserver, 1.2 Les 10 problèmes mesurés, 1.3 Verdict, PARTIE 1 — Évaluation de l'existant

### Community 182 - "PARTIE 2 — Pourquoi ce périmètre plutôt qu'un monolithe modulaire complet"
Cohesion: 0.50
Nodes (4): 2.1 Propriété décisive : ce n'est pas une impasse, 2.2 Coût comparé, 2.3 Couverture des problèmes, PARTIE 2 — Pourquoi ce périmètre plutôt qu'un monolithe modulaire complet

### Community 183 - "PARTIE 3 — Arborescence cible"
Cohesion: 0.50
Nodes (4): 3.1 Règle de promotion fichier → répertoire, 3.2 Suppression du suffixe `_queries`, 3.3 ★ Re-exports : le chemin externe ne bouge pas, PARTIE 3 — Arborescence cible

### Community 184 - "PARTIE 6 — Plan d'exécution"
Cohesion: 0.50
Nodes (4): Code à réutiliser tel quel — ne pas réécrire, Contraintes transverses, Fichiers critiques, PARTIE 6 — Plan d'exécution

### Community 185 - "useScanPlate.ts"
Cohesion: 0.29
Nodes (6): useScanPlate(), UseScanPlateProps, DetectionResult, useStabilityDetection(), UseStabilityDetectionProps, CameroonPlateClassification

### Community 186 - "otp_service_tests.rs"
Cohesion: 0.52
Nodes (6): setup_otp_service(), test_rate_limit_blocks_after_3_requests(), test_rate_limit_is_per_phone_number(), test_request_otp_succeeds(), test_validate_otp_no_key_fails(), test_validate_otp_wrong_code_fails()

### Community 187 - "Admin Terminate Session Endpoint (/api/v1/admin/terminate-session)"
Cohesion: 0.13
Nodes (15): Admin Session Restart, Same-Day Re-Entry Block After Termination, Re-Activation Regression Check (agents not permanently blocked), Opaque Refresh Token stored as SHA-256 hash, refresh_tokens Table (token_hash, user_id, device_id, expires_at), Admin Terminate Session Endpoint (/api/v1/admin/terminate-session), Device Status Lifecycle (INACTIVE/ACTIVE/SUSPENDED/REVOKED/PENDING), End-of-Shift Device Revocation (+7 more)

### Community 203 - "usePhotoCapture.ts"
Cohesion: 0.24
Nodes (10): ScanDetectionsList(), ScanDetectionsListProps, ScanResultCardProps, extractPlateFromAny(), findPlateInText(), normalizePlateCandidate(), PhotoCaptureState, UsePhotoCaptureProps (+2 more)

### Community 204 - "VehicleApiCredentials"
Cohesion: 0.38
Nodes (9): ApiUserAuth, ExternalApiHeaderParms, ExternalVehicle, Option, String, VehicleInfo, VehicleApiCredentials, VehicleApiResponse (+1 more)

### Community 205 - "restart_user_session"
Cohesion: 0.52
Nodes (6): restart_user_session(), Duration, PgPool, Result, Uuid, terminate_user_sessions()

### Community 206 - "EmailProvider"
Cohesion: 0.31
Nodes (7): EmailProvider, Send, Sync, EmailService, Arc, Result, Self

### Community 207 - "initialize_pool"
Cohesion: 0.38
Nodes (6): ensure_database_exists(), initialize_pool(), DbPool, Result, main(), Result

### Community 208 - "handlers/auth/mod.rs"
Cohesion: 0.33
Nodes (3): on_shift_ended(), PgPool, Uuid

## Ambiguous Edges - Review These
- `Modular Design Principle` → `Community Detection and Labeling`  [AMBIGUOUS]
  .claude/rules/general-coding.md · relation: conceptually_related_to
- `Image Version Pinning` → `minio Service (S3 cache)`  [AMBIGUOUS]
  .claude/skills/docker/SKILL.md · relation: references
- `Same-Day Re-Entry Block After Termination` → `Re-Activation Regression Check (agents not permanently blocked)`  [AMBIGUOUS]
  docs/admin_session_termination.md · relation: semantically_similar_to
- `Neutral Missing-Image Placeholder Graphic` → `IVISS Visual Identity: Vehicle-in-Shield on Navy`  [AMBIGUOUS]
  frontend/public/placeholder.svg · relation: conceptually_related_to
- `Pending Validation Review Workflow` → `Audit Logs View`  [AMBIGUOUS]
  frontend/public/screenshot-desktop.png · relation: conceptually_related_to

## Knowledge Gaps
- **487 isolated node(s):** `@modelcontextprotocol/server-filesystem`, `postgres-mcp`, `DATABASE_URI`, `mcp-server-git`, `@upstash/context7-mcp` (+482 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **16 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Modular Design Principle` and `Community Detection and Labeling`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Image Version Pinning` and `minio Service (S3 cache)`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **What is the exact relationship between `Same-Day Re-Entry Block After Termination` and `Re-Activation Regression Check (agents not permanently blocked)`?**
  _Edge tagged AMBIGUOUS (relation: semantically_similar_to) - confidence is low._
- **What is the exact relationship between `Neutral Missing-Image Placeholder Graphic` and `IVISS Visual Identity: Vehicle-in-Shield on Navy`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **What is the exact relationship between `Pending Validation Review Workflow` and `Audit Logs View`?**
  _Edge tagged AMBIGUOUS (relation: conceptually_related_to) - confidence is low._
- **Why does `AppError` connect `AppError` to `login`, `control_queries_tests.rs`, `get_user_profile`, `services/ocr/photo.rs`, `users.rs`, `submissions/mod.rs`, `activate`, `search.rs`, `engine.rs`, `update_organization`, `AuthenticatedAdmin`, `request_refresh`, `AppCache`, `StageTimings`, `list_audit_logs`, `vehicles.rs`, `errors.rs`, `VehicleSearchControlRecordInsert`, `submissions.rs`, `middleware/auth.rs`, `queries/organizations.rs`, `update_location`, `logout`, `restart_user_session`, `handlers/auth/mod.rs`, `change_password`, `preprocess`, `request_daily_login`, `hash_password`, `search_vehicle`, `OtpService`, `String`, `require_auth_web`, `queries/users/location.rs`?**
  _High betweenness centrality (0.146) - this node is a cross-community bridge._
- **Why does `AppCache` connect `AppCache` to `queries/organizations.rs`, `OtpService`, `AppError`, `users_and_user_manager_tests.rs`, `AppState`, `stats_handler_tests.rs`?**
  _High betweenness centrality (0.026) - this node is a cross-community bridge._