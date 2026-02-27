# Design: Admin User Management (Add User)

## Overview
Implement the "Add User" feature for administrators to provision new accounts.

## User Flow
1. Admin navigates to **User Management** page.
2. Admin clicks the **Add User** button.
3. A modal opens with the **Add User Form**.
4. Admin fills in:
   - Username
   - Full Name
   - Phone Number
   - Role (Admin, Manager, Agent)
   - Organization (Select from list)
   - Email (Optional)
   - Badge ID (Optional)
5. Admin clicks **Save**.
6. The system sends a `POST /admin/users` request.
7. System displays success toast and closes modal.
8. System refreshes the user list.

## Components
### [NEW] `AddUserForm.tsx`
- **Purpose**: Generic form component for creating users.
- **Library**: `react-hook-form` + `zod` + `shadcn/ui`.
- **Validation**:
  - `username`: Required, min 3 chars.
  - `fullName`: Required.
  - `phoneNumber`: Required, valid format.
  - `role`: Required.
  - `organizationId`: Required.

### [MODIFY] `UserManagement.tsx`
- Integrate `AddUserForm` into a `Dialog`.
- Replace `mockAuthService` with `useUsers` hook.

## Data Layer
### [NEW] `useUsers.ts`
- **Purpose**: Bridge between generated OpenAPI hooks and UI.
- **Actions**:
  - `listUsers()`
  - `provisionUser(data)`
  - `updateUser(id, data)`
  - `deleteUser(id)`

## i18n
Add keys for the new form fields and validation messages to `backOfficeUserManagement` namespace.
