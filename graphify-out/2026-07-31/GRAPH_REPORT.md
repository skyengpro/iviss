# Graph Report - .  (2026-07-31)

## Corpus Check
- 8 files · ~298,597 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2837 nodes · 6065 edges · 166 communities (153 shown, 13 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 220 edges (avg confidence: 0.85)
- Token cost: 124,308 input · 0 output

## Community Hubs (Navigation)
- AppState
- cn
- control_queries_tests.rs
- middleware/auth.rs
- Button
- storeKey.ts
- ImageProcessor
- AuthenticatedAdmin
- auth_queries.rs
- UserManagement.tsx
- handlers/pending_submission.rs
- users_and_user_manager_tests.rs
- useAuth
- ocr_service_tests.rs
- mockControls.ts
- button.tsx
- use-toast.ts
- Backend perf/observability: opt-level 3, per-stage timings, OcrBudget, Semaphore, in-memory BMP
- hcloud_server.control_plane
- organization_queries.rs
- routes.ts
- dto/stats.rs
- sidebar.tsx
- handlers/search_vehicle.rs
- vehicle_client_service.rs
- stats_queries_tests.rs
- AppCache
- stats_handler_tests.rs
- authInterceptor.test.ts
- list_audit_logs
- vehicle_data_cache.rs
- authInterceptor.ts
- SmsProvider
- admin_login_and_refresh_token_tests.rs
- plate_format.rs
- IVISS Back Office Desktop Dashboard
- vehicle_queries.rs
- auth_queries_tests.rs
- binarize_replica.py
- PendingVehicles.tsx
- sms_provider_tests.rs
- users table
- Config
- TelemetryHandle
- Backend CI Report Job
- email_provider.rs
- Partner API Integration Flow
- ScanPlateResponse
- Shared backend-environment Anchor
- photo_ocr_service_tests.rs
- compilerOptions
- var.project_name
- submission_queries.rs
- IVISS README (English)
- components.json
- EmailProvider
- Status
- AppError
- ocr_service.rs
- tokenManager.ts
- config.rs
- Docker Rules Skill
- Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time)
- Admin Terminate Session Endpoint (/api/v1/admin/terminate-session)
- JwtService (RS256 access token issuance)
- API Route Groups (public, web-auth, agent-protected, admin, org-admin)
- ErrorBoundary.tsx
- metricsCollector.ts
- compilerOptions
- errors.rs
- Extraction Subagent Prompt
- VehicleStatusRow
- ScanResultData
- .mcp.json
- seed_admin.rs
- extract_image_field
- location_queries.rs
- setup_test_app
- Graphify Pipeline
- IVISS Platform
- AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret)
- OpenAPI Frontend Codegen (openapi-rq queries/requests)
- IVISS Docker Compose Stack
- start_minio_cache
- metrics-server.js
- mockExternalAPIs.ts
- OtpService
- Incremental Update Flow
- ControlRecord Interface
- Prometheus Metrics Server (port 9091)
- update_organization
- compilerOptions
- App.tsx
- chart.tsx
- IVISS Frontend Architecture
- IVISS Developer Documentation Index
- send_otp HTTP handler (POST /otp)
- SmsProvider trait
- hash_password
- skew_probe.py
- Project Graphify Integration Rules
- Admin-Only RBAC
- control_records table
- Validation du dossier contre le code réel et la documentation officielle
- mockAuth.ts
- useVehicles.test.ts
- FeatureFlags
- Sync Server (Rust/Axum intermediary binary)
- PendingSubmission
- Install Banner Component
- IVISS Visual Identity: Vehicle-in-Shield on Navy
- otp_service_tests.rs
- test_flow.sh
- SimpleStorage
- metrics.rs
- Preprocessing order fix: binarize+normalize polarity before deskew, white fill on rotation
- BEFORE upload_3.5.jpg - Cameroon (CMR) license plate CE 568 LR photo
- Graphify Knowledge Graph System
- restart_user_session
- gh-artifacts.sh
- try_seed
- AFTER_fixed_4.5_C5.png - post-processed Cameroon license plate image (CE 568 LR)
- Reference plate image CE568LR
- fetch_openapi.mjs
- cors_layer
- coverage.sh
- BEFORE sample license plate image (shipped_3.5_C5)
- Organization Work Window (start_work_time/end_work_time, UTC+1)
- usePWA.ts
- backend.tf
- networking.tf
- setup-remote-state.sh
- main
- SubmissionLocation
- Decision — Phase 1 scope: resolution + performance + observability, gate becomes non-blocking coaching
- Live scan — stability rewrite: sliding majority vote instead of consecutive-match confidence threshold
- Viewfinder — aspect 3.5→4.5, single source of truth computeViewfinderCrop
- PSM ensemble: require agreement between PSM 7 & 13, PSM 11 only as last resort
- FalkorDB Cypher Export
- DB Init Script
- phone_imei Uniqueness Conflict Check
- NotFound.tsx

## God Nodes (most connected - your core abstractions)
1. `AppError` - 167 edges
2. `AppState` - 83 edges
3. `cn()` - 80 edges
4. `useAuth()` - 43 edges
5. `Button` - 37 edges
6. `setup_test_app()` - 32 edges
7. `Config` - 26 edges
8. `setup_test_app()` - 25 edges
9. `setup_test_infrastructure()` - 25 edges
10. `AppCache` - 24 edges

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
- None detected.

