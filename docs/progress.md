# Progress: Admin User Management

## Phase 1: API & Data Layer
- [x] API Analysis (provisionUser, listUsers, etc. found)
- [x] Run `npm run codegen`
- [x] Create `useUsers.ts` hook

## Phase 2: UI Components
- [x] Create `UserForm.tsx` (supports Add/Edit)
- [x] Add i18n keys for User Management

## Phase 3: Page Assembly
- [x] Integrate `UserForm` into `UserManagement.tsx`
- [x] Implement User List (Read)
- [x] Implement User Provisioning (Create)
- [x] Implement User Update (Update)
- [x] Implement User Status Toggle (Update)
- [x] Implement User Deletion (Delete) with AlertDialog

## Phase 4: Polish
- [x] Resolve lint errors (Loader2, Fragments, Rename)
- [x] Fix backend schema mismatch (is_active vs status)
  - [x] Update `UserProfile` DTO
  - [x] Update SQL queries in `user_queries.rs`
- [x] Refine User Status UI
  - [x] Handle enum-based status in `UserManagement.tsx`
  - [x] Update `StatusBadge` variants
- [x] Update i18n keys for status
- [x] Sanitize User Form Inputs
  - [x] Update `types.gen.ts` for frontend type safety
  - [x] Default phone number to `+237`
  - [x] Enforce numeric-only phone input
  - [x] Add regex validation for full name (letters only)
- [x] Finalize walkthrough
