# Graph Report - iviss  (2026-08-04)

## Corpus Check
- 417 files · ~338,289 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2904 nodes · 6140 edges · 252 communities (169 shown, 83 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 203 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `54435934`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- handlers/auth.rs
- cn
- control_queries_tests.rs
- jwt_service.rs
- Settings.tsx
- deviceId.ts
- ImageProcessor
- dto/users.rs
- auth_queries.rs
- UserManagement.tsx
- handlers/pending_submission.rs
- users_and_user_manager_tests.rs
- useAuth
- Result
- mockControls.ts
- button.tsx
- use-toast.ts
- Audit du pipeline de capture/scan de plaques
- BackOfficeLayout.tsx
- organization_queries.rs
- routes.ts
- dto/stats.rs
- sidebar.tsx
- handlers/search_vehicle.rs
- vehicle_client_service.rs
- stats_queries_tests.rs
- setup_test_infrastructure
- stats_handler_tests.rs
- StageTimings
- list_audit_logs
- vehicle_data_cache.rs
- IVISS Back Office Desktop Dashboard
- sms_provider.rs
- admin_login_and_refresh_token_tests.rs
- plate_format.rs
- AppState
- vehicle_queries.rs
- auth_queries_tests.rs
- binarize_replica.py
- PendingVehicles.tsx
- sms_provider_tests.rs
- Backend CI Report Job
- Config
- admin_logout_tests.rs
- Shared backend-environment Anchor
- email_provider.rs
- hcloud_server.control_plane
- ScanPlateResponse
- captureFrame.ts
- handlers/stats.rs
- compilerOptions
- initialize_pool
- submission_queries.rs
- users table
- components.json
- EmailProvider
- Status
- setup_test_infrastructure
- TesseractGuard
- tokenManager.ts
- Docker Rules Skill
- Activation vs Daily Login Distinction
- Core Database Tables and Domains
- JwtService (RS256 access token issuance)
- Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time)
- var.project_name
- ErrorBoundary.tsx
- metricsCollector.ts
- compilerOptions
- TelemetryHandle
- ocr_service_tests.rs
- VehicleStatusRow
- photo_ocr_service.rs
- .mcp.json
- seed_admin.rs
- update_location
- location_queries.rs
- setup_test_app
- IVISS Platform
- AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret)
- OpenAPI Frontend Codegen (openapi-rq queries/requests)
- ocr_service.rs
- IVISS Docker Compose Stack
- start_minio_cache
- metrics-server.js
- mockExternalAPIs.ts
- OtpService
- storeKey.ts
- ControlRecord Interface
- Prometheus Metrics Server (port 9091)
- control_records table
- compilerOptions
- App.tsx
- IVISS Frontend Architecture
- IVISS Developer Documentation Index
- send_otp HTTP handler (POST /otp)
- SmsProvider trait
- middleware/auth.rs
- hash_password
- skew_probe.py
- Partner API Integration Flow
- AppCache
- Project Graphify Integration Rules
- getAccessToken
- mockAuth.ts
- authInterceptor.ts
- FeatureFlags
- Semantic Release Automation
- PendingSubmission
- AppError
- Install Banner Component
- IVISS Visual Identity: Vehicle-in-Shield on Navy
- test_flow.sh
- health.rs
- metrics.rs
- Ticket backend - pipeline OCR
- CEMAC regional emblem/logo on plate
- Graphify Knowledge Graph System
- config.rs
- gh-artifacts.sh
- try_seed
- Validation du dossier contre le code reel et la documentation officielle
- License plate number CE 568 LR
- fetch_openapi.mjs
- cors_layer
- coverage.sh
- OCR pipeline BEFORE baseline sample set
- IVISS README (English)
- Organization Work Window (start_work_time/end_work_time, UTC+1)
- ScanResultData
- authInterceptor.test.ts
- setup-remote-state.sh
- main
- SubmissionLocation
- opt-level = "z" on a CPU-bound image pipeline
- FalkorDB Cypher Export
- init_metrics Custom Metric Registration
- phone_imei Uniqueness Conflict Check
- ~130px is 4x beyond the willus.com optimum (30px), not a proven safety margin
- Extraction Subagent Prompt
- MobileScan.tsx
- Graphify Pipeline
- keyManagement.ts
- pre-commit
- init_db.sh
- Phase 1 decisions: non-blocking coaching, no pixel fabrication, observability first
- Capture scale order: grabFrame -> takePhoto (1.5s timeout) -> getScreenshot
- remote_setup.sh
- imageProcessor.ts
- errors.rs
- Incremental Update Flow
- External Vehicle Registry Database (PostgreSQL 9.4, third-party, read-only)
- require_auth_web
- General Coding Guidelines
- app_state.rs
- useScanPlate.ts
- ScanResultCard.tsx
- usePhotoCapture.ts
- useCaptureCoaching.test.ts
- Frontend quality gate measures the wrong thing (scale-dependent Laplacian, fail-open)
- Ticket frontend - pipeline de scan de plaques
- OEM cannot be set explicitly with leptess 0.14 (private tess_api field)
- Viewfinder vertical overflow by aspect: 3.5=+31%, 4.5=+2.1% (recommended)
- restart_user_session
- otp_service_tests.rs
- Admin Terminate Session Endpoint (/api/v1/admin/terminate-session)
- routes.rs
- .cache_necessary_data_from_database
- usePWA.ts
- GRAPH_REPORT.md (fallback for broad architecture review)
- graphify query/path/explain commands
- graphify update . (AST-only refresh)
- Conventional Commits + Semantic Release
- getScreenshot() captures a video frame, not a still photo
- Realistic latency budget (server p95 < 1.5s)
- Deviations from Tesseract documentation (dawgs, invert, whitelist, OEM)
- Root cause #1: resolution starvation (~135x38px ROI)
- CMR country code (Cameroon)
- Root cause #2: backend CPU cost cascade (aborted spawn_blocking no-op)
- Taunus Auto / Mercedes-Benz und smart in Wiesbaden dealer stamp
- Implementation lots A1-F2 (superseded by tickets 02/03)
- Live path JPEG quality 0.65->0.95 via LIVE_CROP_OPTIONS
- Shared viewfinder geometry: VF_ASPECT 3.5->4.5, computeViewfinderCrop single source of truth
- Native capture resolution: 1920x1080 + forceScreenshotSourceSize
- take_tesseract configuration: load_system_dawg, load_freq_dawg, tessedit_do_invert
- Corrected preprocessing order: contrast -> Sauvola threshold -> polarity -> deskew(binary) -> morphology -> border
- useStabilityDetection rewrite: sliding majority vote, max confidence, format_valid-only votes
- Backend performance & observability foundations (opt-level 3, ocr_timings.rs, Semaphore, BMP in-memory)
- useCaptureCoaching restricted to idle photo mode, 640px preview screenshot
- Mesures verifiees
- PSM 7/13 must agree before accepting; PSM 11 only if both empty
- photo_ocr_service.rs post-processing: drop char-by-char vote, derive format_valid from classification, skip redundant color crop
- plate_format.rs anti-hallucination: remove len>=4 fallback, bound fuzzy_correct to 3 chars, exclude Military/GovernmentLegacy from fuzzy substitution
- Plate geometry measurement: aspect 4.60 on reference photo
- Sauvola binarization in Rust (t = m*(1-k*(1-s/128)), k~=0.35), replacing local-mean-minus-C
- Confidence semantics: never overwrite mean_text_conf from format_valid (3 locations)
- Black 30px border on inverted path (add_border(&invert_image(&binary),0,255) is a no-op)
- Live JPEG quality measured 0.65 on dev vs 0.92 photo path; ~15KB gain not worth glyph ringing
- Capital letter height ~130px measured on 800x229 live crop; not proven to be a 4x margin
- Tests de non-regression a reintroduire
- ADAPTIVE_C sweep (5/10/15/20): glyphs visually identical, not the limiting factor
- Deskew angle bias on grayscale (+2.5 spurious) vs binary (0.0 correct) input
- test_pick_best_ensemble_skips_candidates_without_a_plate
- test_extract_plate_fuzzy_rejects_dealer_surround: 6 field-log strings must return None
- is_light_on_dark margin only 5-10 points on whole image vs 50% threshold
- test_deskew_fills_corners_with_background / test_deskew_output_is_bilevel (dark-on-light fixture)
- test_submitted_image_always_has_light_border (4 corners == 255, both polarities)
- test_sauvola_beats_local_mean_under_illumination_gradient
- test_confidence_is_never_synthesised across finalize/enhance_photo_result/pick_best
- Frontend test: tolerates a misread interleaved between agreeing readings
- AFTER_fixed_4.5_C5.png - post-processed Cameroon license plate image (CE 568 LR)
- Plate number CE 568 LR
- Naming/value corrections to tickets (VF_ASPECT, LIVE_CROP_OPTIONS, minConfidence=40, take_tesseract)
- leptess/leptonica crate limitations: no OEM, no invert_threshold, no thresholding_method, no pixSauvolaBinarizeTiled without direct FFI
- nginx client_max_body_size only covers dev same-origin fallback, not production cross-origin API
- Three real defects not covered by either ticket (black border, 2 extra confidence sites, 3rd hardcoded aspect)
- Dealer sticker text: TAUNUS AUTO / Mercedes-Benz und smart in Wiesbaden
- OCR / scan de plaques - dossier de reprise
- Decisions already settled: 1920x1080 capture, Sauvola binarization, ADAPTIVE_C not the issue, OEM unreachable
- CMR country code (Cameroon)
- Dealer plate frame text 'TAUNUS AUTO - Mercedes-Benz und smart in Wiesbaden'
- BEFORE upload_3.5.jpg - Cameroon (CMR) license plate CE 568 LR photo
- OCR pipeline 'BEFORE' quality/resolution test sample
- Plate text 'CE 568 LR'
- CEMAC (Central African Economic and Monetary Community) region marking
- CMR country code (Cameroon)
- Reference plate image CE568LR
- Taunus Auto / Mercedes-Benz und smart in Wiesbaden dealer stamp