## Hyperedges (group relationships)
- **Fabricated confidence (0.90/0.50) traced and fixed across backend, frontend, and tests** — ocr_perf_improvement_03_ticket_backend_confidence_semantics, ocr_perf_improvement_06_validation_documentaire_three_undocumented_defects, ocr_perf_improvement_05_tests_de_non_regression_confidence_never_synthesized, ocr_perf_improvement_02_ticket_frontend_stability_detection_majority_vote [INFERRED 0.85]
- **Coordinated defense against dealer-frame noise (CE568LR reference case) across frontend geometry, backend regex, and tests** — ocr_perf_improvement_02_ticket_frontend_viewfinder_geometry, ocr_perf_improvement_03_ticket_backend_anti_hallucination, ocr_perf_improvement_05_tests_de_non_regression_dealer_surround_rejection, ocr_perf_improvement_04_mesures_verifiees_viewfinder_overflow [INFERRED 0.85]
- **Sauvola binarization decision: crate-limit investigation, ADAPTIVE_C measurement, implementation, and gradient test** — ocr_perf_improvement_03_ticket_backend_sauvola_binarization, ocr_perf_improvement_04_mesures_verifiees_adaptive_c_rejected, ocr_perf_improvement_06_validation_documentaire_crate_limitations, ocr_perf_improvement_05_tests_de_non_regression_sauvola_gradient [INFERRED 0.85]
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

## Communities (166 total, 13 thin omitted)

### Community 0 - "AppState"
Cohesion: 0.06
Nodes (92): HeaderValue, AppState, String, Vec, DeviceStatus, ProvisionUserRequest, ProvisionUserResponse, ResendActivationRequest (+84 more)

### Community 1 - "cn"
Cohesion: 0.03
Nodes (63): BackOfficeHeader(), BackOfficeHeaderProps, BackOfficeLayoutInner(), BackOfficeLayoutProps, BackOfficeSidebar(), NavLink(), MobileHeader(), MobileHeaderProps (+55 more)

### Community 2 - "control_queries_tests.rs"
Cohesion: 0.05
Nodes (56): Config, Environment, LogLevel, mock_vehicle_api_credentials(), FromStr, Option, Result, Self (+48 more)

### Community 3 - "middleware/auth.rs"
Cohesion: 0.06
Nodes (63): formSchema, FormValues, minutesToTimeValue(), OrganizationForm(), OrganizationFormProps, timeValueToMinutes(), FormMode, formSchema (+55 more)

### Community 4 - "Button"
Cohesion: 0.05
Nodes (68): Expiry, AppCache, OtpEntry, OtpExpiry, Cache, Default, Duration, Instant (+60 more)

### Community 5 - "storeKey.ts"
Cohesion: 0.06
Nodes (62): EncodingKey, From, String, UpdateLocationRequest, UpdateLocationResponse, get_user_profile(), Arc, Extension (+54 more)

### Community 6 - "ImageProcessor"
Cohesion: 0.06
Nodes (40): MobileLayoutProps, MobileNavigation(), MobileSidebar(), MobileSidebarProps, StatCard(), StatCardProps, statCardVariants, Textarea (+32 more)

### Community 7 - "AuthenticatedAdmin"
Cohesion: 0.09
Nodes (28): ControlActivityChart(), ControlActivityChartProps, agentIcon, DEFAULT_CENTER, leafletIconProto, LiveControlMap(), LiveControlMapProps, BackOfficeLayout() (+20 more)

### Community 8 - "auth_queries.rs"
Cohesion: 0.06
Nodes (28): ScanDetectionsList(), ScanDetectionsListProps, ScanResultCard(), ScanResultCardProps, ScanViewfinder(), ScanViewfinderProps, FacingMode, useCamera() (+20 more)

### Community 9 - "UserManagement.tsx"
Cohesion: 0.06
Nodes (39): FnOnce, ApiDoc, SecurityAddon, ensure_database_exists(), initialize_pool(), DbPool, Result, main() (+31 more)

### Community 10 - "handlers/pending_submission.rs"
Cohesion: 0.07
Nodes (54): Root cause #2: backend CPU cost cascade (aborted spawn_blocking no-op), Audit du pipeline de capture/scan de plaques, Frontend quality gate measures the wrong thing (scale-dependent Laplacian, fail-open), Phase 1 decisions: non-blocking coaching, no pixel fabrication, observability first, Implementation lots A1-F2 (superseded by tickets 02/03), Realistic latency budget (server p95 < 1.5s), opt-level = "z" on a CPU-bound image pipeline, Root cause #1: resolution starvation (~135x38px ROI) (+46 more)

### Community 11 - "users_and_user_manager_tests.rs"
Cohesion: 0.10
Nodes (37): CreatePendingSubmissionRequest, DataEntryResponse, PendingSubmissionDetail, PendingSubmissionListItem, ReviewSubmissionRequest, ReviewSubmissionResponse, Option, Self (+29 more)

### Community 12 - "useAuth"
Cohesion: 0.13
Nodes (44): create_test_organization(), create_test_user(), generate_test_public_key_base64(), generate_test_rsa_keypair_pem(), hash_otp_code(), issue_admin_token(), Arc, Box (+36 more)

### Community 13 - "ocr_service_tests.rs"
Cohesion: 0.07
Nodes (29): GrayImage, adaptive_threshold(), add_border(), contrast_stretch_percentile(), deskew(), finalize(), invert_image(), morphology_open() (+21 more)

### Community 14 - "mockControls.ts"
Cohesion: 0.05
Nodes (35): SheetContent, SheetContentProps, SheetDescription, SheetFooter(), SheetHeader(), SheetOverlay, SheetTitle, sheetVariants (+27 more)

### Community 15 - "button.tsx"
Cohesion: 0.08
Nodes (19): MobileLayout(), VehicleActionFooterProps, VehicleHeader(), VehicleHeaderProps, VehicleImageCollapsible(), VehicleImageCollapsibleProps, VehicleNotFound(), VehicleNotFoundProps (+11 more)

### Community 16 - "use-toast.ts"
Cohesion: 0.07
Nodes (33): ProtectedRoute(), ProtectedRouteProps, Activate, AdminLogin, AppRoute, AuditLogPage, BackOfficeDashboard, BackOfficeReports (+25 more)

### Community 17 - "Backend perf/observability: opt-level 3, per-stage timings, OcrBudget, Semaphore, in-memory BMP"
Cohesion: 0.09
Nodes (29): Toast, ToastAction, ToastActionElement, ToastClose, ToastDescription, ToastProps, ToastTitle, toastVariants (+21 more)

