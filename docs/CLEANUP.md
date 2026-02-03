# IVISS Cleanup Checklist

> **Purpose**: Issues to fix before production deployment

---

## 🔴 Critical Issues

### 1. Authentication is Mock-Only
**Location**: `src/services/mockAuth.ts`, `src/contexts/AuthContext.tsx`

**Issue**: Hardcoded credentials in source code
```typescript
// mockAuth.ts - line 27-76
const mockUsers = {
  agent01: { password: 'agent123', ... },
  supervisor01: { password: 'supervisor123', ... },
  admin01: { password: 'admin123', ... },
};
```

**Action**: Replace with real backend authentication (Keycloak, Auth0, or custom JWT)

---

### 2. TypeScript Strictness Disabled
**Location**: `tsconfig.json`

```json
{
  "noImplicitAny": false,      // ❌ Should be true
  "strictNullChecks": false,   // ❌ Should be true
  "noUnusedParameters": false, // ⚠️ Consider true
  "noUnusedLocals": false      // ⚠️ Consider true
}
```

**Action**: Enable strict mode gradually

---

### 3. No Real Tests
**Location**: `src/test/example.test.ts`

```typescript
it("should pass", () => {
  expect(true).toBe(true); // Placeholder only
});
```

**Action**: Add tests for OCR, auth flows, and API services

---

## 🟡 Important Issues

### 4. Placeholder Routes
**Location**: `src/App.tsx` (lines 145-175)

These routes all render `BackOfficeDashboard` instead of dedicated pages:
- `/backoffice/vehicles` 
- `/backoffice/organizations`
- `/backoffice/audit`
- `/backoffice/settings`

**Action**: Either implement or remove from navigation

---

### 5. Duplicate Mock Data
**Location**: Multiple files have inline mock data

| File | Lines | Issue |
|------|-------|-------|
| `MobileHistory.tsx` | 18-64 | `mockControls` array |
| `ControlHistory.tsx` | 33-95 | `mockControls` array |
| `UserManagement.tsx` | 43-95 | `mockUsers` array |
| `BackOfficeDashboard.tsx` | 12-51 | Multiple `const` arrays |

**Action**: Centralize all mock data in `services/` or create `constants/mockData.ts`

---

### 6. Large Page Components (>300 lines)
| File | Lines | Recommendation |
|------|-------|----------------|
| `MobileScan.tsx` | 565 | Extract: CameraView, ScanResult, LiveScanner |
| `MobileVehicleResult.tsx` | 468 | Extract: DetailItem, StatusCard to shared |
| `BackOfficeDashboard.tsx` | 277 | Extract chart and table sections |

---

### 7. Duplicate Files
**Issue**: `use-toast.ts` exists in two locations
- `src/hooks/use-toast.ts` (main)
- `src/components/ui/use-toast.ts` (re-export)

**Action**: Keep only one, update imports

---

### 8. Unused shadcn Components (Verify)
Components installed but potentially unused:
- `accordion.tsx`
- `carousel.tsx`
- `context-menu.tsx`
- `hover-card.tsx`
- `input-otp.tsx`
- `menubar.tsx`
- `navigation-menu.tsx`
- `slider.tsx`
- `toggle.tsx`
- `toggle-group.tsx`

**Action**: Run grep to verify, delete unused

---

### 9. package.json Name
```json
{
  "name": "vite_react_shadcn_ts"  // ❌ Generic template name
}
```

**Action**: Rename to `"iviss"` or `"iviss-frontend"`

---

### 10. Missing `.env.example`
No environment variable template exists.

**Action**: Create `.env.example`:
```env
VITE_API_URL=
VITE_AUTH_URL=
VITE_ENABLE_MOCK=true
```

---

## 🟢 Minor Issues

### 11. Console Logs in Production Code
**Location**: `src/utils/imageProcessor.ts`
```typescript
console.log('Preprocessed image...'); // line 53
console.log('Calculated threshold:', threshold); // line 139
```

**Action**: Remove or wrap with `import.meta.env.DEV`

---

### 12. Hardcoded Strings
**Locations**:
- Coordinates: `48.8566, 2.3522` (Paris) in multiple files
- Location: `"Highway A1, KM 42"` hardcoded
- French text mixed with English

**Action**: Move to constants file if kept, or use i18n

---

### 13. Non-Functional UI Elements
| Component | Element | Issue |
|-----------|---------|-------|
| `ControlHistory.tsx` | "More Filters" button | No handler |
| `ControlHistory.tsx` | "Date Range" button | No handler |
| `MobileHistory.tsx` | "Export to PDF" button | No handler |
| All tables | Pagination | Static, non-functional |

---

### 14. Missing Error Boundaries
No React error boundaries exist.

**Action**: Add `ErrorBoundary` wrapper component

---

## Cleanup Commands

```bash
# Find unused shadcn components
for comp in accordion carousel context-menu hover-card input-otp menubar navigation-menu slider toggle; do
  echo "=== $comp ==="
  grep -rn "$comp" src/ --include="*.tsx" | grep -v "ui/$comp"
done

# Find console.log statements
grep -rn "console\." src/ --include="*.ts" --include="*.tsx"

# Find TODO comments
grep -rn "TODO\|FIXME\|XXX" src/

# Check TypeScript errors with strict mode
npx tsc --noEmit --strict 2>&1 | head -50
```

---

## Recommended Cleanup Order

1. **First** - Fix package.json name
2. **Then** - Remove duplicate `use-toast.ts`
3. **Then** - Verify/delete unused shadcn components
4. **Then** - Centralize mock data
5. **Then** - Remove console.logs
6. **Then** - Create `.env.example`
7. **Later** - Split large page components
8. **Later** - Enable TypeScript strict mode
9. **Later** - Add real tests
10. **Final** - Implement or remove placeholder routes