## God Nodes (most connected - your core abstractions)
1. `AppError` - 178 edges
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

## Communities (252 total, 83 thin omitted)

### Community 0 - "handlers/auth.rs"
Cohesion: 0.21
Nodes (28): activate(), change_password(), login(), logout(), on_shift_ended(), RefreshChallengeResponse, request_daily_login(), request_refresh() (+20 more)

### Community 1 - "cn"
Cohesion: 0.04
Nodes (45): NavLink(), ScanActionButtons(), ScanActionButtonsProps, ScanTopControls(), ScanTopControlsProps, NavLink, NavLinkCompatProps, Alert (+37 more)

### Community 2 - "control_queries_tests.rs"
Cohesion: 0.08
Nodes (65): ControlAction, IdentificationMode, CreateControlRequest, CreateControlResponse, Option, String, Uuid, ActionType (+57 more)

### Community 3 - "jwt_service.rs"
Cohesion: 0.19
Nodes (20): EncodingKey, AccessTokenClaims, generate_test_keys(), JwtService, Result, Self, String, UserRole (+12 more)

### Community 4 - "Settings.tsx"
Cohesion: 0.08
Nodes (30): ControlActivityChart(), ControlActivityChartProps, agentIcon, DEFAULT_CENTER, leafletIconProto, LiveControlMap(), LiveControlMapProps, BackOfficeLayout() (+22 more)

### Community 5 - "deviceId.ts"
Cohesion: 0.14
Nodes (10): DailyLogin(), getDeviceId(), resetDeviceId(), clearAllStoredData(), MyDatabase, SimpleStorage, storage, dummyDB (+2 more)

### Community 6 - "ImageProcessor"
Cohesion: 0.32
Nodes (3): ImageProcessor, PLATE_PATTERNS, ViewfinderBox

### Community 7 - "dto/users.rs"
Cohesion: 0.08
Nodes (50): ActivateRequest, ActivateResponse, AuthResponse, ChangePasswordRequest, ChangePasswordResponse, LoginRequest, LogoutRequestHeaders, RefreshRequest (+42 more)

### Community 8 - "auth_queries.rs"
Cohesion: 0.24
Nodes (24): AdminAuthRow, AuthValidationContext, blacklist_jti_cache(), blacklist_jti_db(), check_device_exists(), DeviceForLogin, find_admin_by_email(), find_admin_by_identity() (+16 more)

### Community 9 - "UserManagement.tsx"
Cohesion: 0.06
Nodes (64): formSchema, FormValues, minutesToTimeValue(), OrganizationForm(), OrganizationFormProps, timeValueToMinutes(), FormMode, formSchema (+56 more)

### Community 10 - "handlers/pending_submission.rs"
Cohesion: 0.10
Nodes (37): CreatePendingSubmissionRequest, DataEntryResponse, PendingSubmissionDetail, PendingSubmissionListItem, ReviewSubmissionRequest, ReviewSubmissionResponse, Option, Self (+29 more)