### Community 18 - "hcloud_server.control_plane"
Cohesion: 0.16
Nodes (33): CreateOrganizationRequest, Organization, OrganizationDetails, OrganizationType, Option, String, Uuid, UpdateOrganizationRequest (+25 more)

### Community 19 - "organization_queries.rs"
Cohesion: 0.14
Nodes (21): applyAuthTokenToApiClient(), AuthProvider(), humanizeActivationError(), requiresDeviceReactivation(), defaultFocusSetup(), useProactiveRefresh(), AppRouter(), RequireAuth() (+13 more)

### Community 20 - "routes.ts"
Cohesion: 0.19
Nodes (32): ActivityData, ActivityFeedItemDto, ActivityFeedResponse, AgentLocationDto, ControlActivityPoint, ControlActivityResponse, DashboardRange, DashboardStats (+24 more)

### Community 21 - "dto/stats.rs"
Cohesion: 0.13
Nodes (32): Uuid, VehicleSearchRequest, cache_vehicle_search_result(), log_search_location(), record_vehicle_search_control(), Arc, IntoResponse, Json (+24 more)

### Community 22 - "sidebar.tsx"
Cohesion: 0.23
Nodes (32): ContainerAsync, PgPool, Postgres, Uuid, seed_agent(), seed_agent_location(), seed_control_record(), seed_organization() (+24 more)

### Community 23 - "handlers/search_vehicle.rs"
Cohesion: 0.20
Nodes (29): generate_test_rsa_keypair_pem(), Arc, ContainerAsync, PgPool, Postgres, Router, String, Uuid (+21 more)

### Community 24 - "vehicle_client_service.rs"
Cohesion: 0.11
Nodes (24): AuditAction, AuditLogEntry, AuditLogQuery, Err, FromStr, Option, Result, Self (+16 more)

### Community 25 - "stats_queries_tests.rs"
Cohesion: 0.21
Nodes (28): activate(), change_password(), login(), logout(), on_shift_ended(), RefreshChallengeResponse, request_daily_login(), request_refresh() (+20 more)

### Community 26 - "AppCache"
Cohesion: 0.15
Nodes (16): MockSmsProvider, OrangeSmsProvider, OrangeTokenResponse, Arc, Cache, Client, Instant, Result (+8 more)

### Community 27 - "stats_handler_tests.rs"
Cohesion: 0.28
Nodes (26): create_control_request(), ContainerAsync, PgPool, Postgres, Uuid, seed_agent(), seed_control_action(), seed_control_record_sql() (+18 more)

### Community 28 - "authInterceptor.test.ts"
Cohesion: 0.16
Nodes (23): hcloud_firewall.k3s, hcloud_server.control_plane, hcloud_server.worker, hcloud_ssh_key.k3s, output.cluster_name, output.control_plane_ips, output.kubeconfig, output.private_key_openssh (+15 more)

### Community 29 - "list_audit_logs"
Cohesion: 0.18
Nodes (25): create_admin_user(), generate_test_rsa_keypair_pem(), hash_password(), ContainerAsync, PgPool, Postgres, Router, String (+17 more)

### Community 30 - "vehicle_data_cache.rs"
Cohesion: 0.17
Nodes (15): classify(), classify_compact(), correct_digit(), correct_letter(), correct_with_mask(), extract_first(), find_candidate(), format_display() (+7 more)

### Community 31 - "authInterceptor.ts"
Cohesion: 0.12
Nodes (25): IVISS PWA App Icon 192x192, IVISS Brand Identity (Shield + Vehicle + Licence Plate), Navy / Teal Law-Enforcement Colour Palette, IVISS PWA App Icon 512x512, Multi-Resolution PWA Icon Set, IVISS PWA Favicon 64x64, Audit Logs View, IVISS Back Office Desktop Dashboard (+17 more)

### Community 32 - "SmsProvider"
Cohesion: 0.17
Nodes (8): signNonce(), decryptPrivateKey(), encryptPrivateKey(), generateKeyPair(), PasswordManager, retrieveKeyPair(), storeKeyPair(), mockedRetrieveKeyPair

### Community 33 - "admin_login_and_refresh_token_tests.rs"
Cohesion: 0.11
Nodes (14): Option, String, VehicleRow, create_test_status_row(), create_test_vehicle_row(), get_vehicle_status_by_plate(), get_vehicle_with_owner_by_plate(), Option (+6 more)

### Community 34 - "plate_format.rs"
Cohesion: 0.24
Nodes (24): AdminAuthRow, AuthValidationContext, blacklist_jti_cache(), blacklist_jti_db(), check_device_exists(), DeviceForLogin, find_admin_by_email(), find_admin_by_identity() (+16 more)

### Community 35 - "IVISS Back Office Desktop Dashboard"
Cohesion: 0.14
Nodes (21): adaptive_radius_for(), adaptive_threshold(), add_border(), contrast_stretch_percentile(), crop_to_viewfinder(), deskew(), estimate_skew_deg(), invert() (+13 more)

### Community 36 - "vehicle_queries.rs"
Cohesion: 0.18
Nodes (17): AppInner(), queryClient, BeforeInstallPromptEvent, PWAInstallPrompt(), Toaster(), ToasterProps, useMetrics(), collectPageLoadDuration() (+9 more)

### Community 37 - "auth_queries_tests.rs"
Cohesion: 0.12
Nodes (13): DailyLogin(), mockedGetDeviceId, mockedRequestRefresh, mockedSignNonce, mockedVerifyRefresh, getDeviceId(), resetDeviceId(), clearAllStoredData() (+5 more)

### Community 38 - "binarize_replica.py"
Cohesion: 0.18
Nodes (14): Result, test_twilio_authentication_headers(), test_twilio_form_parameters(), test_twilio_sms_provider_authentication_error(), test_twilio_sms_provider_empty_message(), test_twilio_sms_provider_invalid_phone_error(), test_twilio_sms_provider_network_timeout(), test_twilio_sms_provider_rate_limit_error() (+6 more)

