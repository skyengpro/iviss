# IVISS Frontend Architecture

> **Document**: Complete architecture overview for IVISS (Intelligent Vehicle Identification & Security System)

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Build** | Vite 5.4 + SWC |
| **Framework** | React 18.3 |
| **Language** | TypeScript 5.8 |
| **Styling** | Tailwind CSS 3.4 + CSS Variables |
| **UI** | shadcn/ui (Radix primitives) |
| **State** | React Query + Context |
| **Routing** | React Router DOM 6.30 |
| **Forms** | React Hook Form + Zod |
| **OCR** | Tesseract.js 7.0 |
| **Camera** | react-webcam |

---

## Folder Structure

```
src/
├── components/           # Reusable UI components
│   ├── layout/           # Page layouts (Mobile, BackOffice)
│   ├── ui/               # shadcn/ui atomic components (51 files)
│   └── vehicle/          # Domain-specific components
├── contexts/             # React Context (AuthContext)
├── hooks/                # Custom hooks
├── lib/                  # Utility libraries (cn helper)
├── pages/                # Page components
│   ├── auth/             # Login
│   ├── backoffice/       # Admin dashboard, controls, users
│   └── mobile/           # Agent mobile interface
├── services/             # Mock API services
├── test/                 # Test setup & tests
└── utils/                # Utility functions (imageProcessor)
```

---

## Application Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         App.tsx                                 │
│  QueryClientProvider → TooltipProvider → BrowserRouter          │
│                              │                                  │
│                        AuthProvider                             │
│                              │                                  │
│              ┌───────────────┼───────────────┐                  │
│              ▼               ▼               ▼                  │
│         /login         /mobile/*       /backoffice/*            │
│           │                 │                │                  │
│         Login         MobileLayout    BackOfficeLayout          │
│                            │                │                   │
│                    ┌───────┴───────┐   ┌────┴────┐              │
│                    │ MobileHeader  │   │ Sidebar │              │
│                    │ MobileNav     │   │ Header  │              │
│                    │ {children}    │   │{children}│             │
│                    └───────────────┘   └─────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

---

## Route Map

### Public Routes
| Path | Component | Description |
|------|-----------|-------------|
| `/` | → `/login` | Redirect to login |
| `/login` | `Login` | Authentication page |

### Mobile (Agent/Supervisor)
| Path | Component | Description |
|------|-----------|-------------|
| `/mobile` | `MobileDashboard` | Agent home |
| `/mobile/search` | `MobileSearch` | Manual plate entry |
| `/mobile/scan` | `MobileScan` | Camera OCR |
| `/mobile/history` | `MobileHistory` | Control history |
| `/mobile/profile` | `MobileProfile` | User profile |
| `/mobile/vehicle/:plate` | `MobileVehicleResult` | Vehicle details |
| `/mobile/carte-grise` | `MobileCarteGrise` | Registration lookup |

### Back Office (Admin/Supervisor)
| Path | Component | Description |
|------|-----------|-------------|
| `/backoffice` | `BackOfficeDashboard` | Admin dashboard |
| `/backoffice/controls` | `ControlHistory` | All controls |
| `/backoffice/controls/:id` | `ControlDetail` | Control details |
| `/backoffice/users` | `UserManagement` | User CRUD |
| `/backoffice/validation` | `PendingVehicles` | Pending approvals |

---

## Authentication System

**Current**: Mock authentication stored in `localStorage`

```typescript
// User Roles
type UserRole = 'agent' | 'supervisor' | 'admin';

// Role → Route Access
agent:      /mobile/*
supervisor: /mobile/* + /backoffice (limited)
admin:      /backoffice/* (full access)
```

**Components**:
- `AuthProvider` - Context wrapper
- `useAuth()` - Hook for auth state
- `RequireAuth` - Route guard HOC

---

## Design System

### CSS Variables (index.css)
- **Colors**: Navy primary, Teal accent
- **Status**: valid (green), warning (orange), critical (red), pending (gray)
- **Palettes**: navy-50 to navy-900, teal-50 to teal-600
- **Dark Mode**: Full support via `.dark` class

### Component Categories
1. **Atomic** (`ui/`): Button, Input, Card, Badge, etc.
2. **Layout** (`layout/`): MobileLayout, BackOfficeLayout, Headers, Sidebars
3. **Domain** (`vehicle/`): PlateInput, VehicleStatusCard

---

## Data Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Page       │ ──► │   Service    │ ──► │   Mock Data  │
│  Component   │ ◄── │  (async)     │ ◄── │   (arrays)   │
└──────────────┘     └──────────────┘     └──────────────┘
       │
       ▼
┌──────────────┐
│  UI State    │
│ (useState)   │
└──────────────┘
```

**Services**:
- `mockAuth.ts` - Login/logout/session
- `mockVehicles.ts` - Vehicle database
- `mockControls.ts` - Control logging
- `mockExternalAPIs.ts` - Insurance/Police/Customs

---

## Key Features

### 1. License Plate OCR
- **Camera**: `react-webcam` for capture
- **Processing**: `ImageProcessor` class (grayscale, contrast)
- **Recognition**: `Tesseract.js` (configured for Cameroon plates)
- **Format**: `XX ### XX` (e.g., "CE 128 BC")

### 2. Vehicle Status Check
- Aggregates 4 API calls in parallel
- Returns overall status: valid | warning | critical
- Displays cards for each subsystem

### 3. Control Logging
- Records: plate, agent, location, timestamp, status
- Supports actions: check, flag, citation, impound, release
- Statistics: today, week, month, alerts

---

## External Dependencies

### UI (Radix-based)
All 27 `@radix-ui/*` packages are used by shadcn/ui components.

### Core Libraries
| Package | Purpose |
|---------|---------|
| `@tanstack/react-query` | Server state |
| `react-hook-form` | Form management |
| `zod` | Validation |
| `lucide-react` | Icons |
| `recharts` | Charts |
| `date-fns` | Date formatting |

### Feature Libraries
| Package | Purpose |
|---------|---------|
| `tesseract.js` | OCR engine |
| `react-webcam` | Camera capture |
| `sonner` | Toast notifications |
