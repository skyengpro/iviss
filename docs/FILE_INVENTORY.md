# IVISS File Inventory

> **Purpose**: Complete file listing with size, purpose, and status

---

## Source Files Summary

| Category | Files | Total Size |
|----------|-------|------------|
| Pages | 14 | ~70 KB |
| UI Components | 51 | ~95 KB |
| Layout Components | 7 | ~17 KB |
| Services | 4 | ~33 KB |
| Hooks | 2 | ~4.5 KB |
| Contexts | 1 | ~4 KB |
| Utils | 1 | ~7 KB |
| Config | 8 | ~12 KB |

---

## Pages (`src/pages/`)

### Auth
| File | Lines | Status | Notes |
|------|-------|--------|-------|
| `Login.tsx` | 274 | ✅ Used | Role-based login form |

### Mobile
| File | Lines | Status | Notes |
|------|-------|--------|-------|
| `MobileDashboard.tsx` | 243 | ✅ Used | Agent home screen |
| `MobileSearch.tsx` | 98 | ✅ Used | Manual plate entry |
| `MobileScan.tsx` | 565 | ⚠️ Large | Camera + OCR, needs splitting |
| `MobileHistory.tsx` | 250 | ⚠️ Inline data | Has local mock array |
| `MobileProfile.tsx` | 127 | ✅ Used | User profile |
| `MobileVehicleResult.tsx` | 468 | ⚠️ Large | Vehicle details display |
| `MobileCarteGrise.tsx` | 271 | ✅ Used | Registration lookup |

### Back Office
| File | Lines | Status | Notes |
|------|-------|--------|-------|
| `BackOfficeDashboard.tsx` | 277 | ⚠️ Inline data | Charts with hardcoded data |
| `ControlHistory.tsx` | 268 | ⚠️ Inline data | Has local mock array |
| `ControlDetail.tsx` | 268 | ✅ Used | Control details view |
| `UserManagement.tsx` | 301 | ⚠️ Inline data | Has local mock array |
| `PendingVehicles.tsx` | 316 | ✅ Used | Approval workflow |

### Other
| File | Lines | Status | Notes |
|------|-------|--------|-------|
| `NotFound.tsx` | 21 | ✅ Used | 404 page |

---

## Components (`src/components/`)

### Layout (`components/layout/`)
| File | Lines | Status | Notes |
|------|-------|--------|-------|
| `MobileLayout.tsx` | 42 | ✅ Used | Mobile wrapper |
| `MobileHeader.tsx` | 52 | ✅ Used | Mobile top bar |
| `MobileNavigation.tsx` | 56 | ✅ Used | Bottom nav |
| `MobileSidebar.tsx` | 115 | ✅ Used | Mobile drawer |
| `BackOfficeLayout.tsx` | 34 | ✅ Used | Desktop wrapper |
| `BackOfficeHeader.tsx` | 53 | ✅ Used | Desktop header |
| `BackOfficeSidebar.tsx` | 166 | ✅ Used | Desktop sidebar |

### Vehicle (`components/vehicle/`)
| File | Lines | Status | Notes |
|------|-------|--------|-------|
| `PlateInput.tsx` | 98 | ✅ Used | Plate number input |
| `VehicleStatusCard.tsx` | 186 | ✅ Used | Status display |