### Community 39 - "PendingVehicles.tsx"
Cohesion: 0.23
Nodes (21): create_admin_user(), create_refresh_token(), generate_access_token(), generate_test_rsa_keypair_pem(), hash_password(), ContainerAsync, Option, PgPool (+13 more)

### Community 40 - "sms_provider_tests.rs"
Cohesion: 0.11
Nodes (21): Environment-Seeded Admin Bootstrap, Back-Office Sidebar Admin Nav Gating, Role Matrix (admin / agent / supervisor), Admin Bootstrap Idempotency Verification, PENDING_ACTIVATION Manual Password Workaround, Pre-Seeded Test Users (seed_data.sql), Gray-Card Submission Workflow, access_token_blacklist table (+13 more)

### Community 41 - "users table"
Cohesion: 0.13
Nodes (20): General Coding Guidelines, Modular Design Principle, Preserve Repository Architecture, Security-First Approach, Test Coverage Requirement, Rust Testing and Quality Gate, graph.json Shrink Guard, Backend Security Audit Job (+12 more)

### Community 42 - "Config"
Cohesion: 0.18
Nodes (15): aws_iam_openid_connect_provider.github_actions, aws_iam_role.github_actions_deploy, aws_iam_role_policy.deploy_permissions, aws_secretsmanager_secret.app_secrets, aws_secretsmanager_secret.provider_keys, aws_secretsmanager_secret.vehicle_api_keys, aws_secretsmanager_secret_version.app_secrets, aws_secretsmanager_secret_version.provider_keys (+7 more)

### Community 43 - "TelemetryHandle"
Cohesion: 0.19
Nodes (18): ImageUploadRequest, Option, String, ScanErrorData, ScanPlateResponse, error_response(), error_response_tuple(), extract_image_field() (+10 more)

### Community 44 - "Backend CI Report Job"
Cohesion: 0.12
Nodes (19): Image Version Pinning, SHA-Pinned GitHub Actions, Build & Push Docker Images Job, GHCR Image Registry, Determine Release Version Job, Semantic Release Action, Dual-Key OIDC Trust Hardening, PEM Key and Secret Parsing Hardening (+11 more)

### Community 45 - "email_provider.rs"
Cohesion: 0.13
Nodes (6): enhance_photo_result(), test_enhance_photo_result_already_valid_plate(), test_enhance_photo_result_invalid_but_strict_extracts(), test_enhance_photo_result_no_plate_found(), test_enhance_photo_result_strict_extract_boosts_low_confidence(), test_enhance_photo_result_strict_extract_does_not_lower_high_confidence()

### Community 46 - "Partner API Integration Flow"
Cohesion: 0.19
Nodes (10): AsyncSmtpTransport, LettreEmailProvider, MockEmailProvider, ResendEmailProvider, Arc, Client, Result, Self (+2 more)

### Community 47 - "ScanPlateResponse"
Cohesion: 0.20
Nodes (15): ControlHistory(), FILTER_TABS, FilterTab, PendingVehicles(), statusVariantMap, fetchWithAuth(), getSubmissionAuditLog(), getSubmissionById() (+7 more)

### Community 48 - "Shared backend-environment Anchor"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, baseUrl, isolatedModules, jsx, module, moduleDetection, moduleResolution (+10 more)

### Community 49 - "photo_ocr_service_tests.rs"
Cohesion: 0.23
Nodes (17): approve_submission(), create_pending_submission(), get_pending_submissions(), get_submission_audit_log(), get_submission_by_id(), reject_submission(), Option, PgPool (+9 more)

### Community 50 - "compilerOptions"
Cohesion: 0.13
Nodes (17): Cameroon License Plate Formats, isValidPlate Validation Utility, PlateInput Flexible Auto-Formatting, vehicle_owners table, vehicle_statuses table, vehicles table, Agent Login and Vehicle Lookup Flow, API Gateway (JWT extraction and claim forwarding) (+9 more)

### Community 51 - "var.project_name"
Cohesion: 0.12
Nodes (16): aliases, components, hooks, lib, ui, utils, rsc, $schema (+8 more)

### Community 52 - "submission_queries.rs"
Cohesion: 0.21
Nodes (16): AuthErrorCategory, classifyAuthError(), classifyAuthErrorMessage(), extractErrorCode(), extractErrorMessage(), HeyApiClient, isAdminSession(), isDeviceReactivationMessage() (+8 more)

### Community 53 - "IVISS README (English)"
Cohesion: 0.32
Nodes (8): Into, AppError, AppErrorResponse, ErrorCode, Error, IntoResponse, Self, String

### Community 54 - "components.json"
Cohesion: 0.25
Nodes (16): ec_public_key_to_b64_jwk(), generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, Router, String, Uuid (+8 more)

### Community 55 - "EmailProvider"
Cohesion: 0.21
Nodes (12): Deref, DerefMut, Drop, extract_plate_fuzzy(), normalise_plate(), put_tesseract(), Option, String (+4 more)

### Community 56 - "Status"
Cohesion: 0.15
Nodes (16): IVISS Platform Overview, Multi-Tenant Data Isolation by Organization, Retention and Archival Policy, IVISS PostgreSQL Schema, organizations table, Soft-Delete via deleted_at, SMS Gateway Service (binary), Axum Web Framework (+8 more)

### Community 57 - "AppError"
Cohesion: 0.14
Nodes (14): ControlAction, ControlRecord, ControlStats, ControlStatus, mockControls, Translatable, mockVehicles, mockVehicleService (+6 more)

### Community 58 - "ocr_service.rs"
Cohesion: 0.27
Nodes (15): ActivateRequest, ActivateResponse, AuthResponse, ChangePasswordRequest, ChangePasswordResponse, LoginRequest, LogoutRequestHeaders, RefreshRequest (+7 more)

