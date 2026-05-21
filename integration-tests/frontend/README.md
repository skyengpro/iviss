# Frontend Integration Tests

Integration tests for the IVISS frontend application.

## Overview

These tests verify that frontend components correctly interact with:
- Backend APIs
- State management (React Query)
- Browser APIs (Camera, Storage, Service Workers)
- Routing and navigation

## Test Structure

```
frontend/
├── auth-flow.test.ts          # Authentication integration
├── vehicle-search.test.ts     # Vehicle search workflow
├── photo-capture.test.ts      # Camera and OCR integration
├── offline-mode.test.ts       # PWA offline functionality
└── README.md                  # This file
```

## Running Tests

```bash
# From project root
cd frontend

# Run all tests
npm run test

# Run integration tests only
npm run test:integration

# Watch mode
npm run test:watch

# With coverage
npm run coverage
```

## Writing Tests

### Test Template

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryWrapper } from '@/test/queryWrapper';
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

// Mock API server
const server = setupServer(
  http.post('/api/v1/auth/admin/login', () => {
    return HttpResponse.json({
      access_token: 'mock-token',
      refresh_token: 'mock-refresh',
    });
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('Feature Integration', () => {
  it('should complete user workflow', async () => {
    const user = userEvent.setup();
    
    render(
      <QueryWrapper>
        <MyComponent />
      </QueryWrapper>
    );

    // Interact with component
    await user.type(screen.getByLabelText('Email'), 'test@example.com');
    await user.click(screen.getByRole('button', { name: 'Submit' }));

    // Assert results
    await waitFor(() => {
      expect(screen.getByText('Success')).toBeInTheDocument();
    });
  });
});
```

## Test Scenarios

### 1. Authentication Flow (TODO)

**File**: `auth-flow.test.ts`

- Login with valid credentials
- Login with invalid credentials
- Token refresh on expiration
- Logout and session cleanup
- Redirect after login

### 2. Vehicle Search (TODO)

**File**: `vehicle-search.test.ts`

- Search by plate number
- Display vehicle details
- Handle API errors
- Show loading states
- Cache search results

### 3. Photo Capture (TODO)

**File**: `photo-capture.test.ts`

- Request camera permission
- Capture photo
- Process with OCR
- Extract plate number
- Handle OCR errors

### 4. Offline Mode (TODO)

**File**: `offline-mode.test.ts`

- Service worker registration
- Cache API responses
- Work offline
- Sync when online
- Show offline indicator

## Mocking

### API Mocking with MSW

```typescript
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

const server = setupServer(
  http.get('/api/v1/vehicles/:plate', ({ params }) => {
    return HttpResponse.json({
      plate_number: params.plate,
      brand: 'Toyota',
      model: 'Camry',
    });
  })
);
```

### Browser API Mocking

```typescript
// Mock camera API
Object.defineProperty(navigator, 'mediaDevices', {
  value: {
    getUserMedia: vi.fn().mockResolvedValue({
      getTracks: () => [],
    }),
  },
});

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  clear: vi.fn(),
};
global.localStorage = localStorageMock as any;
```

## Best Practices

✅ **DO:**
- Use MSW for API mocking
- Test user interactions, not implementation
- Use semantic queries (getByRole, getByLabelText)
- Wait for async operations with waitFor
- Clean up after each test

❌ **DON'T:**
- Test implementation details
- Use setTimeout for waiting
- Query by class names or IDs
- Share state between tests
- Mock React Query directly

## Debugging

### View Rendered Output

```typescript
import { screen, debug } from '@testing-library/react';

it('test', () => {
  render(<Component />);
  screen.debug(); // Prints DOM to console
});
```

### Check Queries

```typescript
// See all available queries
screen.logTestingPlaygroundURL();
```

### Run Single Test

```bash
npm run test -- auth-flow.test.ts
```

## Resources

- [Testing Library Docs](https://testing-library.com/docs/react-testing-library/intro/)
- [Vitest Docs](https://vitest.dev/)
- [MSW Docs](https://mswjs.io/)
- [React Query Testing](https://tanstack.com/query/latest/docs/framework/react/guides/testing)
