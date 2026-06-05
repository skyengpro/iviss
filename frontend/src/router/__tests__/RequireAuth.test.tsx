import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { RequireAuth } from '../RequireAuth';

// ─── Mock dependencies ────────────────────────────────────────────────────────
const mockNavigate = vi.fn();

vi.mock('react-router-dom', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router-dom')>();
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

vi.mock('@/services/auth/tokenManager', () => ({
  getAccessToken: vi.fn(),
  getRefreshToken: vi.fn(),
}));

vi.mock('@/hooks/auth/use-auth', () => ({
  useAuth: vi.fn(),
}));

import { useAuth } from '@/hooks/auth/use-auth';
import * as tokenManager from '@/services/auth/tokenManager';

// ─── Helper ───────────────────────────────────────────────────────────────────
function renderWithRouter(
  ui: React.ReactElement,
  { initialEntries = ['/protected'] }: { initialEntries?: string[] } = {}
) {
  return render(
    <MemoryRouter initialEntries={initialEntries}>
      <Routes>
        <Route path="/protected" element={ui} />
        <Route path="/activate" element={<div>Activate Page</div>} />
        <Route path="/daily-login" element={<div>Daily Login Page</div>} />
        <Route path="/backoffice" element={<div>Back Office</div>} />
        <Route path="/mobile" element={<div>Mobile Page</div>} />
      </Routes>
    </MemoryRouter>
  );
}

// ─── Tests ────────────────────────────────────────────────────────────────────
describe('RequireAuth', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    vi.mocked(tokenManager.getAccessToken).mockReturnValue(null);
    vi.mocked(tokenManager.getRefreshToken).mockReturnValue(null);
  });

  it('shows loading spinner when isLoading is true', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: true,
      isAuthenticated: false,
      user: null,
    } as never);

    renderWithRouter(
      <RequireAuth>
        <div>Protected Content</div>
      </RequireAuth>
    );

    expect(screen.getByText('Loading...')).toBeInTheDocument();
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument();
  });

  it('renders null (no navigate) while auth state is loading', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: true,
      isAuthenticated: false,
      user: null,
    } as never);

    renderWithRouter(
      <RequireAuth>
        <div>Protected Content</div>
      </RequireAuth>
    );

    expect(mockNavigate).not.toHaveBeenCalled();
  });

  it('navigates to /activate when no token and device not activated', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: false,
      isAuthenticated: false,
      user: null,
    } as never);
    vi.mocked(tokenManager.getAccessToken).mockReturnValue(null);
    vi.mocked(tokenManager.getRefreshToken).mockReturnValue(null);
    localStorage.removeItem('iviss_device_activated');

    renderWithRouter(
      <RequireAuth>
        <div>Protected Content</div>
      </RequireAuth>
    );

    expect(mockNavigate).toHaveBeenCalledWith('/activate', expect.any(Object));
  });

  it('navigates to /daily-login when device is activated but no token', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: false,
      isAuthenticated: false,
      user: null,
    } as never);
    vi.mocked(tokenManager.getAccessToken).mockReturnValue(null);
    vi.mocked(tokenManager.getRefreshToken).mockReturnValue(null);
    localStorage.setItem('iviss_device_activated', 'true');

    renderWithRouter(
      <RequireAuth>
        <div>Protected Content</div>
      </RequireAuth>
    );

    expect(mockNavigate).toHaveBeenCalledWith('/daily-login', expect.any(Object));
  });

  it('navigates to /daily-login when refresh token exists but no access token', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: false,
      isAuthenticated: false,
      user: null,
    } as never);
    vi.mocked(tokenManager.getAccessToken).mockReturnValue(null);
    vi.mocked(tokenManager.getRefreshToken).mockReturnValue('some-refresh-token');

    renderWithRouter(
      <RequireAuth>
        <div>Protected Content</div>
      </RequireAuth>
    );

    expect(mockNavigate).toHaveBeenCalledWith('/daily-login', expect.any(Object));
  });

  it('renders children when authenticated with no role restriction', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: false,
      isAuthenticated: true,
      user: { id: 'u1', role: 'agent' },
    } as never);

    renderWithRouter(
      <RequireAuth>
        <div>Protected Content</div>
      </RequireAuth>
    );

    expect(screen.getByText('Protected Content')).toBeInTheDocument();
    expect(mockNavigate).not.toHaveBeenCalled();
  });

  it('renders children when role is in allowedRoles', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: false,
      isAuthenticated: true,
      user: { id: 'u1', role: 'agent' },
    } as never);

    renderWithRouter(
      <RequireAuth allowedRoles={['agent', 'manager']}>
        <div>Agent Content</div>
      </RequireAuth>
    );

    expect(screen.getByText('Agent Content')).toBeInTheDocument();
    expect(mockNavigate).not.toHaveBeenCalled();
  });

  it('navigates to /backoffice when admin role is not in allowedRoles', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: false,
      isAuthenticated: true,
      user: { id: 'u1', role: 'admin' },
    } as never);

    renderWithRouter(
      <RequireAuth allowedRoles={['agent']}>
        <div>Agent Only</div>
      </RequireAuth>
    );

    expect(mockNavigate).toHaveBeenCalledWith('/backoffice');
  });

  it('navigates to /mobile when agent role is not allowed', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: false,
      isAuthenticated: true,
      user: { id: 'u1', role: 'agent' },
    } as never);

    renderWithRouter(
      <RequireAuth allowedRoles={['admin', 'manager']}>
        <div>Admin Only</div>
      </RequireAuth>
    );

    expect(mockNavigate).toHaveBeenCalledWith('/mobile');
  });

  it('does NOT navigate again if admin is already at /backoffice', () => {
    vi.mocked(useAuth).mockReturnValue({
      isLoading: false,
      isAuthenticated: true,
      user: { id: 'u1', role: 'admin' },
    } as never);

    render(
      <MemoryRouter initialEntries={['/backoffice']}>
        <Routes>
          <Route
            path="/backoffice"
            element={
              <RequireAuth allowedRoles={['agent']}>
                <div>Back Office</div>
              </RequireAuth>
            }
          />
        </Routes>
      </MemoryRouter>
    );

    expect(mockNavigate).not.toHaveBeenCalled();
  });
});
