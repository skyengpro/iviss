# Graph Report - .  (2026-07-30)

## Corpus Check
- 21 files · ~296,419 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 2812 nodes · 5937 edges · 185 communities (157 shown, 28 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 226 edges (avg confidence: 0.85)
- Token cost: 209,285 input · 52,319 output

## Community Hubs (Navigation)
- Back Office Layout Shell
- Backend Config & Environment
- Admin Forms & UI Primitives
- Dashboard Charts & Live Map
- Frontend Key/Crypto Bootstrap
- Telemetry & Tracing Setup
- Plate Scan UI Components
- OpenAPI Doc Generation
- Mobile Layout & Navigation
- Auth DTOs
- Pending Submission DTOs
- User Manager Integration Tests
- OCR Image Preprocessing
- Sheet & Dialog UI Primitives
- Toast Notification System
- Frontend App TSConfig
- Organization Models & Queries
- Vehicle Detail UI
- Frontend Router & Routes
- Vehicle Search Handler
- Dashboard Stats DTOs
- Stats Query Tests
- Stats Handler Tests
- Audit Log DTOs
- Auth Handlers
- Control Query Tests
- Admin Login Refresh Tests
- Plate Format Validation
- JWT Service & Claims
- IVISS Brand & Screenshots
- Vehicle Row Queries
- Auth Queries & Device Rows
- SMS Provider Implementations
- Auth Query Tests
- User Management Handlers
- SMS Provider Tests
- Admin Bootstrap & Role Matrix
- User/Vehicle Hooks & Textarea
- Stats Handlers
- Admin Logout Tests
- Cameroon Plate & Control Schema
- Scan DTOs & Photo Handler
- Rust Coding Rules & CI
- Location & User Profile
- Auth Middleware & Bearer Tokens
- Photo OCR Service Tests
- General Coding Guidelines & CI
- Email Provider (Lettre/SMTP)
- Frontend Backend Fetch Layer
- Frontend Node TSConfig
- Submission Queries
- Graphify Skill Configuration
- Release Pipeline & Image Registry
- Platform Overview & Multi-Tenancy
- In-Process Moka App Cache
- Shadcn Components Config
- Frontend Auth Interceptor
- Backend Error Types
- Tesseract OCR Service
- Frontend Auth Context
- Mock Control Data
- Vehicle Status DTOs
- Docker Rules & Compose Stack
- Login Flow & Cache Migration
- Device Lifecycle & Refresh Tokens
- Token Storage & Forced Logout
- Architecture Spec & RBAC Roles
- Frontend Metrics Collection
- Vehicle Service & Status Rows
- Photo OCR Service
- MCP Server Configuration
- Health & Metrics Endpoints
- Scan Handler & Multipart
- Location Queries
- Terminate Session Tests
- AWS Deployment & Secrets
- OpenAPI Codegen Contract
- Docker Compose & Release Docs
- Frontend Metrics Server
- Route Guards & Protected Routes
- Mock External Partner APIs
- Control List DTOs
- Error Response Tests
- User Queries
- OTP Service & Rate Limiting
- Graphify Ingestion & Wiki
- Control Record Queries
- Data Model & Query Layer Docs
- Observability Stack
- Error Boundary & Fallback
- Proactive Token Refresh Hook
- Root TSConfig
- Common & Control DTOs
- App Root & PWA Install Prompt
- Recharts Chart Wrapper
- RBAC Middleware
- Graphify Extraction Rules
- Frontend Architecture Layering
- Deployment Modes & Dev Guides
- Notification Retry & Backoff
- SMS Provider Trait Family
- Backend App State
- Control List Handlers
- Email Service Abstraction
- Password Hashing
- Admin Login & RBAC Enforcement
- UI Design System & Offline Page
- Mock Auth Service
- Feature Flags
- Graphify Query Traversal
- S3 Vehicle Cache Sync
- Pending Submission Model
- PWA Install Testing
- PWA Icon Artwork
- Session Restart Queries
- OTP Service Tests
- Backend Test Flow Script
- HTTP Metrics Middleware
- GitHub Artifacts Cleanup Script
- DB Pool & Cache Warmup
- Database Seed Data
- OpenAPI Fetch Script
- CORS Middleware
- Coverage Script
- Organization Shift Windows
- PWA Hook
- Terraform Remote State Setup
- Seed Binary
- Graph Database Exports
- Metrics Registration Docs
- Agent Assignment Checks
- Not Found Page
- Rust Parsing Test Fixture
- DB Init Script
- Remote Setup Script
- Conventional Commits Standard
- ESLint Config
- PostCSS Config
- Toast Lib Re-export
- Metrics Collector Test
- Example Test
- Vitest Setup
- Vite Env Types
- Vite Config
- JWT Debug Script
- DTO Module Root
- Handlers Module Root
- Backend Lib Root
- Middleware Module Root
- Cache Type
- HTTP Client Type
- OffsetDateTime Type
- Option Type
- Result Type
- Self Type
- Send Trait
- String Type
- Sync Trait
- Vec Type
- Backend Audit Findings
- Camera Capture Latency Fix
- OCR Post-Processing Cleanup
- Tesseract Conformance Table

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
- `manager Role = supervisor Business Label` --rationale_for--> `Role Matrix (admin / agent / supervisor)`  [INFERRED]
  IVISS-README.fr.pdf → docs/fe_admin_rbac.md

## Import Cycles
- None detected.

## Hyperedges (group relationships)
- **Session refresh resilience — five combining bugs causing agent logout** — docs_agent_session_refresh_resilience_summary_bug1_poisoned_promise, docs_agent_session_refresh_resilience_summary_bug2_false_logout, docs_agent_session_refresh_resilience_summary_bug3_retry_broken_body, docs_agent_session_refresh_resilience_summary_bug4_nonce_fragile_backend, docs_agent_session_refresh_resilience_summary_bug5_device_identity_wipe [EXTRACTED 0.90]
- **OCR resolution starvation: root cause → fix → verification measurement** — ocr_perf_improvement_00_audit_original_dev_resolution_starvation, ocr_perf_improvement_02_ticket_frontend_capture_native_resolution_fix, ocr_perf_improvement_04_mesures_verifiees_capital_height_measurement [EXTRACTED 0.90]
- **Deskew polarity correction: pipeline reorder → measured bias → regression test** — ocr_perf_improvement_03_ticket_backend_preprocessing_pipeline_order_fix, ocr_perf_improvement_04_mesures_verifiees_deskew_grayscale_vs_binary_bias, ocr_perf_improvement_05_tests_de_non_regression_deskew_polarity_fixture_test [EXTRACTED 0.90]
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

## Communities (185 total, 28 thin omitted)

### Community 0 - "Back Office Layout Shell"
Cohesion: 0.05
Nodes (126): HeaderValue, Into, AppState, String, Vec, AppError, AppErrorResponse, ErrorCode (+118 more)

### Community 1 - "Backend Config & Environment"
Cohesion: 0.04
Nodes (49): NavLink(), MobileSidebar(), MobileSidebarProps, ScanActionButtons(), ScanActionButtonsProps, ScanResultCard(), ScanTopControls(), ScanTopControlsProps (+41 more)

### Community 2 - "Admin Forms & UI Primitives"
Cohesion: 0.08
Nodes (65): ControlAction, IdentificationMode, CreateControlRequest, CreateControlResponse, Option, String, Uuid, ActionType (+57 more)

### Community 3 - "Dashboard Charts & Live Map"
Cohesion: 0.07
Nodes (52): EncodingKey, From, String, UpdateLocationRequest, UpdateLocationResponse, get_user_profile(), Arc, Extension (+44 more)

### Community 4 - "Frontend Key/Crypto Bootstrap"
Cohesion: 0.10
Nodes (26): BackOfficeLayout(), Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Input (+18 more)

### Community 5 - "Telemetry & Tracing Setup"
Cohesion: 0.07
Nodes (21): AppInitializer(), AppInitializerProps, tryInitializeKeys(), mockedKeyManagement, resetDeviceId(), checkKeyPairExists(), decryptPrivateKey(), encryptPrivateKey() (+13 more)

### Community 6 - "Plate Scan UI Components"
Cohesion: 0.07
Nodes (24): ScanDetectionsList(), ScanDetectionsListProps, ScanResultCardProps, FacingMode, useCamera(), UseCameraProps, extractPlateFromAny(), findPlateInText() (+16 more)

### Community 7 - "OpenAPI Doc Generation"
Cohesion: 0.09
Nodes (38): ActivateRequest, ActivateResponse, AuthResponse, ChangePasswordRequest, ChangePasswordResponse, LoginRequest, LogoutRequestHeaders, RefreshRequest (+30 more)

### Community 8 - "Mobile Layout & Navigation"
Cohesion: 0.10
Nodes (39): Expiry, AppCache, OtpEntry, OtpExpiry, Cache, Default, Duration, Instant (+31 more)

### Community 9 - "Auth DTOs"
Cohesion: 0.10
Nodes (36): UserForm(), AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter(), AlertDialogHeader(), AlertDialogOverlay (+28 more)

### Community 10 - "Pending Submission DTOs"
Cohesion: 0.10
Nodes (37): CreatePendingSubmissionRequest, DataEntryResponse, PendingSubmissionDetail, PendingSubmissionListItem, ReviewSubmissionRequest, ReviewSubmissionResponse, Option, Self (+29 more)

### Community 11 - "User Manager Integration Tests"
Cohesion: 0.13
Nodes (44): create_test_organization(), create_test_user(), generate_test_public_key_base64(), generate_test_rsa_keypair_pem(), hash_otp_code(), issue_admin_token(), Arc, Box (+36 more)

### Community 12 - "OCR Image Preprocessing"
Cohesion: 0.08
Nodes (20): MobileLayout(), MobileLayoutProps, MobileNavigation(), VehicleHeader(), VehicleHeaderProps, VehicleStatusGrid(), VehicleStatusGridProps, StatusBadge() (+12 more)

### Community 13 - "Sheet & Dialog UI Primitives"
Cohesion: 0.07
Nodes (29): GrayImage, adaptive_threshold(), add_border(), contrast_stretch_percentile(), deskew(), finalize(), invert_image(), morphology_open() (+21 more)

### Community 14 - "Toast Notification System"
Cohesion: 0.07
Nodes (29): ControlActivityChart(), agentIcon, DEFAULT_CENTER, leafletIconProto, LiveControlMap(), LiveControlMapProps, DialogContent, StatCard() (+21 more)

### Community 15 - "Frontend App TSConfig"
Cohesion: 0.09
Nodes (23): VehicleActionFooterProps, VehicleImageCollapsible(), VehicleImageCollapsibleProps, VehicleNotFound(), VehicleNotFoundProps, VehicleErrorState(), VehicleLoadingState(), Button (+15 more)

### Community 16 - "Organization Models & Queries"
Cohesion: 0.08
Nodes (31): Toast, ToastAction, ToastActionElement, ToastClose, ToastDescription, ToastProps, ToastTitle, toastVariants (+23 more)

### Community 17 - "Vehicle Detail UI"
Cohesion: 0.06
Nodes (36): Branch fix/agent-session-refresh-resilience, Bug 1 — Poisoned refreshPromise (performTokenRefresh), Bug 2 — False-positive logout on nonce expiry text match, Bug 3 — Broken retry for requests with body (consumed Request), Bug 4 (backend) — Nonce read/invalidated before auth, keyed by device_id only, Bug 5 — Silent device identity wipe on transient KeyManagement exception, Phase 1 — Frontend-only fix (deployable independently), Phase 2 — Backend hardening (optional, non-blocking) (+28 more)

### Community 18 - "Frontend Router & Routes"
Cohesion: 0.10
Nodes (29): ControlActivityChartProps, formSchema, FormValues, minutesToTimeValue(), OrganizationForm(), OrganizationFormProps, timeValueToMinutes(), FormMode (+21 more)

### Community 19 - "Vehicle Search Handler"
Cohesion: 0.16
Nodes (33): CreateOrganizationRequest, Organization, OrganizationDetails, OrganizationType, Option, String, Uuid, UpdateOrganizationRequest (+25 more)

### Community 20 - "Dashboard Stats DTOs"
Cohesion: 0.07
Nodes (31): Activate, AdminLogin, AppRoute, AuditLogPage, BackOfficeDashboard, BackOfficeReports, backOfficeRoutes, catchAllRoute (+23 more)

### Community 21 - "Stats Query Tests"
Cohesion: 0.19
Nodes (32): ActivityData, ActivityFeedItemDto, ActivityFeedResponse, AgentLocationDto, ControlActivityPoint, ControlActivityResponse, DashboardRange, DashboardStats (+24 more)

### Community 22 - "Stats Handler Tests"
Cohesion: 0.07
Nodes (27): Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupAction, SidebarGroupContent, SidebarGroupLabel, SidebarHeader (+19 more)

### Community 23 - "Audit Log DTOs"
Cohesion: 0.13
Nodes (32): Uuid, VehicleSearchRequest, cache_vehicle_search_result(), log_search_location(), record_vehicle_search_control(), Arc, IntoResponse, Json (+24 more)

### Community 24 - "Auth Handlers"
Cohesion: 0.14
Nodes (24): build_search_result(), ApiUserAuth, clean_value(), decode_basic_html_entities(), ExternalApiHeaderParms, html_to_text(), is_vehicle_not_found_response(), parse_inline_customs_status() (+16 more)

### Community 25 - "Control Query Tests"
Cohesion: 0.23
Nodes (32): ContainerAsync, PgPool, Postgres, Uuid, seed_agent(), seed_agent_location(), seed_control_record(), seed_organization() (+24 more)

### Community 26 - "Admin Login Refresh Tests"
Cohesion: 0.12
Nodes (29): generate_test_rsa_keypair_pem(), hash_otp_code(), Arc, Box, ContainerAsync, Error, PgPool, Postgres (+21 more)

### Community 27 - "Plate Format Validation"
Cohesion: 0.20
Nodes (29): generate_test_rsa_keypair_pem(), Arc, ContainerAsync, PgPool, Postgres, Router, String, Uuid (+21 more)

### Community 28 - "JWT Service & Claims"
Cohesion: 0.10
Nodes (21): BackOfficeHeader(), BackOfficeHeaderProps, BackOfficeLayoutInner(), BackOfficeLayoutProps, BackOfficeSidebar(), MobileHeader(), MobileHeaderProps, mockNavigate (+13 more)

### Community 29 - "IVISS Brand & Screenshots"
Cohesion: 0.11
Nodes (24): AuditAction, AuditLogEntry, AuditLogQuery, Err, FromStr, Option, Result, Self (+16 more)

### Community 30 - "Vehicle Row Queries"
Cohesion: 0.14
Nodes (21): Cache, Client, cache_partition_for_plate(), CachedEntry, CachedVehicleData, decrypt(), decrypt_rejects_short_payload(), decrypt_with_wrong_key_fails() (+13 more)

### Community 31 - "Auth Queries & Device Rows"
Cohesion: 0.12
Nodes (21): AuthErrorCategory, classifyAuthError(), classifyAuthErrorMessage(), extractErrorCode(), extractErrorMessage(), HeyApiClient, isAdminSession(), isDeviceReactivationMessage() (+13 more)

### Community 32 - "SMS Provider Implementations"
Cohesion: 0.15
Nodes (16): MockSmsProvider, OrangeSmsProvider, OrangeTokenResponse, Arc, Cache, Client, Instant, Result (+8 more)

### Community 33 - "Auth Query Tests"
Cohesion: 0.18
Nodes (25): create_admin_user(), generate_test_rsa_keypair_pem(), hash_password(), ContainerAsync, PgPool, Postgres, Router, String (+17 more)

### Community 34 - "User Management Handlers"
Cohesion: 0.17
Nodes (15): classify(), classify_compact(), correct_digit(), correct_letter(), correct_with_mask(), extract_first(), find_candidate(), format_display() (+7 more)

### Community 35 - "SMS Provider Tests"
Cohesion: 0.12
Nodes (25): IVISS PWA App Icon 192x192, IVISS Brand Identity (Shield + Vehicle + Licence Plate), Navy / Teal Law-Enforcement Colour Palette, IVISS PWA App Icon 512x512, Multi-Resolution PWA Icon Set, IVISS PWA Favicon 64x64, Audit Logs View, IVISS Back Office Desktop Dashboard (+17 more)

### Community 36 - "Admin Bootstrap & Role Matrix"
Cohesion: 0.11
Nodes (14): Option, String, VehicleRow, create_test_status_row(), create_test_vehicle_row(), get_vehicle_status_by_plate(), get_vehicle_with_owner_by_plate(), Option (+6 more)

### Community 37 - "User/Vehicle Hooks & Textarea"
Cohesion: 0.20
Nodes (24): generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, String, Uuid, seed_device(), seed_refresh_token() (+16 more)

### Community 38 - "Stats Handlers"
Cohesion: 0.14
Nodes (21): adaptive_radius_for(), adaptive_threshold(), add_border(), contrast_stretch_percentile(), crop_to_viewfinder(), deskew(), estimate_skew_deg(), invert() (+13 more)

### Community 39 - "Admin Logout Tests"
Cohesion: 0.17
Nodes (18): FILTER_TABS, FilterTab, PendingVehicles(), statusVariantMap, fetchWithAuth(), getBaseUrl(), isBackendUrl(), getSubmissionAuditLog() (+10 more)

### Community 40 - "Cameroon Plate & Control Schema"
Cohesion: 0.18
Nodes (14): Result, test_twilio_authentication_headers(), test_twilio_form_parameters(), test_twilio_sms_provider_authentication_error(), test_twilio_sms_provider_empty_message(), test_twilio_sms_provider_invalid_phone_error(), test_twilio_sms_provider_network_timeout(), test_twilio_sms_provider_rate_limit_error() (+6 more)

### Community 41 - "Scan DTOs & Photo Handler"
Cohesion: 0.11
Nodes (22): Environment-Seeded Admin Bootstrap, Back-Office Sidebar Admin Nav Gating, Role Matrix (admin / agent / supervisor), Admin Bootstrap Idempotency Verification, PENDING_ACTIVATION Manual Password Workaround, Pre-Seeded Test Users (seed_data.sql), Gray-Card Submission Workflow, access_token_blacklist table (+14 more)

### Community 42 - "Rust Coding Rules & CI"
Cohesion: 0.20
Nodes (9): Config, Environment, Option, Result, Self, String, Vec, SmsProviderCredentials (+1 more)

### Community 43 - "Location & User Profile"
Cohesion: 0.23
Nodes (21): create_admin_user(), create_refresh_token(), generate_access_token(), generate_test_rsa_keypair_pem(), hash_password(), ContainerAsync, Option, PgPool (+13 more)

### Community 44 - "Auth Middleware & Bearer Tokens"
Cohesion: 0.13
Nodes (20): General Coding Guidelines, Modular Design Principle, Preserve Repository Architecture, Security-First Approach, Test Coverage Requirement, Rust Testing and Quality Gate, graph.json Shrink Guard, Backend Security Audit Job (+12 more)

### Community 45 - "Photo OCR Service Tests"
Cohesion: 0.17
Nodes (11): AsyncSmtpTransport, EmailProviderCredentials, LettreEmailProvider, MockEmailProvider, ResendEmailProvider, Arc, Client, Result (+3 more)

### Community 46 - "General Coding Guidelines & CI"
Cohesion: 0.12
Nodes (20): Cameroon License Plate Formats, isValidPlate Validation Utility, PlateInput Flexible Auto-Formatting, Vehicle Check Workflow, control_actions table, control_records table, vehicle_owners table, vehicle_statuses table (+12 more)

### Community 47 - "Email Provider (Lettre/SMTP)"
Cohesion: 0.19
Nodes (18): ImageUploadRequest, Option, String, ScanErrorData, ScanPlateResponse, error_response(), error_response_tuple(), extract_image_field() (+10 more)

### Community 48 - "Frontend Backend Fetch Layer"
Cohesion: 0.12
Nodes (19): Image Version Pinning, SHA-Pinned GitHub Actions, Build & Push Docker Images Job, GHCR Image Registry, Determine Release Version Job, Semantic Release Action, Dual-Key OIDC Trust Hardening, PEM Key and Secret Parsing Hardening (+11 more)

### Community 49 - "Frontend Node TSConfig"
Cohesion: 0.13
Nodes (6): enhance_photo_result(), test_enhance_photo_result_already_valid_plate(), test_enhance_photo_result_invalid_but_strict_extracts(), test_enhance_photo_result_no_plate_found(), test_enhance_photo_result_strict_extract_boosts_low_confidence(), test_enhance_photo_result_strict_extract_does_not_lower_high_confidence()

### Community 50 - "Submission Queries"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, baseUrl, isolatedModules, jsx, module, moduleDetection, moduleResolution (+10 more)

### Community 51 - "Graphify Skill Configuration"
Cohesion: 0.12
Nodes (10): ApiDoc, SecurityAddon, ensure_database_exists(), initialize_pool(), DbPool, Result, main(), Result (+2 more)

### Community 52 - "Release Pipeline & Image Registry"
Cohesion: 0.23
Nodes (17): approve_submission(), create_pending_submission(), get_pending_submissions(), get_submission_audit_log(), get_submission_by_id(), reject_submission(), Option, PgPool (+9 more)

### Community 53 - "Platform Overview & Multi-Tenancy"
Cohesion: 0.15
Nodes (17): IVISS Platform Overview, Multi-Tenant Data Isolation by Organization, Retention and Archival Policy, IVISS PostgreSQL Schema, organizations table, Soft-Delete via deleted_at, SMS Gateway Service (binary), Axum Web Framework (+9 more)

### Community 54 - "In-Process Moka App Cache"
Cohesion: 0.12
Nodes (16): aliases, components, hooks, lib, ui, utils, rsc, $schema (+8 more)

### Community 55 - "Shadcn Components Config"
Cohesion: 0.18
Nodes (12): Arc, DbPool, Option, Result, Self, EmailProvider, Send, Sync (+4 more)

### Community 56 - "Frontend Auth Interceptor"
Cohesion: 0.38
Nodes (13): Status, CustomsStatus, InsuranceStatus, OwnerInfo, PoliceStatus, Option, String, Vec (+5 more)

### Community 57 - "Backend Error Types"
Cohesion: 0.25
Nodes (16): ec_public_key_to_b64_jwk(), generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, Router, String, Uuid (+8 more)

### Community 58 - "Tesseract OCR Service"
Cohesion: 0.21
Nodes (12): Deref, DerefMut, Drop, extract_plate_fuzzy(), normalise_plate(), put_tesseract(), Option, String (+4 more)

### Community 59 - "Frontend Auth Context"
Cohesion: 0.26
Nodes (10): applyAuthTokenToApiClient(), AuthProvider(), humanizeActivationError(), requiresDeviceReactivation(), AuthContext, AuthContextType, clearAccessToken(), clearTokens() (+2 more)

### Community 60 - "Mock Control Data"
Cohesion: 0.16
Nodes (10): LogLevel, mock_vehicle_api_credentials(), FromStr, test_config_helpers(), test_parse_allowed_origins_accepts_explicit_origin_list(), test_parse_allowed_origins_rejects_path_or_trailing_slash(), test_parse_allowed_origins_rejects_wildcard(), test_s3_cache_requires_bucket_when_enabled() (+2 more)

### Community 61 - "Vehicle Status DTOs"
Cohesion: 0.21
Nodes (15): Custom Bridge Network, Docker Rules Skill, HEALTHCHECK on All Services, Named Volumes for Persistence, Non-Root Container User, No Secrets in Image Layers, Log to stdout/stderr Only, adminer Service (+7 more)

### Community 62 - "Docker Rules & Compose Stack"
Cohesion: 0.14
Nodes (15): Activation vs Daily Login Distinction, Testing Guide: Admin Session Management & Authentication, Browser-Persisted Device ID (iviss_device_id in localStorage), BE-08 Daily Login Flow (Agent), Moka AppCache (otp_store, rate_limit, refresh_nonce, jti_blacklist, org_work_time), Redis-to-Moka In-Process Cache Migration, Backend Startup Sequence (config → pool → migrations → bootstrap seed → dev seed → AppCache), IVISS Database Development Guide (+7 more)

### Community 63 - "Login Flow & Cache Migration"
Cohesion: 0.13
Nodes (15): Admin Session Restart, Same-Day Re-Entry Block After Termination, Re-Activation Regression Check (agents not permanently blocked), Opaque Refresh Token stored as SHA-256 hash, refresh_tokens Table (token_hash, user_id, device_id, expires_at), Admin Terminate Session Endpoint (/api/v1/admin/terminate-session), Device Status Lifecycle (INACTIVE/ACTIVE/SUSPENDED/REVOKED/PENDING), End-of-Shift Device Revocation (+7 more)

### Community 64 - "Device Lifecycle & Refresh Tokens"
Cohesion: 0.14
Nodes (15): Testing Guide: Admin Session Termination & Auth Fixes, Forced Logout with EventKeyStorage IndexedDB Wipe, JWT RSA Key Pair Generation and .env Setup, Access & Refresh Tokens (Backend), AccessTokenClaims (sub, device_id, role, exp, jti), AuthUser Extractor (FromRequestParts bearer verification), JwtService (RS256 access token issuance), Stateless Access Token (never stored server-side) (+7 more)

### Community 65 - "Token Storage & Forced Logout"
Cohesion: 0.14
Nodes (15): IVISS Technical Architecture & System Design, Multi-Tenant Organization Hierarchy (Super Admin → Org → Admin → Supervisor → Agent), RBAC Role Definitions (super_admin, admin, supervisor, agent), Auto-Refresh Signature Mechanism, Two-Call Refresh Cycle (/auth/refresh + /auth/refresh/verify), User Interface (frontend), IVISS API Development Guide, Backend-First API Contract Workflow (+7 more)

### Community 66 - "Architecture Spec & RBAC Roles"
Cohesion: 0.16
Nodes (5): App(), ErrorFallback(), ErrorBoundary, Props, State

### Community 67 - "Frontend Metrics Collection"
Cohesion: 0.31
Nodes (12): AppInner(), useMetrics(), collectPageLoadDuration(), destroyMetrics(), generateSessionId(), initMetrics(), observeCLS(), observeFID() (+4 more)

### Community 68 - "Vehicle Service & Status Rows"
Cohesion: 0.13
Nodes (15): compilerOptions, allowImportingTsExtensions, isolatedModules, lib, module, moduleDetection, moduleResolution, noEmit (+7 more)

### Community 69 - "Photo OCR Service"
Cohesion: 0.23
Nodes (11): init_metrics(), init_telemetry(), init_tracer_provider(), Arc, Option, Result, Self, String (+3 more)

### Community 70 - "MCP Server Configuration"
Cohesion: 0.15
Nodes (14): Discrete Confidence Score Rubric, Hyperedge Extraction Rule, Extraction Subagent Prompt, EXTRACTED/INFERRED/AMBIGUOUS Audit Trail, Cumulative Token Cost Tracker, Semantic Extraction Cache, Graphify Honesty Rules, No API Key Required Policy (+6 more)

### Community 71 - "Health & Metrics Endpoints"
Cohesion: 0.29
Nodes (7): Date, OffsetDateTime, String, VehicleStatusRow, Option, VehicleInfo, VehicleService

### Community 72 - "Scan Handler & Multipart"
Cohesion: 0.20
Nodes (12): DynamicImage, Eq, Self, ScanResultData, color_adaptive_crop(), extract_plate_strict(), photo_plate(), pick_best() (+4 more)

### Community 73 - "Location Queries"
Cohesion: 0.18
Nodes (13): DATABASE_URI, npx, uvx, context7, filesystem, git, postgres, sequential-thinking (+5 more)

### Community 74 - "Terminate Session Tests"
Cohesion: 0.30
Nodes (13): BootstrapResult, create_test_config(), ContainerAsync, PgPool, Postgres, Result, String, run_bootstrap_seed() (+5 more)

### Community 75 - "AWS Deployment & Secrets"
Cohesion: 0.27
Nodes (13): error_response(), error_response_tuple(), extract_image_field(), IntoResponse, Json, Multipart, Option, Result (+5 more)

### Community 76 - "OpenAPI Codegen Contract"
Cohesion: 0.36
Nodes (13): create_test_user(), ContainerAsync, PgPool, Postgres, Result, Uuid, setup_test_db(), test_update_agent_location_handles_boundary_coordinates() (+5 more)

### Community 77 - "Docker Compose & Release Docs"
Cohesion: 0.30
Nodes (13): generate_test_rsa_keypair_pem(), ContainerAsync, PgPool, Postgres, Router, String, Uuid, seed_users_with_active_session() (+5 more)

### Community 78 - "Frontend Metrics Server"
Cohesion: 0.21
Nodes (13): Graphify Skill Trigger, Folder Watch Auto-Rebuild, Graphify MCP Stdio Server, Token Reduction Benchmark, Cross-Repo Graph Merge, GitHub Repo Clone, Post-Commit Auto-Rebuild Hook, Code-Only Change Shortcut (+5 more)

### Community 79 - "Route Guards & Protected Routes"
Cohesion: 0.17
Nodes (13): Robust Error Handling and Logging, Concurrency and Async Discipline, Rust Error Handling (thiserror/anyhow), Ownership and Type Modeling, Rust General Rules, Backend Build Job, metrics Service, Backend Architecture (Rust/Axum/SQLx) (+5 more)

### Community 80 - "Mock External Partner APIs"
Cohesion: 0.15
Nodes (13): Mock SMS Provider (OTP retrieved from backend logs), AWS Secrets Manager Secret Groups (app-secrets, provider-keys, cloudfront-origin-secret), Temporary Debug SSH Profile (port 22 open to 0.0.0.0/0), infra/scripts/deploy.sh Deployment Pipeline, Cost-Aware Edge-and-Origin Architecture (CloudFront → Lightsail), Edge Lockdown (CloudFront CIDR restriction, default enabled), IVISS Master Deployment & Infrastructure Guide (v3.3), SMS/Email Provider Configuration (mock, twilio, vonage, orange, resend, lettre) (+5 more)

### Community 81 - "Control List DTOs"
Cohesion: 0.17
Nodes (13): API Gateway (JWT + Rate Limit + CORS), IVISS WebService (Rust + Axum + Tokio), Endpoint Change Order (DTO → handler → query → route → api_doc → tests → codegen), OpenAPI Frontend Codegen (openapi-rq queries/requests), Generated Artifacts Must Not Be Hand-Edited, OpenAPI and Codegen Drift Symptom/Fix, predev OpenAPI Fetch with Local Snapshot Fallback, Backend Layering (main/routes/handlers/services/queries/dto/models/middleware/db) (+5 more)

### Community 82 - "Error Response Tests"
Cohesion: 0.15
Nodes (13): Development Hot-Reload Services, IVISS Docker Compose Stack, GHCR Image Publishing via GitHub Actions, Prod Compose Profile (backend-prod / frontend-prod), Manual RBAC Test Plan, Conventional Commit Prefixes, dev-Branch Release Trigger, Semantic Release Automation (+5 more)

### Community 83 - "User Queries"
Cohesion: 0.31
Nodes (11): FnOnce, get_missing_plate_returns_none(), make_test_vehicle(), ContainerAsync, second_store_is_deduped(), start_minio_cache(), store_and_get_no_encryption(), store_and_get_option_e_client_layer_verified() (+3 more)

### Community 84 - "OTP Service & Rate Limiting"
Cohesion: 0.15
Nodes (11): activeSessions, app, clsGauge, errorsTotal, fidGauge, frontendUp, lcpGauge, pageLoadDuration (+3 more)

### Community 85 - "Graphify Ingestion & Wiki"
Cohesion: 0.15
Nodes (10): AggregatedVehicleStatus, APIResponse, APIStatus, CustomsResult, InsuranceResult, mockAPIData, mockExternalAPIService, PoliceResult (+2 more)

### Community 86 - "Control Record Queries"
Cohesion: 0.32
Nodes (6): OtpService, Arc, Result, Self, String, Uuid

### Community 87 - "Data Model & Query Layer Docs"
Cohesion: 0.18
Nodes (12): URL Ingestion into Corpus, Agent-Crawlable Wiki Export, Verbatim source_file Rule, save-result Work Memory Loop, Whisper Domain Hint Prompt, Whisper Video/Audio Transcription, build_merge Replace-on-Re-extract, Cluster-Only Rebuild (+4 more)

### Community 88 - "Observability Stack"
Cohesion: 0.18
Nodes (12): Partner Compliance APIs (Insurance, Customs, Inspection, Wanted), ControlRecord Interface, VehicleStatus Enum (valid/warning/critical/pending), Core Database Tables and Domains, Query Layer Modules (auth/user/organization/vehicle/control/submission/stats/audit/location/session), Mock External Providers in Tests (no real SMS/email/OCR calls), Control Logging (frontend feature), Mock API Services Layer (mockAuth, mockVehicles, mockControls, mockExternalAPIs) (+4 more)

### Community 89 - "Error Boundary & Fallback"
Cohesion: 0.18
Nodes (12): OpenTelemetry Distributed Tracing (OTLP/HTTP to Alloy), Metrics Port Isolation from Public Ingress, IVISS Observability Stack, Prometheus Metrics Server (port 9091), kube-prometheus-stack ServiceMonitor Scrape Config, Layered Structured Logging (fmt + OTel), TelemetryHandle::shutdown Graceful Flush, Traces-Logs-Metrics Correlation in Grafana (+4 more)

### Community 90 - "Proactive Token Refresh Hook"
Cohesion: 0.24
Nodes (7): defaultFocusSetup(), useProactiveRefresh(), mockedPerformRefresh, SetupFn, performTokenRefresh(), getTokenExpiry(), isTokenExpired()

### Community 91 - "Root TSConfig"
Cohesion: 0.17
Nodes (11): compilerOptions, allowJs, baseUrl, noImplicitAny, noUnusedLocals, noUnusedParameters, paths, skipLibCheck (+3 more)

### Community 92 - "Common & Control DTOs"
Cohesion: 0.22
Nodes (6): queryClient, BeforeInstallPromptEvent, PWAInstallPrompt(), Toaster(), ToasterProps, AppRouter()

### Community 93 - "App Root & PWA Install Prompt"
Cohesion: 0.18
Nodes (7): ChartConfig, ChartContainer, ChartContext, ChartContextProps, ChartLegendContent, ChartTooltipContent, THEMES

### Community 94 - "Recharts Chart Wrapper"
Cohesion: 0.20
Nodes (10): Identification Modes (manual/photo/live), Frontend Layering (main/App/router/pages/components/hooks/services/openapi-rq/i18n), Frontend Test Helpers (setup.ts, createQueryWrapper, MemoryRouter), AuthProvider (React Auth Context), IVISS Design System (CSS Variables, Navy/Teal, status colors), IVISS Frontend Architecture, License Plate OCR Pipeline (react-webcam + ImageProcessor + Tesseract.js), ProtectedRoute / RequireAuth Guard (+2 more)

### Community 95 - "RBAC Middleware"
Cohesion: 0.22
Nodes (10): Deployment Modes A/B/C (local .env, Secrets Manager, CI/CD OIDC), GitHub Actions OIDC Assume Role (no static AWS keys), IVISS Coding Standards, IVISS Debugging Guide, IVISS Developer Getting Started, IVISS Project Structure, IVISS Developer Documentation Index, CI Pipeline Jobs (backend coverage/clippy/audit, frontend codegen/lint/build/Sonar) (+2 more)

### Community 96 - "Graphify Extraction Rules"
Cohesion: 0.27
Nodes (10): Partner Timeout / Unavailable Graceful Degradation, is_permanent_error, is_transient_error, NotificationJob (Otp / MagicEmail), process_notification_job, send_otp_with_retry (exponential backoff), Transient vs Permanent SMS Error Taxonomy, Permanent-to-400 / Transient-to-502 Status Mapping (+2 more)

### Community 97 - "Frontend Architecture Layering"
Cohesion: 0.33
Nodes (10): ApiSmsProvider (generic third-party API), AvlytextSmsProvider, ConsoleSmsProvider, OrangeSmsProvider, SmsProvider trait, SnsSmsProvider (AWS SNS), create_sms_provider factory, GatewayState (Arc<dyn SmsProvider>) (+2 more)

### Community 98 - "Deployment Modes & Dev Guides"
Cohesion: 0.38
Nodes (9): hash_password(), Result, String, test_hash_is_different_each_time(), test_hash_password_generates_valid_hash(), test_verify_password_correct_password(), test_verify_password_malformed_hash(), test_verify_password_wrong_password() (+1 more)

### Community 99 - "Notification Retry & Backoff"
Cohesion: 0.27
Nodes (9): best_angle_in(), estimate_skew_deg(), fixture(), plate(), Replicates ocr_service::estimate_skew_angle exactly, in PIL. Goal: check…, Perfectly level plate: one band of glyph-like blocks across the middle., The image used by test_estimate_skew_angle_*: bright bars on black., rotated() (+1 more)

### Community 100 - "SMS Provider Trait Family"
Cohesion: 0.28
Nodes (9): Deterministic Node ID Format, Native CLAUDE.md Integration, BFS Traversal Mode, Explain Node Flow, Inline NetworkX Traversal Fallback, Constrained Query Expansion, Shortest Path Between Concepts, Graph Health Check (+1 more)

### Community 101 - "Backend App State"
Cohesion: 0.22
Nodes (9): Admin Login Flow (email + password), Admin-Only RBAC, require_auth JWT Middleware, RequireAuth Route Guard (allowedRoles), Backend RBAC Enforcement Matrix (401/403/200), Admin Creates Member (Hierarchy Enforcement), argon2 Hashing on spawn_blocking, argon2 Password Hashing (+1 more)

### Community 102 - "Control List Handlers"
Cohesion: 0.22
Nodes (9): Offline Cache Fallback Behaviour, Silent Service Worker Auto-Update, Authority & Precision Color Palette, Micro-Gestalt Interactive Mechanics, Premium Layer Design Tokens (glassmorphism, gradients, shadows), Technocratic Authority Visual Identity, Auto-Reload When Connection Restored, Offline Fallback Page (+1 more)

### Community 103 - "Email Service Abstraction"
Cohesion: 0.31
Nodes (5): ProtectedRoute(), ProtectedRouteProps, RequireAuth(), mockNavigate, getRefreshToken()

### Community 104 - "Password Hashing"
Cohesion: 0.25
Nodes (5): AuthSession, mockAuthService, mockUsers, User, UserRole

### Community 105 - "Admin Login & RBAC Enforcement"
Cohesion: 0.22
Nodes (7): include, postcss.config.js, src, tailwind.config.ts, tsconfig.json, tsconfig.node.json, vitest.config.ts

### Community 106 - "UI Design System & Offline Page"
Cohesion: 0.28
Nodes (5): FeatureFlags, Default, HashMap, Self, String

### Community 107 - "Mock Auth Service"
Cohesion: 0.25
Nodes (8): Data Schema & Constants, Vehicle Interface, Cold Start Challenge (plate never cached in S3), Plate-Indexed JSON Object Model (vehicles/{PLATE}.json), Responsibility Boundary (IVISS App never touches External DB or S3), S3 Fallback Cache Strategy, Sync Server (Rust/Axum intermediary binary), IVISS Sync Server Architecture

### Community 108 - "Feature Flags"
Cohesion: 0.50
Nodes (7): PendingSubmission, OffsetDateTime, Option, String, Uuid, Value, SubmissionAuditLogRow

### Community 109 - "Graphify Query Traversal"
Cohesion: 0.29
Nodes (7): Install Banner Component, No localStorage Persistence for Banner Dismissal, PWA Installation Testing Guide, display-mode: standalone Installed Detection, PWA Meta Tags and Icons (index.html), SPA Root Mount (#root + /src/main.tsx), Crawler Allow-All Policy

### Community 110 - "S3 Vehicle Cache Sync"
Cohesion: 0.43
Nodes (7): IVISS Shield Mark (Apple Touch Icon), IVISS Shield Mark (Browser Favicon), Monochrome Shield-with-Checkmark Glyph (Safari Mask Icon), IVISS Shield Mark (Maskable PWA Icon 512x512), IVISS Visual Identity: Vehicle-in-Shield on Navy, Maskable Icon Safe-Zone Padding, Neutral Missing-Image Placeholder Graphic

### Community 111 - "Pending Submission Model"
Cohesion: 0.52
Nodes (6): setup_otp_service(), test_rate_limit_blocks_after_3_requests(), test_rate_limit_is_per_phone_number(), test_request_otp_succeeds(), test_validate_otp_no_key_fails(), test_validate_otp_wrong_code_fails()

### Community 112 - "PWA Install Testing"
Cohesion: 0.67
Nodes (5): ensure_cargo_llvm_cov(), print_error(), print_status(), print_warning(), test_flow.sh script

### Community 113 - "PWA Icon Artwork"
Cohesion: 0.60
Nodes (5): health_check(), metrics_export(), Arc, IntoResponse, State

### Community 114 - "Session Restart Queries"
Cohesion: 0.60
Nodes (5): Body, Next, Request, Response, track_metrics()

### Community 115 - "OTP Service Tests"
Cohesion: 0.33
Nodes (6): Preprocessing order fix: binarize+normalize polarity before deskew, white fill on rotation, Deskew rotation fill polarity measurement (black fill leaves dark corner artifacts), Deskew angle bias: grayscale +2.5° false vs binary +0.0° correct, Polarity detection margin measurement (5-10 points, needs central-region measurement), test_deskew_fills_corners_with_background / test_deskew_output_is_bilevel (dark-on-light fixture), test_is_light_on_dark_ignores_dark_edges / test_is_light_on_dark_detects_inverted_plate

### Community 116 - "Backend Test Flow Script"
Cohesion: 0.47
Nodes (6): CEMAC regional emblem/logo on plate, CMR country code (Cameroon), Dealer plate frame text 'TAUNUS AUTO - Mercedes-Benz und smart in Wiesbaden', BEFORE upload_3.5.jpg - Cameroon (CMR) license plate CE 568 LR photo, OCR pipeline 'BEFORE' quality/resolution test sample, Plate text 'CE 568 LR'

### Community 117 - "HTTP Metrics Middleware"
Cohesion: 0.40
Nodes (5): GRAPH_REPORT.md (fallback for broad architecture review), Graphify Knowledge Graph System, graphify query/path/explain commands, graphify update . (AST-only refresh), graphify-out/wiki/index.md navigation

### Community 118 - "GitHub Artifacts Cleanup Script"
Cohesion: 0.40
Nodes (5): lib, DOM, DOM.Iterable, ES2020, WebWorker

### Community 119 - "DB Pool & Cache Warmup"
Cohesion: 0.70
Nodes (4): delete_all_artifacts(), delete_old_artifacts(), list_artifacts(), gh-artifacts.sh script

### Community 120 - "Database Seed Data"
Cohesion: 0.70
Nodes (4): PgPool, Result, run_seed_data(), try_seed()

### Community 121 - "OpenAPI Fetch Script"
Cohesion: 0.40
Nodes (5): Cameroon national emblem and CMR country code on plate, AFTER_fixed_4.5_C5.png - post-processed Cameroon license plate image (CE 568 LR), Plate number CE 568 LR, Fixed processing parameters 4.5 / C5 (likely gamma/contrast + config variant used to binarize/clean plate image for OCR), BEFORE/AFTER OCR pipeline image comparison samples

### Community 122 - "CORS Middleware"
Cohesion: 0.50
Nodes (5): License plate number CE 568 LR, CEMAC (Central African Economic and Monetary Community) region marking, CMR country code (Cameroon), Reference plate image CE568LR, Taunus Auto / Mercedes-Benz und smart in Wiesbaden dealer stamp

### Community 123 - "Coverage Script"
Cohesion: 0.60
Nodes (4): __dirname, fallbackToLocal(), fetchFromBackend(), main()

### Community 124 - "Organization Shift Windows"
Cohesion: 0.50
Nodes (3): CorsLayer, cors_layer(), String

### Community 125 - "PWA Hook"
Cohesion: 0.50
Nodes (3): LLVM_PROFILE_FILE, RUSTFLAGS, coverage.sh script

### Community 126 - "Terraform Remote State Setup"
Cohesion: 0.50
Nodes (4): OCR pipeline BEFORE baseline sample set, Dealer sticker text: TAUNUS AUTO / Mercedes-Benz und smart in Wiesbaden, BEFORE sample license plate image (shipped_3.5_C5), License plate text CE 568 LR (Cameroon CMR)

### Community 127 - "Seed Binary"
Cohesion: 0.67
Nodes (3): Organization Work Window (start_work_time/end_work_time, UTC+1), Shift Bounds Embedded in Access Token and devices.metadata, Org-Level Shift Windows Supersede SHIFT_START_HOUR/SHIFT_END_HOUR

### Community 129 - "Metrics Registration Docs"
Cohesion: 0.67
Nodes (3): types, vite-plugin-pwa/client, vitest/globals

### Community 133 - "DB Init Script"
Cohesion: 0.67
Nodes (3): Option, String, SubmissionLocation

### Community 134 - "Remote Setup Script"
Cohesion: 0.67
Nodes (3): Decision — Phase 1 scope: resolution + performance + observability, gate becomes non-blocking coaching, Frontend quality gate measures the wrong thing (scale-dependent Laplacian variance, fail-open), Coaching restricted to photo/idle mode, dedicated capped-width preview screenshot

### Community 135 - "Conventional Commits Standard"
Cohesion: 0.67
Nodes (3): Live scan — stability rewrite: sliding majority vote instead of consecutive-match confidence threshold, Confidence semantics: never overwrite mean_text_conf from format_valid, tolerates a misread interleaved between agreeing readings (infinite-loop regression test)

### Community 136 - "ESLint Config"
Cohesion: 0.67
Nodes (3): Viewfinder — aspect 3.5→4.5, single source of truth computeViewfinderCrop, Viewfinder aspect overshoot table (3.5→+31%, 4.5→+2.1%), computeViewfinderCrop used identically by overlay and crop (viewfinder.test.ts)

### Community 137 - "PostCSS Config"
Cohesion: 0.67
Nodes (3): PSM ensemble: require agreement between PSM 7 & 13, PSM 11 only as last resort, test_agreeing_passes_report_the_stronger_confidence, test_pick_best_ensemble_skips_candidates_without_a_plate

### Community 138 - "Toast Lib Re-export"
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
- **430 isolated node(s):** `@modelcontextprotocol/server-filesystem`, `postgres-mcp`, `DATABASE_URI`, `mcp-server-git`, `@upstash/context7-mcp` (+425 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **28 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

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