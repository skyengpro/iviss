import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, useLocation } from 'react-router-dom';
import { AppRouter } from '../AppRouter';
import * as tokenManager from '@/services/auth/tokenManager';

vi.mock('@/services/auth/tokenManager', () => ({
  getAccessToken: vi.fn(),
  getRefreshToken: vi.fn(),
}));

// Helper component to spy on the current location after a redirect
const LocationSpy = () => {
  const location = useLocation();
  return <div data-testid="location-display">{location.pathname}</div>;
};

// Mock the routes to simplify testing
vi.mock('../routes', () => {
  return {
    publicRoutes: [{ path: '/public-test', component: () => <div>Public Component</div> }],
    mobileRoutes: [],
    backOfficeRoutes: [
      {
        path: '/protected-test',
        component: () => <div>Protected Component</div>,
        allowedRoles: ['admin'],
      },
      {
        path: '/redirect-test',
        component: null,
        redirectTo: '/somewhere-else',
        replace: true,
      },
    ],
    catchAllRoute: { path: '*', component: () => <div>Catch All Component</div> },
  };
});

// Mock ProtectedRoute to verify it's used
vi.mock('../ProtectedRoute', () => {
  return {
    ProtectedRoute: ({
      children,
      allowedRoles,
    }: {
      children: React.ReactNode;
      allowedRoles?: string[];
    }) => (
      <div data-testid="protected-route-wrapper" data-roles={allowedRoles?.join(',')}>
        {children}
      </div>
    ),
  };
});

describe('AppRouter', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Entry redirects (/)', () => {
    it('redirects to /activate when no tokens exist', () => {
      vi.mocked(tokenManager.getAccessToken).mockReturnValue(null);
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue(null);

      render(
        <MemoryRouter initialEntries={['/']}>
          <AppRouter />
          <LocationSpy />
        </MemoryRouter>
      );

      expect(screen.getByTestId('location-display')).toHaveTextContent('/activate');
    });

    it('redirects to /backoffice when access token exists', () => {
      vi.mocked(tokenManager.getAccessToken).mockReturnValue('access');
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue('refresh');

      render(
        <MemoryRouter initialEntries={['/']}>
          <AppRouter />
          <LocationSpy />
        </MemoryRouter>
      );

      expect(screen.getByTestId('location-display')).toHaveTextContent('/backoffice');
    });

    it('redirects to /mobile when only refresh token exists (recoverable session)', () => {
      vi.mocked(tokenManager.getAccessToken).mockReturnValue(null);
      vi.mocked(tokenManager.getRefreshToken).mockReturnValue('refresh');

      render(
        <MemoryRouter initialEntries={['/']}>
          <AppRouter />
          <LocationSpy />
        </MemoryRouter>
      );

      expect(screen.getByTestId('location-display')).toHaveTextContent('/mobile');
    });
  });

  describe('Route Rendering', () => {
    it('renders public routes without ProtectedRoute wrapper', async () => {
      render(
        <MemoryRouter initialEntries={['/public-test']}>
          <AppRouter />
        </MemoryRouter>
      );

      const component = await screen.findByText('Public Component');
      expect(component).toBeInTheDocument();
      expect(screen.queryByTestId('protected-route-wrapper')).not.toBeInTheDocument();
    });

    it('renders protected routes inside ProtectedRoute wrapper', async () => {
      render(
        <MemoryRouter initialEntries={['/protected-test']}>
          <AppRouter />
        </MemoryRouter>
      );

      const wrapper = await screen.findByTestId('protected-route-wrapper');
      expect(wrapper).toBeInTheDocument();
      expect(wrapper).toHaveAttribute('data-roles', 'admin');
      expect(screen.getByText('Protected Component')).toBeInTheDocument();
    });

    it('handles route redirects correctly within protected routes', () => {
      render(
        <MemoryRouter initialEntries={['/redirect-test']}>
          <AppRouter />
          <LocationSpy />
        </MemoryRouter>
      );

      expect(screen.getByTestId('location-display')).toHaveTextContent('/somewhere-else');
    });

    it('handles catch-all route', async () => {
      render(
        <MemoryRouter initialEntries={['/unknown-path-123']}>
          <AppRouter />
        </MemoryRouter>
      );

      const component = await screen.findByText('Catch All Component');
      expect(component).toBeInTheDocument();
    });
  });
});
