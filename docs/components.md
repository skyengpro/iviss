# Component Guide

This document explains the main components in the IVISS application and what they do.

## Backend Components

### Authentication (handlers/auth.rs)

Handles user login and device activation:
- activate: First-time device setup with activation code
- request_daily_login: Agent requests OTP code for daily shift
- verify_daily_login: Verifies OTP and starts shift
- refresh: Renews expired access tokens

### Vehicle Search (handlers/search_vehicle.rs)

Looks up vehicles by license plate number and returns compliance status (insurance, technical inspection, stolen flag). Automatically creates a control record when a vehicle is checked.

### OCR Processing (handlers/scan.rs, handlers/photo.rs)

Processes license plate images using Tesseract OCR:
- scan.rs: Handles live camera scans
- photo.rs: Handles uploaded photos

Returns the detected plate number and confidence score.

### Control Records (handlers/list_control.rs)

Manages vehicle inspection logs:
- Lists all control records with filtering
- Creates new control records
- Tracks GPS location, timestamp, and results

### User Management (handlers/user_management.rs)

Admin functions for managing users:
- Create, update, delete users
- Terminate or restart agent sessions
- Resend activation codes
- List users by organization

### Organization Management (handlers/organization_management.rs)

Admin functions for managing organizations:
- Create, update, delete organizations
- Each organization has isolated data

### Statistics (handlers/stats.rs)

Dashboard data:
- Control counts by time period
- Top performing agents
- Recent alerts
- Activity feed

### Audit Log (handlers/audit.rs)

Complete audit trail:
- Lists all system actions
- Exports to CSV
- Tracks who did what and when

## Frontend Components

### Layout Components

#### MobileLayout
The main container for agent mobile screens. Includes header with user info and bottom navigation bar.

#### BackOfficeLayout
The main container for admin/manager screens. Includes sidebar navigation and header.

### Authentication Pages

#### Activate (pages/auth/Activate.tsx)
First-time device setup. Agent enters badge ID and activation code to register their device.

#### DailyLogin (pages/auth/DailyLogin.tsx)
Daily shift login. Agent enters badge ID, requests OTP via SMS, then verifies the code to start their shift.

#### AdminLogin (pages/auth/AdminLogin.tsx)
Email and password login for administrators and managers.

### Mobile Pages

#### MobileDashboard
Agent home screen showing:
- Today's control count
- Quick actions (scan, search)
- Recent controls

#### MobileScan
Camera interface for scanning license plates using OCR. Shows live preview and detected plate number.

#### MobileSearch
Manual plate number entry. Agent types the plate and searches the database.

#### MobileVehicleResult
Shows vehicle details and compliance status after a search:
- Vehicle information (brand, model, color)
- Insurance status
- Technical inspection status
- Stolen/wanted status
- Option to log control record

#### MobileHistory
List of all controls performed by the agent, with filters by date and status.

### Back-Office Pages

#### BackOfficeDashboard
Admin/manager home screen showing:
- Statistics (total controls, alerts, active agents)
- Charts (control activity over time)
- Recent activity feed

#### ControlHistory
Complete list of all control records across all agents (filtered by organization for managers). Includes search and date filters.

#### UserManagement
Admin interface for managing users:
- Create new users
- Edit user details
- Deactivate accounts
- Terminate sessions
- Resend activation codes

#### OrganizationManagement
Admin interface for managing organizations:
- Create new organizations
- Edit organization details
- View organization members

#### PendingVehicles
Queue of gray card (vehicle registration) submissions from agents waiting for admin review and approval.

#### AuditLogPage
Complete audit trail of all system actions with CSV export.

### Shared Components

#### VehicleStatusCard (components/vehicle/VehicleStatusCard.tsx)
Displays compliance status for one aspect of a vehicle (insurance, technical, stolen). Shows status badge (valid/warning/critical) and details.

#### PlateInput (components/vehicle/PlateInput.tsx)
Formatted input field for license plate numbers. Validates format and shows visual feedback.

#### ControlActivityChart (components/dashboard/ControlActivityChart.tsx)
Line chart showing control activity over time. Used in dashboards.

#### LiveControlMap (components/dashboard/LiveControlMap.tsx)
Map showing recent control locations with markers.

#### ErrorBoundary (components/shared/ErrorBoundary.tsx)
Catches React errors and shows user-friendly error message instead of crashing the app.

#### AppInitializer (components/shared/AppInitializer.tsx)
Runs on app startup to check authentication status and redirect to appropriate page.

### UI Components (components/ui/)

These are reusable atomic components from shadcn/ui:
- Button, Input, Card, Badge, Alert
- Dialog, Drawer, Sheet (modals and overlays)
- Table, Form, Select, Checkbox
- Toast, Tooltip, Progress
- And 40+ more

All styled with Tailwind CSS and support dark mode.

## Services

### JWT Service (services/jwt_service.rs)
Creates and verifies JWT access tokens using RS256 algorithm. Tokens include user ID, device ID, role, and shift times.

### OTP Service (services/otp_service.rs)
Generates 6-digit OTP codes, stores them in Redis with 5-minute expiration, and handles rate limiting (max 3 requests per 10 minutes).

### OCR Service (services/ocr_service.rs)
Processes images using Tesseract to extract license plate text. Handles image preprocessing (grayscale, contrast adjustment).

### Vehicle Service (services/vehicle_service.rs)
Builds vehicle information responses by combining data from the database and external status checks.

## Middleware

### Authentication (middleware/auth.rs)
Validates JWT tokens on every protected request. Checks if token is valid, not expired, and shift hasn't ended.

### Authorization (middleware/rbac.rs)
Enforces role-based access control. Ensures only admins can access admin routes.

### Shift Hours (middleware/agent_work_scope.rs)
Restricts OTP requests to configured shift hours (default 6 AM - 10 PM UTC+1).

## How Components Work Together

### Vehicle Check Flow
1. Agent opens MobileScan or MobileSearch
2. Plate number is captured (OCR or manual entry)
3. Frontend calls search_vehicle API endpoint
4. Backend queries database for vehicle info
5. Backend creates control record automatically
6. Frontend shows MobileVehicleResult with status
7. Agent can add notes or actions

### Daily Login Flow
1. Agent opens DailyLogin page
2. Enters badge ID and requests OTP
3. Backend generates code and sends SMS
4. Agent enters code from SMS
5. Backend verifies code and creates tokens
6. Frontend stores tokens and redirects to MobileDashboard

### Admin User Management Flow
1. Admin opens UserManagement page
2. Clicks "Create User" button
3. Fills out UserForm with details
4. Frontend calls provision_user API endpoint
5. Backend creates user and sends activation code
6. Admin gives activation code to new user
7. New user activates device with Activate page