### Community 59 - "tokenManager.ts"
Cohesion: 0.41
Nodes (13): Status, CustomsStatus, InsuranceStatus, OwnerInfo, PoliceStatus, Option, String, Vec (+5 more)

### Community 60 - "config.rs"
Cohesion: 0.21
Nodes (15): Custom Bridge Network, Docker Rules Skill, HEALTHCHECK on All Services, Named Volumes for Persistence, Non-Root Container User, No Secrets in Image Layers, Log to stdout/stderr Only, adminer Service (+7 more)

### Community 61 - "Docker Rules Skill"
Cohesion: 0.14
Nodes (15): Activation vs Daily Login Distinction, Testing Guide: Admin Session Management & Authentication, Browser-Persisted Device ID (iviss_device_id in localStorage), BE-08 Daily Login Flow (Agent), Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time), Redis-to-Moka In-Process Cache Migration, Backend Startup Sequence (config → pool → migrations → bootstrap seed → dev seed → AppCache), IVISS Database Development Guide (+7 more)

### Community 62 - "Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time)"
Cohesion: 0.13
Nodes (15): Admin Session Restart, Same-Day Re-Entry Block After Termination, Re-Activation Regression Check (agents not permanently blocked), Opaque Refresh Token stored as SHA-256 hash, refresh_tokens Table (token_hash, user_id, device_id, expires_at), Admin Terminate Session Endpoint (/api/v1/admin/terminate-session), Device Status Lifecycle (INACTIVE/ACTIVE/SUSPENDED/REVOKED/PENDING), End-of-Shift Device Revocation (+7 more)

### Community 63 - "Admin Terminate Session Endpoint (/api/v1/admin/terminate-session)"
Cohesion: 0.14
Nodes (15): Testing Guide: Admin Session Termination & Auth Fixes, Forced Logout with EventKeyStorage IndexedDB Wipe, JWT RSA Key Pair Generation and .env Setup, Access & Refresh Tokens (Backend), AccessTokenClaims (sub, device_id, role, exp, jti), AuthUser Extractor (FromRequestParts bearer verification), JwtService (RS256 access token issuance), Stateless Access Token (never stored server-side) (+7 more)

### Community 64 - "JwtService (RS256 access token issuance)"
Cohesion: 0.14
Nodes (15): IVISS Technical Architecture & System Design, Multi-Tenant Organization Hierarchy (Super Admin → Org → Admin → Supervisor → Agent), RBAC Role Definitions (super_admin, admin, supervisor, agent), Auto-Refresh Signature Mechanism, Two-Call Refresh Cycle (/auth/refresh + /auth/refresh/verify), User Interface (frontend), IVISS API Development Guide, Backend-First API Contract Workflow (+7 more)

### Community 65 - "API Route Groups (public, web-auth, agent-protected, admin, org-admin)"
Cohesion: 0.16
Nodes (5): App(), ErrorFallback(), ErrorBoundary, Props, State

### Community 66 - "ErrorBoundary.tsx"
Cohesion: 0.24
Nodes (8): AppInitializer(), AppInitializerProps, tryInitializeKeys(), mockedKeyManagement, Activate(), base64EncodeUtf8(), checkKeyPairExists(), KeyManagement()

### Community 67 - "metricsCollector.ts"
Cohesion: 0.13
Nodes (15): compilerOptions, allowImportingTsExtensions, isolatedModules, lib, module, moduleDetection, moduleResolution, noEmit (+7 more)

### Community 68 - "compilerOptions"
Cohesion: 0.23
Nodes (11): init_metrics(), init_telemetry(), init_tracer_provider(), Arc, Option, Result, Self, String (+3 more)

### Community 69 - "errors.rs"
Cohesion: 0.15
Nodes (14): Discrete Confidence Score Rubric, Hyperedge Extraction Rule, Extraction Subagent Prompt, EXTRACTED/INFERRED/AMBIGUOUS Audit Trail, Cumulative Token Cost Tracker, Semantic Extraction Cache, Graphify Honesty Rules, No API Key Required Policy (+6 more)

### Community 70 - "Extraction Subagent Prompt"
Cohesion: 0.29
Nodes (7): Date, OffsetDateTime, String, VehicleStatusRow, Option, VehicleInfo, VehicleService

### Community 71 - "VehicleStatusRow"
Cohesion: 0.14
Nodes (14): Development Hot-Reload Services, IVISS Docker Compose Stack, Admin Login Flow (email + password), Admin-Only RBAC, require_auth JWT Middleware, RequireAuth Route Guard (allowedRoles), Backend RBAC Enforcement Matrix (401/403/200), Manual RBAC Test Plan (+6 more)

### Community 72 - "ScanResultData"
Cohesion: 0.20
Nodes (12): DynamicImage, Eq, Self, ScanResultData, color_adaptive_crop(), extract_plate_strict(), photo_plate(), pick_best() (+4 more)

### Community 73 - ".mcp.json"
Cohesion: 0.18
Nodes (13): DATABASE_URI, npx, uvx, context7, filesystem, git, postgres, sequential-thinking (+5 more)

### Community 74 - "seed_admin.rs"
Cohesion: 0.27
Nodes (13): error_response(), error_response_tuple(), extract_image_field(), IntoResponse, Json, Multipart, Option, Result (+5 more)

### Community 75 - "extract_image_field"
Cohesion: 0.36
Nodes (13): create_test_user(), ContainerAsync, PgPool, Postgres, Result, Uuid, setup_test_db(), test_update_agent_location_handles_boundary_coordinates() (+5 more)

### Community 76 - "location_queries.rs"
Cohesion: 0.30
Nodes (13): generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, Router, String, Uuid, seed_users_with_active_session() (+5 more)

### Community 77 - "setup_test_app"
Cohesion: 0.21
Nodes (13): Graphify Skill Trigger, Folder Watch Auto-Rebuild, Graphify MCP Stdio Server, Token Reduction Benchmark, Cross-Repo Graph Merge, GitHub Repo Clone, Post-Commit Auto-Rebuild Hook, Code-Only Change Shortcut (+5 more)