### Community 11 - "users_and_user_manager_tests.rs"
Cohesion: 0.13
Nodes (44): create_test_organization(), create_test_user(), generate_test_public_key_base64(), generate_test_rsa_keypair_pem(), hash_otp_code(), issue_admin_token(), Arc, Box (+36 more)

### Community 12 - "useAuth"
Cohesion: 0.08
Nodes (25): MobileLayout(), MobileLayoutProps, MobileNavigation(), MobileSidebar(), MobileSidebarProps, isValidPlate(), PlateInput(), PlateInputProps (+17 more)

### Community 13 - "Result"
Cohesion: 0.24
Nodes (14): acquire_ocr_permit(), configure_tesseract(), decode_image(), encode_bmp(), finalize(), DynamicImage, Result, Self (+6 more)

### Community 14 - "mockControls.ts"
Cohesion: 0.14
Nodes (14): ControlAction, ControlRecord, ControlStats, ControlStatus, mockControls, Translatable, mockVehicles, mockVehicleService (+6 more)

### Community 15 - "button.tsx"
Cohesion: 0.10
Nodes (19): VehicleActionFooterProps, VehicleImageCollapsible(), VehicleImageCollapsibleProps, VehicleNotFound(), VehicleNotFoundProps, VehicleErrorState(), VehicleLoadingState(), Button (+11 more)

### Community 16 - "use-toast.ts"
Cohesion: 0.08
Nodes (31): Toast, ToastAction, ToastActionElement, ToastClose, ToastDescription, ToastProps, ToastTitle, toastVariants (+23 more)

### Community 18 - "BackOfficeLayout.tsx"
Cohesion: 0.10
Nodes (21): BackOfficeHeader(), BackOfficeHeaderProps, BackOfficeLayoutInner(), BackOfficeLayoutProps, BackOfficeSidebar(), MobileHeader(), MobileHeaderProps, mockNavigate (+13 more)

### Community 19 - "organization_queries.rs"
Cohesion: 0.16
Nodes (33): CreateOrganizationRequest, Organization, OrganizationDetails, OrganizationType, Option, String, Uuid, UpdateOrganizationRequest (+25 more)

### Community 20 - "routes.ts"
Cohesion: 0.07
Nodes (32): AppRouter(), Activate, AdminLogin, AppRoute, AuditLogPage, BackOfficeDashboard, BackOfficeReports, backOfficeRoutes (+24 more)

### Community 21 - "dto/stats.rs"
Cohesion: 0.19
Nodes (32): ActivityData, ActivityFeedItemDto, ActivityFeedResponse, AgentLocationDto, ControlActivityPoint, ControlActivityResponse, DashboardRange, DashboardStats (+24 more)

### Community 22 - "sidebar.tsx"
Cohesion: 0.05
Nodes (35): SheetContent, SheetContentProps, SheetDescription, SheetFooter(), SheetHeader(), SheetOverlay, SheetTitle, sheetVariants (+27 more)

### Community 23 - "handlers/search_vehicle.rs"
Cohesion: 0.13
Nodes (32): Uuid, VehicleSearchRequest, cache_vehicle_search_result(), log_search_location(), record_vehicle_search_control(), Arc, IntoResponse, Json (+24 more)

### Community 24 - "vehicle_client_service.rs"
Cohesion: 0.14
Nodes (24): build_search_result(), ApiUserAuth, clean_value(), decode_basic_html_entities(), ExternalApiHeaderParms, html_to_text(), is_vehicle_not_found_response(), parse_inline_customs_status() (+16 more)

### Community 25 - "stats_queries_tests.rs"
Cohesion: 0.23
Nodes (32): ContainerAsync, PgPool, Postgres, Uuid, seed_agent(), seed_agent_location(), seed_control_record(), seed_organization() (+24 more)

### Community 26 - "setup_test_infrastructure"
Cohesion: 0.12
Nodes (29): generate_test_rsa_keypair_pem(), hash_otp_code(), Arc, Box, ContainerAsync, Error, PgPool, Postgres (+21 more)

### Community 27 - "stats_handler_tests.rs"
Cohesion: 0.20
Nodes (29): generate_test_rsa_keypair_pem(), Arc, ContainerAsync, PgPool, Postgres, Router, String, Uuid (+21 more)

### Community 28 - "StageTimings"
Cohesion: 0.16
Nodes (11): is_budget_exceeded(), OcrBudget, OcrBudgetExceeded, Duration, FnOnce, Instant, Result, Self (+3 more)

### Community 29 - "list_audit_logs"
Cohesion: 0.11
Nodes (24): AuditAction, AuditLogEntry, AuditLogQuery, Err, FromStr, Option, Result, Self (+16 more)

### Community 30 - "vehicle_data_cache.rs"
Cohesion: 0.14
Nodes (20): cache_partition_for_plate(), CachedEntry, CachedVehicleData, decrypt(), decrypt_rejects_short_payload(), decrypt_with_wrong_key_fails(), encrypt(), encrypt_decrypt_round_trip() (+12 more)

### Community 31 - "IVISS Back Office Desktop Dashboard"
Cohesion: 0.12
Nodes (25): IVISS PWA App Icon 192x192, IVISS Brand Identity (Shield + Vehicle + Licence Plate), Navy / Teal Law-Enforcement Colour Palette, IVISS PWA App Icon 512x512, Multi-Resolution PWA Icon Set, IVISS PWA Favicon 64x64, Audit Logs View, IVISS Back Office Desktop Dashboard (+17 more)

### Community 32 - "sms_provider.rs"
Cohesion: 0.16
Nodes (13): MockSmsProvider, OrangeSmsProvider, OrangeTokenResponse, Arc, Cache, Client, Instant, Result (+5 more)

### Community 33 - "admin_login_and_refresh_token_tests.rs"
Cohesion: 0.18
Nodes (25): create_admin_user(), generate_test_rsa_keypair_pem(), hash_password(), ContainerAsync, PgPool, Postgres, Router, String (+17 more)

### Community 34 - "plate_format.rs"
Cohesion: 0.14
Nodes (16): classify(), classify_compact(), correct_digit(), correct_letter(), correct_with_mask(), extract_first(), find_candidate(), format_display() (+8 more)

### Community 35 - "AppState"
Cohesion: 0.32
Nodes (26): AppState, String, Vec, delete_user(), get_user(), list_org_users(), list_organizations(), list_users() (+18 more)

### Community 36 - "vehicle_queries.rs"
Cohesion: 0.11
Nodes (14): Option, String, VehicleRow, create_test_status_row(), create_test_vehicle_row(), get_vehicle_status_by_plate(), get_vehicle_with_owner_by_plate(), Option (+6 more)

