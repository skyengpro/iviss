# Walkthrough — Admin User Management Frontend

I have implemented the frontend functionality for the Administrator User Management, specifically focusing on the "Add User" (Provisioning) feature and replacing mock data with real API integration.

## Changes Made

### 1. Data Layer (`frontend/src/hooks/api/useUsers.ts`)
- Created a new custom hook `useUsers` that wraps the generated OpenAPI hooks.
- Implemented `provision`, `update`, and `remove` methods with automatic cache invalidation using `queryClient.invalidateQueries`.
- Added `useUser` for individual user details.

### 2. UI Components (`frontend/src/components/shared/Admin/UserForm.tsx`)
- Created a polymorphic `UserForm` component that supports both **Creation** and **Editing**.
- Used `react-hook-form` with `zod` for strict validation.
- Integrated with `shadcn/ui` components (Input, Select, Button, Form).
- Supports localized validation messages.

### 3. Page Integration (`frontend/src/pages/backoffice/UserManagement.tsx`)
- **Retrieval**: Replaced mock data with the new `useUsers` hook for a live user list.
- **Create**: Added a "Add User" button that opens the `UserForm`.
- **Update**: 
  - Added an "Edit" action in the row menu that pre-fills the `UserForm` with `initialData`.
  - Implemented an "Activate/Deactivate" toggle in the row menu using the `update` mutation.
- **Delete**: Integrated an `AlertDialog` for safe deletion, wired to the `remove` mutation.
- Enhanced the user table with real data, search, and filtering.

### 4. Internationalization (`frontend/src/i18n/locales/`)
- Added comprehensive translation keys for the user management feature in both English (`en.json`) and French (`fr.json`).

## Verification Plan

### Automated
- Verified TypeScript types against the generated OpenAPI client.
- Fixed Zod-to-API mapping issues.
- Resolved all lint errors (missing imports, JSX fragments).

### Manual
- **User List**: Verified live data retrieval.
- **Add User**: Verified user provisioning and list refresh.
- **Edit User**: Verified profile updates.
- **Toggle Status**: Verified immediate UI feedback using cached data invalidation.
- **Delete User**: Verified deletion with confirmation safeguard.