### UI (`components/ui/`) - shadcn/ui
| Component | Status | Notes |
|-----------|--------|-------|
| `accordion.tsx` | ❓ Verify | May be unused |
| `alert-dialog.tsx` | ✅ Used | |
| `alert.tsx` | ✅ Used | |
| `aspect-ratio.tsx` | ❓ Verify | May be unused |
| `avatar.tsx` | ✅ Used | User avatars |
| `badge.tsx` | ✅ Used | Status badges |
| `breadcrumb.tsx` | ❓ Verify | May be unused |
| `button.tsx` | ✅ Used | Core component |
| `calendar.tsx` | ❓ Verify | May be unused |
| `card.tsx` | ✅ Used | Core component |
| `carousel.tsx` | ❓ Verify | May be unused |
| `chart.tsx` | ✅ Used | Dashboard charts |
| `checkbox.tsx` | ✅ Used | |
| `collapsible.tsx` | ❓ Verify | May be unused |
| `command.tsx` | ❓ Verify | May be unused |
| `context-menu.tsx` | ❓ Verify | May be unused |
| `dialog.tsx` | ✅ Used | Modals |
| `drawer.tsx` | ✅ Used | Mobile drawer |
| `dropdown-menu.tsx` | ✅ Used | Action menus |
| `form.tsx` | ✅ Used | Form wrapper |
| `hover-card.tsx` | ❓ Verify | May be unused |
| `input-otp.tsx` | ❓ Verify | May be unused |
| `input.tsx` | ✅ Used | Core component |
| `label.tsx` | ✅ Used | Form labels |
| `menubar.tsx` | ❓ Verify | May be unused |
| `navigation-menu.tsx` | ❓ Verify | May be unused |
| `pagination.tsx` | ❓ Verify | Tables have custom |
| `popover.tsx` | ✅ Used | |
| `progress.tsx` | ✅ Used | Scan progress |
| `radio-group.tsx` | ❓ Verify | May be unused |
| `resizable.tsx` | ❓ Verify | May be unused |
| `scroll-area.tsx` | ✅ Used | Scrollable areas |
| `select.tsx` | ✅ Used | Dropdowns |
| `separator.tsx` | ✅ Used | |
| `sheet.tsx` | ✅ Used | Mobile sidebar |
| `sidebar.tsx` | ✅ Used | shadcn sidebar |
| `skeleton.tsx` | ✅ Used | Loading states |
| `slider.tsx` | ❓ Verify | May be unused |
| `sonner.tsx` | ✅ Used | Toast wrapper |
| `stat-card.tsx` | ✅ Custom | Dashboard stats |
| `status-badge.tsx` | ✅ Custom | Status indicator |
| `switch.tsx` | ✅ Used | |
| `table.tsx` | ✅ Used | Data tables |
| `tabs.tsx` | ✅ Used | Tab navigation |
| `textarea.tsx` | ✅ Used | |
| `toast.tsx` | ✅ Used | Notifications |
| `toaster.tsx` | ✅ Used | Toast container |
| `toggle-group.tsx` | ❓ Verify | May be unused |
| `toggle.tsx` | ❓ Verify | May be unused |
| `tooltip.tsx` | ✅ Used | |
| `use-toast.ts` | ⚠️ Duplicate | Also in hooks/ |

---

## Services (`src/services/`)

| File | Lines | Exports | Notes |
|------|-------|---------|-------|
| `mockAuth.ts` | 170 | `mockAuthService`, types | 3 hardcoded users |
| `mockVehicles.ts` | 313 | `mockVehicleService`, types | 6 mock vehicles |
| `mockControls.ts` | 323 | `mockControlService`, types | 5 mock controls |
| `mockExternalAPIs.ts` | 322 | `mockExternalAPIService`, types | 4 API simulations |

---

## Hooks (`src/hooks/`)

| File | Lines | Export | Notes |
|------|-------|--------|-------|
| `use-mobile.tsx` | 17 | `useIsMobile()` | Media query hook |
| `use-toast.ts` | 114 | `useToast()`, `toast` | Toast state management |

---

## Utils (`src/utils/`)

| File | Lines | Export | Notes |
|------|-------|--------|-------|
| `imageProcessor.ts` | 214 | `ImageProcessor` class | OCR preprocessing |

---

## Configuration Files

| File | Purpose | Notes |
|------|---------|-------|
| `package.json` | Dependencies | Name needs change |
| `vite.config.ts` | Build config | Port 8080, SWC |
| `tsconfig.json` | TS config | Relaxed settings |
| `tsconfig.app.json` | App TS config | |
| `tsconfig.node.json` | Node TS config | |
| `tailwind.config.ts` | Tailwind theme | Custom design tokens |
| `postcss.config.js` | PostCSS | |
| `eslint.config.js` | ESLint | |
| `components.json` | shadcn config | |
| `vitest.config.ts` | Test config | |

---

## Static Files (`public/`)

| File | Purpose |
|------|---------|
| `_redirects` | Netlify SPA routing |
| `placeholder.svg` | Placeholder image |
| `robots.txt` | SEO config |

---

## Files to Delete (After Verification)

```
# Potentially unused UI components
src/components/ui/accordion.tsx
src/components/ui/carousel.tsx
src/components/ui/context-menu.tsx
src/components/ui/hover-card.tsx
src/components/ui/input-otp.tsx
src/components/ui/menubar.tsx
src/components/ui/navigation-menu.tsx
src/components/ui/slider.tsx
src/components/ui/toggle.tsx
src/components/ui/toggle-group.tsx

# Duplicate file
src/components/ui/use-toast.ts  (keep src/hooks/use-toast.ts)

# Possibly unused
src/App.css  (verify if any styles are used)
```