### Community 37 - "auth_queries_tests.rs"
Cohesion: 0.20
Nodes (24): generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, String, Uuid, seed_device(), seed_refresh_token() (+16 more)

### Community 38 - "binarize_replica.py"
Cohesion: 0.14
Nodes (21): adaptive_radius_for(), adaptive_threshold(), add_border(), contrast_stretch_percentile(), crop_to_viewfinder(), deskew(), estimate_skew_deg(), invert() (+13 more)

### Community 39 - "PendingVehicles.tsx"
Cohesion: 0.10
Nodes (21): VehicleHeader(), VehicleHeaderProps, VehicleStatusGrid(), VehicleStatusGridProps, StatusBadge(), StatusBadgeProps, statusBadgeVariants, statusIcons (+13 more)

### Community 40 - "sms_provider_tests.rs"
Cohesion: 0.18
Nodes (14): Result, test_twilio_authentication_headers(), test_twilio_form_parameters(), test_twilio_sms_provider_authentication_error(), test_twilio_sms_provider_empty_message(), test_twilio_sms_provider_invalid_phone_error(), test_twilio_sms_provider_network_timeout(), test_twilio_sms_provider_rate_limit_error() (+6 more)

### Community 41 - "Backend CI Report Job"
Cohesion: 0.17
Nodes (16): Security-First Approach, Test Coverage Requirement, Rust Testing and Quality Gate, Backend Security Audit Job, Backend Clippy Job, Backend Test and Coverage Job, Backend Documentation Job, Backend Format Job (cargo fmt) (+8 more)

### Community 42 - "Config"
Cohesion: 0.17
Nodes (11): Config, Option, Result, String, Vec, test_parse_allowed_origins_accepts_explicit_origin_list(), test_parse_allowed_origins_rejects_path_or_trailing_slash(), test_parse_allowed_origins_rejects_wildcard() (+3 more)

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

### Community 47 - "ScanPlateResponse"
Cohesion: 0.12
Nodes (31): ImageUploadRequest, Option, String, ScanErrorData, ScanPlateResponse, error_response(), error_response_tuple(), extract_image_field() (+23 more)

### Community 48 - "captureFrame.ts"
Cohesion: 0.14
Nodes (10): bitmapToDataUrl(), blobToDataUrl(), captureFrame(), getImageCaptureCtor(), ImageCapture, ImageCaptureConstructor, Window, withTimeout() (+2 more)

### Community 49 - "handlers/stats.rs"
Cohesion: 0.35
Nodes (21): ActivityFeedQuery, ActivityQuery, get_activity_feed(), get_control_activity(), get_dashboard_stats(), get_org_activity_feed(), get_org_control_activity(), get_org_dashboard_stats() (+13 more)

### Community 50 - "compilerOptions"
Cohesion: 0.06
Nodes (34): compilerOptions, allowImportingTsExtensions, baseUrl, isolatedModules, jsx, lib, module, moduleDetection (+26 more)

### Community 51 - "initialize_pool"
Cohesion: 0.12
Nodes (10): ApiDoc, SecurityAddon, ensure_database_exists(), initialize_pool(), DbPool, Result, main(), Result (+2 more)

### Community 52 - "submission_queries.rs"
Cohesion: 0.23
Nodes (17): approve_submission(), create_pending_submission(), get_pending_submissions(), get_submission_audit_log(), get_submission_by_id(), reject_submission(), Option, PgPool (+9 more)

### Community 53 - "users table"
Cohesion: 0.11
Nodes (21): Environment-Seeded Admin Bootstrap, Back-Office Sidebar Admin Nav Gating, Role Matrix (admin / agent / supervisor), Admin Bootstrap Idempotency Verification, PENDING_ACTIVATION Manual Password Workaround, Pre-Seeded Test Users (seed_data.sql), Gray-Card Submission Workflow, access_token_blacklist table (+13 more)

### Community 54 - "components.json"
Cohesion: 0.12
Nodes (16): aliases, components, hooks, lib, ui, utils, rsc, $schema (+8 more)

### Community 55 - "EmailProvider"
Cohesion: 0.31
Nodes (7): EmailProvider, Send, Sync, EmailService, Arc, Result, Self

### Community 56 - "Status"
Cohesion: 0.38
Nodes (13): Status, CustomsStatus, InsuranceStatus, OwnerInfo, PoliceStatus, Option, String, Vec (+5 more)

### Community 57 - "setup_test_infrastructure"
Cohesion: 0.25
Nodes (16): ec_public_key_to_b64_jwk(), generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, Router, String, Uuid (+8 more)

### Community 58 - "TesseractGuard"
Cohesion: 0.25
Nodes (6): Deref, DerefMut, Drop, put_tesseract(), TesseractGuard, Target

### Community 59 - "tokenManager.ts"
Cohesion: 0.15
Nodes (18): applyAuthTokenToApiClient(), AuthProvider(), humanizeActivationError(), requiresDeviceReactivation(), AuthContext, AuthContextType, defaultFocusSetup(), useProactiveRefresh() (+10 more)

### Community 60 - "Docker Rules Skill"
Cohesion: 0.21
Nodes (15): Custom Bridge Network, Docker Rules Skill, HEALTHCHECK on All Services, Named Volumes for Persistence, Non-Root Container User, No Secrets in Image Layers, Log to stdout/stderr Only, adminer Service (+7 more)

### Community 61 - "Activation vs Daily Login Distinction"
Cohesion: 0.33
Nodes (7): Activation vs Daily Login Distinction, Testing Guide: Admin Session Management & Authentication, Browser-Persisted Device ID (iviss_device_id in localStorage), BE-08 Daily Login Flow (Agent), Agent Activation Flow (POST /auth/activate), Daily OTP Enforcement (shift-based login), Device Bootstrap (device_id UUID + Ed25519 keypair in IndexedDB)

### Community 62 - "Core Database Tables and Domains"
Cohesion: 0.17
Nodes (12): Opaque Refresh Token stored as SHA-256 hash, refresh_tokens Table (token_hash, user_id, device_id, expires_at), Security Standards (no secrets, hash refresh tokens, no token logging), Core Database Tables and Domains, Query Layer Modules (auth/user/organization/vehicle/control/submission/stats/audit/location/session), Gitleaks Secret Scanning, Noncurrent Version Lifecycle Policy (3 versions / 30 days), S3 Bucket Versioning as Audit Trail (+4 more)