### Community 78 - "Graphify Pipeline"
Cohesion: 0.17
Nodes (13): Robust Error Handling and Logging, Concurrency and Async Discipline, Rust Error Handling (thiserror/anyhow), Ownership and Type Modeling, Rust General Rules, Backend Build Job, metrics Service, Backend Architecture (Rust/Axum/SQLx) (+5 more)

### Community 79 - "IVISS Platform"
Cohesion: 0.15
Nodes (13): Mock SMS Provider (OTP retrieved from backend logs), AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret), Temporary Debug SSH Profile (port 22 open to 0.0.0.0/0), infra/scripts/deploy.sh Deployment Pipeline, Cost-Aware Edge-and-Origin Architecture (CloudFront → Lightsail), Edge Lockdown (CloudFront CIDR restriction, default enabled), IVISS Master Deployment & Infrastructure Guide (v3.3), SMS/Email Provider Configuration (mock, twilio, vonage, orange, resend, lettre) (+5 more)

### Community 80 - "AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret)"
Cohesion: 0.17
Nodes (13): API Gateway (JWT + Rate Limit + CORS), IVISS WebService (Rust + Axum + Tokio), Endpoint Change Order (DTO → handler → query → route → api_doc → tests → codegen), OpenAPI Frontend Codegen (openapi-rq queries/requests), Generated Artifacts Must Not Be Hand-Edited, OpenAPI and Codegen Drift Symptom/Fix, predev OpenAPI Fetch with Local Snapshot Fallback, Backend Layering (main/routes/handlers/services/queries/dto/models/middleware/db) (+5 more)

### Community 81 - "OpenAPI Frontend Codegen (openapi-rq queries/requests)"
Cohesion: 0.15
Nodes (11): activeSessions, app, clsGauge, errorsTotal, fidGauge, frontendUp, lcpGauge, pageLoadDuration (+3 more)

### Community 82 - "IVISS Docker Compose Stack"
Cohesion: 0.15
Nodes (10): AggregatedVehicleStatus, APIResponse, APIStatus, CustomsResult, InsuranceResult, mockAPIData, mockExternalAPIService, PoliceResult (+2 more)

### Community 83 - "start_minio_cache"
Cohesion: 0.37
Nodes (12): ActionType, ControlAction, ControlListQuery, ControlLocation, ControlPagedQuery, ListControlResponse, PagedControlsResponse, Option (+4 more)

### Community 84 - "metrics-server.js"
Cohesion: 0.35
Nodes (10): get_body_json(), Response, Value, test_bad_request_response(), test_database_error_response(), test_external_api_failure_response(), test_forbidden_response(), test_internal_error_response() (+2 more)

### Community 85 - "mockExternalAPIs.ts"
Cohesion: 0.32
Nodes (6): OtpService, Arc, Result, Self, String, Uuid

### Community 86 - "OtpService"
Cohesion: 0.18
Nodes (12): URL Ingestion into Corpus, Agent-Crawlable Wiki Export, Verbatim source_file Rule, save-result Work Memory Loop, Whisper Domain Hint Prompt, Whisper Video/Audio Transcription, build_merge Replace-on-Re-extract, Cluster-Only Rebuild (+4 more)

### Community 87 - "Incremental Update Flow"
Cohesion: 0.36
Nodes (11): ControlAction, create_control_record(), get_actions_for_control(), get_control_records(), get_paged_control_records(), Option, PgPool, Result (+3 more)

### Community 88 - "ControlRecord Interface"
Cohesion: 0.18
Nodes (12): Partner Compliance APIs (Insurance, Customs, Inspection, Wanted), ControlRecord Interface, VehicleStatus Enum (valid/warning/critical/pending), Core Database Tables and Domains, Query Layer Modules (auth/user/organization/vehicle/control/submission/stats/audit/location/session), Mock External Providers in Tests (no real SMS/email/OCR calls), Control Logging (frontend feature), Mock API Services Layer (mockAuth, mockVehicles, mockControls, mockExternalAPIs) (+4 more)

### Community 89 - "Prometheus Metrics Server (port 9091)"
Cohesion: 0.18
Nodes (12): OpenTelemetry Distributed Tracing (OTLP/HTTP to Alloy), Metrics Port Isolation from Public Ingress, IVISS Observability Stack, Prometheus Metrics Server (port 9091), kube-prometheus-stack ServiceMonitor Scrape Config, Layered Structured Logging (fmt + OTel), TelemetryHandle::shutdown Graceful Flush, Traces-Logs-Metrics Correlation in Grafana (+4 more)

### Community 90 - "update_organization"
Cohesion: 0.17
Nodes (12): Vehicle Check Workflow, Offline Cache Fallback Behaviour, Silent Service Worker Auto-Update, control_actions table, control_records table, Authority & Precision Color Palette, Micro-Gestalt Interactive Mechanics, Premium Layer Design Tokens (glassmorphism, gradients, shadows) (+4 more)

### Community 91 - "compilerOptions"
Cohesion: 0.17
Nodes (11): compilerOptions, allowJs, baseUrl, noImplicitAny, noUnusedLocals, noUnusedParameters, paths, skipLibCheck (+3 more)

### Community 92 - "App.tsx"
Cohesion: 0.24
Nodes (10): IdentificationMode, Option, String, SubmissionLocation, CreateControlRequest, CreateControlResponse, Option, String (+2 more)

### Community 93 - "chart.tsx"
Cohesion: 0.20
Nodes (10): Identification Modes (manual/photo/live), Frontend Layering (main/App/router/pages/components/hooks/services/openapi-rq/i18n), Frontend Test Helpers (setup.ts, createQueryWrapper, MemoryRouter), AuthProvider (React Auth Context), IVISS Design System (CSS Variables, Navy/Teal, status colors), IVISS Frontend Architecture, License Plate OCR Pipeline (react-webcam + ImageProcessor + Tesseract.js), ProtectedRoute / RequireAuth Guard (+2 more)