### Community 63 - "JwtService (RS256 access token issuance)"
Cohesion: 0.14
Nodes (15): Testing Guide: Admin Session Termination & Auth Fixes, Forced Logout with EventKeyStorage IndexedDB Wipe, JWT RSA Key Pair Generation and .env Setup, Access & Refresh Tokens (Backend), AccessTokenClaims (sub, device_id, role, exp, jti), AuthUser Extractor (FromRequestParts bearer verification), JwtService (RS256 access token issuance), Stateless Access Token (never stored server-side) (+7 more)

### Community 64 - "Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time)"
Cohesion: 0.14
Nodes (16): IVISS Technical Architecture & System Design, Multi-Tenant Organization Hierarchy (Super Admin → Org → Admin → Supervisor → Agent), RBAC Role Definitions (super_admin, admin, supervisor, agent), Auto-Refresh Signature Mechanism, Two-Call Refresh Cycle (/auth/refresh + /auth/refresh/verify), Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time), OTP Rate Limiting and Attempt Throttling, Redis-to-Moka In-Process Cache Migration (+8 more)

### Community 65 - "var.project_name"
Cohesion: 0.18
Nodes (15): aws_iam_openid_connect_provider.github_actions, aws_iam_role.github_actions_deploy, aws_iam_role_policy.deploy_permissions, aws_secretsmanager_secret.app_secrets, aws_secretsmanager_secret.provider_keys, aws_secretsmanager_secret.vehicle_api_keys, aws_secretsmanager_secret_version.app_secrets, aws_secretsmanager_secret_version.provider_keys (+7 more)

### Community 66 - "ErrorBoundary.tsx"
Cohesion: 0.16
Nodes (5): App(), ErrorFallback(), ErrorBoundary, Props, State

### Community 67 - "metricsCollector.ts"
Cohesion: 0.31
Nodes (12): AppInner(), useMetrics(), collectPageLoadDuration(), destroyMetrics(), generateSessionId(), initMetrics(), observeCLS(), observeFID() (+4 more)

### Community 68 - "compilerOptions"
Cohesion: 0.11
Nodes (17): compilerOptions, allowImportingTsExtensions, isolatedModules, lib, module, moduleDetection, moduleResolution, noEmit (+9 more)

### Community 69 - "TelemetryHandle"
Cohesion: 0.23
Nodes (11): init_metrics(), init_telemetry(), init_tracer_provider(), Arc, Option, Result, Self, String (+3 more)

### Community 70 - "ocr_service_tests.rs"
Cohesion: 0.11
Nodes (10): dark_on_light(), deskew_fills_corners_with_background_not_black(), deskew_on_a_straight_binary_plate_picks_zero(), deskew_output_is_strictly_bilevel(), finalize_never_rewrites_confidence(), morphology_open_closes_a_single_pixel_hole(), morphology_open_preserves_a_solid_block(), GrayImage (+2 more)

### Community 71 - "VehicleStatusRow"
Cohesion: 0.29
Nodes (7): Date, OffsetDateTime, String, VehicleStatusRow, Option, VehicleInfo, VehicleService

### Community 72 - "photo_ocr_service.rs"
Cohesion: 0.12
Nodes (22): color_adaptive_crop(), enhance_photo_result(), estimate_plate_trapezoid(), extract_plate_strict(), fit_edge(), is_orange_plate_pixel(), perspective_rectify_color_crop(), photo_plate() (+14 more)

### Community 73 - ".mcp.json"
Cohesion: 0.18
Nodes (13): DATABASE_URI, npx, uvx, context7, filesystem, git, postgres, sequential-thinking (+5 more)

### Community 74 - "seed_admin.rs"
Cohesion: 0.30
Nodes (13): BootstrapResult, create_test_config(), ContainerAsync, PgPool, Postgres, Result, String, run_bootstrap_seed() (+5 more)

### Community 75 - "update_location"
Cohesion: 0.16
Nodes (16): From, String, UpdateLocationRequest, UpdateLocationResponse, get_user_profile(), Arc, Extension, IntoResponse (+8 more)

### Community 76 - "location_queries.rs"
Cohesion: 0.36
Nodes (13): create_test_user(), ContainerAsync, PgPool, Postgres, Result, Uuid, setup_test_db(), test_update_agent_location_handles_boundary_coordinates() (+5 more)

### Community 77 - "setup_test_app"
Cohesion: 0.30
Nodes (13): generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, Router, String, Uuid, seed_users_with_active_session() (+5 more)

### Community 78 - "IVISS Platform"
Cohesion: 0.29
Nodes (8): Backend Build Job, metrics Service, Backend Architecture (Rust/Axum/SQLx), PostgreSQL Database and Data Model, Frontend Architecture (React/Vite), IVISS Platform, Monitoring and Logging, Backend-Generated OpenAPI Contract

### Community 79 - "AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret)"
Cohesion: 0.15
Nodes (13): Mock SMS Provider (OTP retrieved from backend logs), AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret), Temporary Debug SSH Profile (port 22 open to 0.0.0.0/0), infra/scripts/deploy.sh Deployment Pipeline, Cost-Aware Edge-and-Origin Architecture (CloudFront → Lightsail), Edge Lockdown (CloudFront CIDR restriction, default enabled), IVISS Master Deployment & Infrastructure Guide (v3.3), SMS/Email Provider Configuration (mock, twilio, vonage, orange, resend, lettre) (+5 more)

### Community 80 - "OpenAPI Frontend Codegen (openapi-rq queries/requests)"
Cohesion: 0.17
Nodes (13): API Gateway (JWT + Rate Limit + CORS), IVISS WebService (Rust + Axum + Tokio), Endpoint Change Order (DTO → handler → query → route → api_doc → tests → codegen), OpenAPI Frontend Codegen (openapi-rq queries/requests), Generated Artifacts Must Not Be Hand-Edited, OpenAPI and Codegen Drift Symptom/Fix, predev OpenAPI Fetch with Local Snapshot Fallback, Backend Layering (main/routes/handlers/services/queries/dto/models/middleware/db) (+5 more)

### Community 81 - "ocr_service.rs"
Cohesion: 0.25
Nodes (18): add_border(), contrast_stretch_percentile(), deskew(), estimate_skew_angle(), invert_image(), is_light_on_dark(), morphology_open(), normalise_plate() (+10 more)

### Community 82 - "IVISS Docker Compose Stack"
Cohesion: 0.14
Nodes (14): Development Hot-Reload Services, IVISS Docker Compose Stack, Admin Login Flow (email + password), Admin-Only RBAC, require_auth JWT Middleware, RequireAuth Route Guard (allowedRoles), Backend RBAC Enforcement Matrix (401/403/200), Manual RBAC Test Plan (+6 more)

### Community 83 - "start_minio_cache"
Cohesion: 0.31
Nodes (11): get_missing_plate_returns_none(), make_test_vehicle(), ContainerAsync, FnOnce, second_store_is_deduped(), start_minio_cache(), store_and_get_no_encryption(), store_and_get_option_e_client_layer_verified() (+3 more)

### Community 84 - "metrics-server.js"
Cohesion: 0.15
Nodes (11): activeSessions, app, clsGauge, errorsTotal, fidGauge, frontendUp, lcpGauge, pageLoadDuration (+3 more)

### Community 85 - "mockExternalAPIs.ts"
Cohesion: 0.15
Nodes (10): AggregatedVehicleStatus, APIResponse, APIStatus, CustomsResult, InsuranceResult, mockAPIData, mockExternalAPIService, PoliceResult (+2 more)

### Community 86 - "OtpService"
Cohesion: 0.32
Nodes (6): OtpService, Arc, Result, Self, String, Uuid

### Community 87 - "storeKey.ts"
Cohesion: 0.25
Nodes (5): decryptPrivateKey(), encryptPrivateKey(), generateKeyPair(), PasswordManager, storeKeyPair()

### Community 88 - "ControlRecord Interface"
Cohesion: 0.33
Nodes (7): Partner Compliance APIs (Insurance, Customs, Inspection, Wanted), ControlRecord Interface, VehicleStatus Enum (valid/warning/critical/pending), Mock External Providers in Tests (no real SMS/email/OCR calls), Control Logging (frontend feature), Mock API Services Layer (mockAuth, mockVehicles, mockControls, mockExternalAPIs), Vehicle Status Check (parallel 4-API aggregation)

### Community 89 - "Prometheus Metrics Server (port 9091)"
Cohesion: 0.18
Nodes (12): OpenTelemetry Distributed Tracing (OTLP/HTTP to Alloy), Metrics Port Isolation from Public Ingress, IVISS Observability Stack, Prometheus Metrics Server (port 9091), kube-prometheus-stack ServiceMonitor Scrape Config, Layered Structured Logging (fmt + OTel), TelemetryHandle::shutdown Graceful Flush, Traces-Logs-Metrics Correlation in Grafana (+4 more)

### Community 90 - "control_records table"
Cohesion: 0.17
Nodes (12): Vehicle Check Workflow, Offline Cache Fallback Behaviour, Silent Service Worker Auto-Update, control_actions table, control_records table, Authority & Precision Color Palette, Micro-Gestalt Interactive Mechanics, Premium Layer Design Tokens (glassmorphism, gradients, shadows) (+4 more)

### Community 91 - "compilerOptions"
Cohesion: 0.17
Nodes (11): compilerOptions, allowJs, baseUrl, noImplicitAny, noUnusedLocals, noUnusedParameters, paths, skipLibCheck (+3 more)

### Community 92 - "App.tsx"
Cohesion: 0.32
Nodes (5): queryClient, BeforeInstallPromptEvent, PWAInstallPrompt(), Toaster(), ToasterProps

### Community 93 - "IVISS Frontend Architecture"
Cohesion: 0.20
Nodes (10): Identification Modes (manual/photo/live), Frontend Layering (main/App/router/pages/components/hooks/services/openapi-rq/i18n), Frontend Test Helpers (setup.ts, createQueryWrapper, MemoryRouter), AuthProvider (React Auth Context), IVISS Design System (CSS Variables, Navy/Teal, status colors), IVISS Frontend Architecture, License Plate OCR Pipeline (react-webcam + ImageProcessor + Tesseract.js), ProtectedRoute / RequireAuth Guard (+2 more)

### Community 94 - "IVISS Developer Documentation Index"
Cohesion: 0.13
Nodes (16): Deployment Modes A/B/C (local .env, Secrets Manager, CI/CD OIDC), GitHub Actions OIDC Assume Role (no static AWS keys), IVISS Coding Standards, Backend Startup Sequence (config → pool → migrations → bootstrap seed → dev seed → AppCache), IVISS Database Development Guide, Seed Mechanisms (admin bootstrap seed + SEED_DATA dev seed), SQLx Migrations (timestamp-ordered, additive-only), IVISS Debugging Guide (+8 more)

### Community 95 - "send_otp HTTP handler (POST /otp)"
Cohesion: 0.27
Nodes (10): Partner Timeout / Unavailable Graceful Degradation, is_permanent_error, is_transient_error, NotificationJob (Otp / MagicEmail), process_notification_job, send_otp_with_retry (exponential backoff), Transient vs Permanent SMS Error Taxonomy, Permanent-to-400 / Transient-to-502 Status Mapping (+2 more)

### Community 96 - "SmsProvider trait"
Cohesion: 0.33
Nodes (10): ApiSmsProvider (generic third-party API), AvlytextSmsProvider, ConsoleSmsProvider, OrangeSmsProvider, SmsProvider trait, SnsSmsProvider (AWS SNS), create_sms_provider factory, GatewayState (Arc<dyn SmsProvider>) (+2 more)

### Community 97 - "middleware/auth.rs"
Cohesion: 0.20
Nodes (16): decode_access_token_rs256(), extract_bearer_token(), extracts_bearer_token(), is_user_status_allowed(), rejects_non_bearer_header(), require_auth(), Arc, Next (+8 more)

### Community 98 - "hash_password"
Cohesion: 0.38
Nodes (9): hash_password(), Result, String, test_hash_is_different_each_time(), test_hash_password_generates_valid_hash(), test_verify_password_correct_password(), test_verify_password_malformed_hash(), test_verify_password_wrong_password() (+1 more)

### Community 99 - "skew_probe.py"
Cohesion: 0.27
Nodes (9): best_angle_in(), estimate_skew_deg(), fixture(), plate(), Replicates ocr_service::estimate_skew_angle exactly, in PIL. Goal: check…, Perfectly level plate: one band of glyph-like blocks across the middle., The image used by test_estimate_skew_angle_*: bright bars on black., rotated() (+1 more)

### Community 100 - "Partner API Integration Flow"
Cohesion: 0.13
Nodes (17): Cameroon License Plate Formats, isValidPlate Validation Utility, PlateInput Flexible Auto-Formatting, vehicle_owners table, vehicle_statuses table, vehicles table, Agent Login and Vehicle Lookup Flow, API Gateway (JWT extraction and claim forwarding) (+9 more)

### Community 101 - "AppCache"
Cohesion: 0.24
Nodes (12): Expiry, AppCache, OtpEntry, OtpExpiry, Cache, Default, Duration, Instant (+4 more)