### Community 94 - "IVISS Frontend Architecture"
Cohesion: 0.22
Nodes (10): Deployment Modes A/B/C (local .env, Secrets Manager, CI/CD OIDC), GitHub Actions OIDC Assume Role (no static AWS keys), IVISS Coding Standards, IVISS Debugging Guide, IVISS Developer Getting Started, IVISS Project Structure, IVISS Developer Documentation Index, CI Pipeline Jobs (backend coverage/clippy/audit, frontend codegen/lint/build/Sonar) (+2 more)

### Community 95 - "IVISS Developer Documentation Index"
Cohesion: 0.27
Nodes (10): Partner Timeout / Unavailable Graceful Degradation, is_permanent_error, is_transient_error, NotificationJob (Otp / MagicEmail), process_notification_job, send_otp_with_retry (exponential backoff), Transient vs Permanent SMS Error Taxonomy, Permanent-to-400 / Transient-to-502 Status Mapping (+2 more)

### Community 96 - "send_otp HTTP handler (POST /otp)"
Cohesion: 0.33
Nodes (10): ApiSmsProvider (generic third-party API), AvlytextSmsProvider, ConsoleSmsProvider, OrangeSmsProvider, SmsProvider trait, SnsSmsProvider (AWS SNS), create_sms_provider factory, GatewayState (Arc<dyn SmsProvider>) (+2 more)

### Community 97 - "SmsProvider trait"
Cohesion: 0.24
Nodes (8): Arc, DbPool, Option, Result, Self, Send, Sync, VehicleDataCache

### Community 98 - "hash_password"
Cohesion: 0.47
Nodes (9): create_control(), get_list_control(), get_list_control_paged(), Arc, IntoResponse, Json, Query, Result (+1 more)

### Community 99 - "skew_probe.py"
Cohesion: 0.31
Nodes (7): EmailProvider, Send, Sync, EmailService, Arc, Result, Self

### Community 100 - "Project Graphify Integration Rules"
Cohesion: 0.38
Nodes (9): hash_password(), Result, String, test_hash_is_different_each_time(), test_hash_password_generates_valid_hash(), test_verify_password_correct_password(), test_verify_password_malformed_hash(), test_verify_password_wrong_password() (+1 more)

### Community 101 - "Admin-Only RBAC"
Cohesion: 0.27
Nodes (9): best_angle_in(), estimate_skew_deg(), fixture(), plate(), Replicates ocr_service::estimate_skew_angle exactly, in PIL. Goal: check…, Perfectly level plate: one band of glyph-like blocks across the middle., The image used by test_estimate_skew_angle_*: bright bars on black., rotated() (+1 more)

### Community 102 - "control_records table"
Cohesion: 0.28
Nodes (9): Deterministic Node ID Format, Native CLAUDE.md Integration, BFS Traversal Mode, Explain Node Flow, Inline NetworkX Traversal Fallback, Constrained Query Expansion, Shortest Path Between Concepts, Graph Health Check (+1 more)

### Community 103 - "Validation du dossier contre le code réel et la documentation officielle"
Cohesion: 0.25
Nodes (5): AuthSession, mockAuthService, mockUsers, User, UserRole

### Community 104 - "mockAuth.ts"
Cohesion: 0.22
Nodes (7): include, postcss.config.js, src, tailwind.config.ts, tsconfig.json, tsconfig.node.json, vitest.config.ts

### Community 105 - "useVehicles.test.ts"
Cohesion: 0.28
Nodes (5): FeatureFlags, Default, HashMap, Self, String

### Community 106 - "FeatureFlags"
Cohesion: 0.25
Nodes (8): Data Schema & Constants, Vehicle Interface, Cold Start Challenge (plate never cached in S3), Plate-Indexed JSON Object Model (vehicles/{PLATE}.json), Responsibility Boundary (IVISS App never touches External DB or S3), S3 Fallback Cache Strategy, Sync Server (Rust/Axum intermediary binary), IVISS Sync Server Architecture

### Community 107 - "Sync Server (Rust/Axum intermediary binary)"
Cohesion: 0.25
Nodes (8): GHCR Image Publishing via GitHub Actions, Prod Compose Profile (backend-prod / frontend-prod), Conventional Commit Prefixes, dev-Branch Release Trigger, Semantic Release Automation, Semantic Versioning (MAJOR.MINOR.PATCH), Version Tag Reset Procedure, Terraform + Ansible + GitHub Actions Deployment to AWS Lightsail

### Community 109 - "Install Banner Component"
Cohesion: 0.50
Nodes (7): PendingSubmission, OffsetDateTime, Option, String, Uuid, Value, SubmissionAuditLogRow