### Community 102 - "Project Graphify Integration Rules"
Cohesion: 0.32
Nodes (8): Deterministic Node ID Format, Native CLAUDE.md Integration, BFS Traversal Mode, Explain Node Flow, Inline NetworkX Traversal Fallback, Constrained Query Expansion, Shortest Path Between Concepts, Project Graphify Integration Rules

### Community 103 - "getAccessToken"
Cohesion: 0.22
Nodes (9): ProtectedRoute(), ProtectedRouteProps, RequireAuth(), mockNavigate, fetchWithAuth(), getBaseUrl(), isBackendUrl(), getAccessToken() (+1 more)

### Community 104 - "mockAuth.ts"
Cohesion: 0.25
Nodes (5): AuthSession, mockAuthService, mockUsers, User, UserRole

### Community 105 - "authInterceptor.ts"
Cohesion: 0.21
Nodes (16): AuthErrorCategory, classifyAuthError(), classifyAuthErrorMessage(), extractErrorCode(), extractErrorMessage(), HeyApiClient, isAdminSession(), isDeviceReactivationMessage() (+8 more)

### Community 106 - "FeatureFlags"
Cohesion: 0.28
Nodes (5): FeatureFlags, Default, HashMap, Self, String

### Community 107 - "Semantic Release Automation"
Cohesion: 0.25
Nodes (8): GHCR Image Publishing via GitHub Actions, Prod Compose Profile (backend-prod / frontend-prod), Conventional Commit Prefixes, dev-Branch Release Trigger, Semantic Release Automation, Semantic Versioning (MAJOR.MINOR.PATCH), Version Tag Reset Procedure, Terraform + Ansible + GitHub Actions Deployment to AWS Lightsail

### Community 108 - "PendingSubmission"
Cohesion: 0.50
Nodes (7): PendingSubmission, OffsetDateTime, Option, String, Uuid, Value, SubmissionAuditLogRow

### Community 109 - "AppError"
Cohesion: 0.32
Nodes (8): Into, AppError, AppErrorResponse, ErrorCode, Error, IntoResponse, Self, String

### Community 110 - "Install Banner Component"
Cohesion: 0.29
Nodes (7): Install Banner Component, No localStorage Persistence for Banner Dismissal, PWA Installation Testing Guide, display-mode: standalone Installed Detection, PWA Meta Tags and Icons (index.html), SPA Root Mount (#root + /src/main.tsx), Crawler Allow-All Policy

### Community 111 - "IVISS Visual Identity: Vehicle-in-Shield on Navy"
Cohesion: 0.43
Nodes (7): IVISS Shield Mark (Apple Touch Icon), IVISS Shield Mark (Browser Favicon), Monochrome Shield-with-Checkmark Glyph (Safari Mask Icon), IVISS Shield Mark (Maskable PWA Icon 512x512), IVISS Visual Identity: Vehicle-in-Shield on Navy, Maskable Icon Safe-Zone Padding, Neutral Missing-Image Placeholder Graphic

### Community 112 - "test_flow.sh"
Cohesion: 0.67
Nodes (5): ensure_cargo_llvm_cov(), print_error(), print_status(), print_warning(), test_flow.sh script

### Community 113 - "health.rs"
Cohesion: 0.60
Nodes (5): health_check(), metrics_export(), Arc, IntoResponse, State

### Community 114 - "metrics.rs"
Cohesion: 0.60
Nodes (5): Body, Next, Request, Response, track_metrics()

### Community 118 - "config.rs"
Cohesion: 0.16
Nodes (9): Environment, LogLevel, mock_vehicle_api_credentials(), FromStr, Self, test_config_helpers(), test_s3_cache_requires_bucket_when_enabled(), test_validate_rejects_empty_allowed_origins() (+1 more)

### Community 119 - "gh-artifacts.sh"
Cohesion: 0.70
Nodes (4): delete_all_artifacts(), delete_old_artifacts(), list_artifacts(), gh-artifacts.sh script

### Community 120 - "try_seed"
Cohesion: 0.70
Nodes (4): PgPool, Result, run_seed_data(), try_seed()

### Community 123 - "fetch_openapi.mjs"
Cohesion: 0.60
Nodes (4): __dirname, fallbackToLocal(), fetchFromBackend(), main()

### Community 124 - "cors_layer"
Cohesion: 0.50
Nodes (3): CorsLayer, cors_layer(), String

### Community 125 - "coverage.sh"
Cohesion: 0.50
Nodes (3): LLVM_PROFILE_FILE, RUSTFLAGS, coverage.sh script

### Community 127 - "IVISS README (English)"
Cohesion: 0.15
Nodes (16): IVISS Platform Overview, Multi-Tenant Data Isolation by Organization, Retention and Archival Policy, IVISS PostgreSQL Schema, organizations table, Soft-Delete via deleted_at, SMS Gateway Service (binary), Axum Web Framework (+8 more)

### Community 128 - "Organization Work Window (start_work_time/end_work_time, UTC+1)"
Cohesion: 0.67
Nodes (3): Organization Work Window (start_work_time/end_work_time, UTC+1), Shift Bounds Embedded in Access Token and devices.metadata, Org-Level Shift Windows Supersede SHIFT_START_HOUR/SHIFT_END_HOUR

### Community 129 - "ScanResultData"
Cohesion: 0.18
Nodes (14): Eq, Self, ScanResultData, extract_plate_fuzzy(), passes_agree(), pick_best_ensemble(), Option, Vec (+6 more)

### Community 130 - "authInterceptor.test.ts"
Cohesion: 0.17
Nodes (7): signNonce(), mockedGetDeviceId, mockedRequestRefresh, mockedSignNonce, mockedVerifyRefresh, retrieveKeyPair(), mockedRetrieveKeyPair

### Community 133 - "SubmissionLocation"
Cohesion: 0.67
Nodes (3): Option, String, SubmissionLocation

### Community 139 - "Extraction Subagent Prompt"
Cohesion: 0.15
Nodes (14): Discrete Confidence Score Rubric, Hyperedge Extraction Rule, Extraction Subagent Prompt, EXTRACTED/INFERRED/AMBIGUOUS Audit Trail, Cumulative Token Cost Tracker, Semantic Extraction Cache, Graphify Honesty Rules, No API Key Required Policy (+6 more)

### Community 140 - "MobileScan.tsx"
Cohesion: 0.19
Nodes (9): ScanViewfinder(), ScanViewfinderProps, FacingMode, MediaTrackCapabilities, MediaTrackConstraintSet, useCamera(), UseCameraProps, MobileScan() (+1 more)

### Community 141 - "Graphify Pipeline"
Cohesion: 0.21
Nodes (13): Graphify Skill Trigger, Folder Watch Auto-Rebuild, Graphify MCP Stdio Server, Token Reduction Benchmark, Cross-Repo Graph Merge, GitHub Repo Clone, Post-Commit Auto-Rebuild Hook, Code-Only Change Shortcut (+5 more)

### Community 142 - "keyManagement.ts"
Cohesion: 0.29
Nodes (6): AppInitializer(), AppInitializerProps, tryInitializeKeys(), mockedKeyManagement, checkKeyPairExists(), KeyManagement()

### Community 150 - "imageProcessor.ts"
Cohesion: 0.22
Nodes (5): CameroonPlateClassification, expectedCropOutput(), mockT, computeViewfinderCrop(), ViewfinderCrop

### Community 169 - "errors.rs"
Cohesion: 0.35
Nodes (10): get_body_json(), Response, Value, test_bad_request_response(), test_database_error_response(), test_external_api_failure_response(), test_forbidden_response(), test_internal_error_response() (+2 more)

### Community 170 - "Incremental Update Flow"
Cohesion: 0.18
Nodes (12): URL Ingestion into Corpus, Agent-Crawlable Wiki Export, Verbatim source_file Rule, save-result Work Memory Loop, Whisper Domain Hint Prompt, Whisper Video/Audio Transcription, build_merge Replace-on-Re-extract, Cluster-Only Rebuild (+4 more)

### Community 171 - "External Vehicle Registry Database (PostgreSQL 9.4, third-party, read-only)"
Cohesion: 0.18
Nodes (11): Data Schema & Constants, Vehicle Interface, EXTERNAL_DATABASE_URL Not Yet Loaded by Config, Cold Start Challenge (plate never cached in S3), External Vehicle Registry Database (PostgreSQL 9.4, third-party, read-only), iviss_reader Read-Only PostgreSQL User, Plate-Indexed JSON Object Model (vehicles/{PLATE}.json), Responsibility Boundary (IVISS App never touches External DB or S3) (+3 more)

### Community 172 - "require_auth_web"
Cohesion: 0.38
Nodes (10): require_admin(), require_auth_web(), require_org_admin(), Arc, Next, Request, Response, Result (+2 more)

### Community 173 - "General Coding Guidelines"
Cohesion: 0.20
Nodes (10): Robust Error Handling and Logging, General Coding Guidelines, Modular Design Principle, Preserve Repository Architecture, Concurrency and Async Discipline, Rust Error Handling (thiserror/anyhow), Ownership and Type Modeling, Rust General Rules (+2 more)

### Community 174 - "app_state.rs"
Cohesion: 0.24
Nodes (8): Arc, DbPool, Option, Result, Self, Send, Sync, SmsProvider

### Community 175 - "useScanPlate.ts"
Cohesion: 0.36
Nodes (5): useScanPlate(), UseScanPlateProps, DetectionResult, useStabilityDetection(), UseStabilityDetectionProps

### Community 176 - "ScanResultCard.tsx"
Cohesion: 0.32
Nodes (6): ScanDetectionsList(), ScanDetectionsListProps, ScanResultCard(), ScanResultCardProps, UsePhotoCaptureProps, DetectedPlate

### Community 177 - "usePhotoCapture.ts"
Cohesion: 0.43
Nodes (6): extractPlateFromAny(), findPlateInText(), normalizePlateCandidate(), PhotoCaptureState, usePhotoCapture(), PlateStatus

### Community 178 - "useCaptureCoaching.test.ts"
Cohesion: 0.38
Nodes (5): useCaptureCoaching(), UseCaptureCoachingProps, Mode, PhotoState, setup()

### Community 185 - "restart_user_session"
Cohesion: 0.52
Nodes (6): restart_user_session(), Duration, PgPool, Result, Uuid, terminate_user_sessions()

### Community 186 - "otp_service_tests.rs"
Cohesion: 0.52
Nodes (6): setup_otp_service(), test_rate_limit_blocks_after_3_requests(), test_rate_limit_is_per_phone_number(), test_request_otp_succeeds(), test_validate_otp_no_key_fails(), test_validate_otp_wrong_code_fails()

### Community 187 - "Admin Terminate Session Endpoint (/api/v1/admin/terminate-session)"
Cohesion: 0.33
Nodes (6): Admin Session Restart, Same-Day Re-Entry Block After Termination, Re-Activation Regression Check (agents not permanently blocked), Admin Terminate Session Endpoint (/api/v1/admin/terminate-session), Device Status Lifecycle (INACTIVE/ACTIVE/SUSPENDED/REVOKED/PENDING), End-of-Shift Device Revocation

### Community 188 - "routes.rs"
Cohesion: 0.53
Nodes (5): HeaderValue, assembly(), metrics_router(), Arc, Router

### Community 189 - ".cache_necessary_data_from_database"
Cohesion: 0.40
Nodes (3): Postgres, Result, Pool

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
- **459 isolated node(s):** `@modelcontextprotocol/server-filesystem`, `postgres-mcp`, `DATABASE_URI`, `mcp-server-git`, `@upstash/context7-mcp` (+454 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **83 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

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
- **Why does `AppError` connect `AppError` to `handlers/auth.rs`, `ScanResultData`, `control_queries_tests.rs`, `dto/users.rs`, `auth_queries.rs`, `handlers/pending_submission.rs`, `Result`, `organization_queries.rs`, `dto/stats.rs`, `handlers/search_vehicle.rs`, `StageTimings`, `list_audit_logs`, `AppState`, `vehicle_queries.rs`, `errors.rs`, `require_auth_web`, `handlers/stats.rs`, `submission_queries.rs`, `restart_user_session`, `.cache_necessary_data_from_database`, `photo_ocr_service.rs`, `update_location`, `location_queries.rs`, `ocr_service.rs`, `OtpService`, `middleware/auth.rs`, `hash_password`, `AppCache`?**
  _High betweenness centrality (0.089) - this node is a cross-community bridge._
- **Why does `AppState` connect `AppState` to `handlers/auth.rs`, `control_queries_tests.rs`, `jwt_service.rs`, `handlers/pending_submission.rs`, `organization_queries.rs`, `handlers/search_vehicle.rs`, `vehicle_client_service.rs`, `list_audit_logs`, `vehicle_data_cache.rs`, `require_auth_web`, `app_state.rs`, `handlers/stats.rs`, `EmailProvider`, `routes.rs`, `TelemetryHandle`, `update_location`, `OtpService`, `middleware/auth.rs`, `AppCache`, `health.rs`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._