### Community 110 - "IVISS Visual Identity: Vehicle-in-Shield on Navy"
Cohesion: 0.29
Nodes (7): Install Banner Component, No localStorage Persistence for Banner Dismissal, PWA Installation Testing Guide, display-mode: standalone Installed Detection, PWA Meta Tags and Icons (index.html), SPA Root Mount (#root + /src/main.tsx), Crawler Allow-All Policy

### Community 111 - "otp_service_tests.rs"
Cohesion: 0.43
Nodes (7): IVISS Shield Mark (Apple Touch Icon), IVISS Shield Mark (Browser Favicon), Monochrome Shield-with-Checkmark Glyph (Safari Mask Icon), IVISS Shield Mark (Maskable PWA Icon 512x512), IVISS Visual Identity: Vehicle-in-Shield on Navy, Maskable Icon Safe-Zone Padding, Neutral Missing-Image Placeholder Graphic

### Community 112 - "test_flow.sh"
Cohesion: 0.52
Nodes (6): restart_user_session(), Duration, PgPool, Result, Uuid, terminate_user_sessions()

### Community 113 - "SimpleStorage"
Cohesion: 0.52
Nodes (6): setup_otp_service(), test_rate_limit_blocks_after_3_requests(), test_rate_limit_is_per_phone_number(), test_request_otp_succeeds(), test_validate_otp_no_key_fails(), test_validate_otp_wrong_code_fails()

### Community 115 - "Preprocessing order fix: binarize+normalize polarity before deskew, white fill on rotation"
Cohesion: 0.67
Nodes (5): ensure_cargo_llvm_cov(), print_error(), print_status(), print_warning(), test_flow.sh script

### Community 116 - "BEFORE upload_3.5.jpg - Cameroon (CMR) license plate CE 568 LR photo"
Cohesion: 0.60
Nodes (5): Body, Next, Request, Response, track_metrics()

### Community 117 - "Graphify Knowledge Graph System"
Cohesion: 0.47
Nodes (6): CEMAC regional emblem/logo on plate, CMR country code (Cameroon), Dealer plate frame text 'TAUNUS AUTO - Mercedes-Benz und smart in Wiesbaden', BEFORE upload_3.5.jpg - Cameroon (CMR) license plate CE 568 LR photo, OCR pipeline 'BEFORE' quality/resolution test sample, Plate text 'CE 568 LR'

### Community 118 - "restart_user_session"
Cohesion: 0.40
Nodes (5): GRAPH_REPORT.md (fallback for broad architecture review), Graphify Knowledge Graph System, graphify query/path/explain commands, graphify update . (AST-only refresh), graphify-out/wiki/index.md navigation

### Community 119 - "gh-artifacts.sh"
Cohesion: 0.40
Nodes (5): lib, DOM, DOM.Iterable, ES2020, WebWorker

### Community 120 - "try_seed"
Cohesion: 0.70
Nodes (4): delete_all_artifacts(), delete_old_artifacts(), list_artifacts(), gh-artifacts.sh script

### Community 121 - "AFTER_fixed_4.5_C5.png - post-processed Cameroon license plate image (CE 568 LR)"
Cohesion: 0.70
Nodes (4): PgPool, Result, run_seed_data(), try_seed()

### Community 122 - "Reference plate image CE568LR"
Cohesion: 0.40
Nodes (5): Cameroon national emblem and CMR country code on plate, AFTER_fixed_4.5_C5.png - post-processed Cameroon license plate image (CE 568 LR), Plate number CE 568 LR, Fixed processing parameters 4.5 / C5 (likely gamma/contrast + config variant used to binarize/clean plate image for OCR), BEFORE/AFTER OCR pipeline image comparison samples

### Community 123 - "fetch_openapi.mjs"
Cohesion: 0.50
Nodes (5): License plate number CE 568 LR, CEMAC (Central African Economic and Monetary Community) region marking, CMR country code (Cameroon), Reference plate image CE568LR, Taunus Auto / Mercedes-Benz und smart in Wiesbaden dealer stamp

### Community 124 - "cors_layer"
Cohesion: 0.60
Nodes (4): __dirname, fallbackToLocal(), fetchFromBackend(), main()

### Community 125 - "coverage.sh"
Cohesion: 0.50
Nodes (3): CorsLayer, cors_layer(), String

### Community 126 - "BEFORE sample license plate image (shipped_3.5_C5)"
Cohesion: 0.50
Nodes (3): LLVM_PROFILE_FILE, RUSTFLAGS, coverage.sh script

### Community 127 - "Organization Work Window (start_work_time/end_work_time, UTC+1)"
Cohesion: 0.50
Nodes (4): OCR pipeline BEFORE baseline sample set, Dealer sticker text: TAUNUS AUTO / Mercedes-Benz und smart in Wiesbaden, BEFORE sample license plate image (shipped_3.5_C5), License plate text CE 568 LR (Cameroon CMR)

### Community 128 - "usePWA.ts"
Cohesion: 0.67
Nodes (3): Organization Work Window (start_work_time/end_work_time, UTC+1), Shift Bounds Embedded in Access Token and devices.metadata, Org-Level Shift Windows Supersede SHIFT_START_HOUR/SHIFT_END_HOUR

### Community 130 - "networking.tf"
Cohesion: 0.67
Nodes (3): types, vite-plugin-pwa/client, vitest/globals

### Community 134 - "Decision — Phase 1 scope: resolution + performance + observability, gate becomes non-blocking coaching"
Cohesion: 0.67
Nodes (3): License plate text CE 568 LR (Cameroon, CMR/CEMAC), OCR pipeline post-upload test/sample case (resolution and speed evaluation), AFTER_upload_4.5.jpg - post-upload sample of Cameroon (CMR) CEMAC license plate 'CE 568 LR'

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
- `Dealer plate frame text 'TAUNUS AUTO - Mercedes-Benz und smart in Wiesbaden'` → `OCR pipeline 'BEFORE' quality/resolution test sample`  [AMBIGUOUS]
  ocr_perf_improvement/samples/BEFORE_upload_3.5.jpg · relation: complicates_ocr_of
- `Reference plate image CE568LR` → `Taunus Auto / Mercedes-Benz und smart in Wiesbaden dealer stamp`  [AMBIGUOUS]
  ocr_perf_improvement/samples/reference_plate_CE568LR.png · relation: sold_by_dealer

## Knowledge Gaps
- **406 isolated node(s):** `@modelcontextprotocol/server-filesystem`, `postgres-mcp`, `DATABASE_URI`, `mcp-server-git`, `@upstash/context7-mcp` (+401 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **13 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

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
- **What is the exact relationship between `Dealer plate frame text 'TAUNUS AUTO - Mercedes-Benz und smart in Wiesbaden'` and `OCR pipeline 'BEFORE' quality/resolution test sample`?**
  _Edge tagged AMBIGUOUS (relation: complicates_ocr_of) - confidence is low._
- **What is the exact relationship between `Reference plate image CE568LR` and `Taunus Auto / Mercedes-Benz und smart in Wiesbaden dealer stamp`?**
  _Edge tagged AMBIGUOUS (relation: sold_by_dealer) - confidence is low._