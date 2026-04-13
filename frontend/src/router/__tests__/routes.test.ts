import { describe, it, expect } from 'vitest';
import { publicRoutes, mobileRoutes, backOfficeRoutes, catchAllRoute } from '../routes';

describe('route configuration', () => {
  describe('publicRoutes', () => {
    it('should include /activate and /daily-login', () => {
      const paths = publicRoutes.map((r) => r.path);
      expect(paths).toContain('/activate');
      expect(paths).toContain('/daily-login');
    });

    it('should not require any roles', () => {
      publicRoutes.forEach((route) => {
        expect(route.allowedRoles).toBeUndefined();
      });
    });

    it('should have a non-null component for navigable routes', () => {
      const navigableRoutes = publicRoutes.filter((r) => !r.redirectTo);
      navigableRoutes.forEach((route) => {
        expect(route.component).not.toBeNull();
      });
    });

    it('should have redirect routes with a redirectTo target', () => {
      const redirectRoutes = publicRoutes.filter((r) => r.redirectTo);
      expect(redirectRoutes.length).toBeGreaterThan(0);
      redirectRoutes.forEach((route) => {
        expect(route.redirectTo).toBeDefined();
      });
    });
  });

  describe('mobileRoutes', () => {
    it('should all require agent, manager, or admin roles', () => {
      mobileRoutes.forEach((route) => {
        expect(route.allowedRoles).toBeDefined();
        expect(route.allowedRoles).toContain('agent');
        expect(route.allowedRoles).toContain('manager');
        expect(route.allowedRoles).toContain('admin');
      });
    });

    it('should all start with /mobile', () => {
      mobileRoutes.forEach((route) => {
        expect(route.path).toMatch(/^\/mobile/);
      });
    });

    it('should have a non-null component for each route', () => {
      mobileRoutes.forEach((route) => {
        expect(route.component).not.toBeNull();
      });
    });
  });

  describe('backOfficeRoutes', () => {
    it('should all require admin or manager roles', () => {
      backOfficeRoutes.forEach((route) => {
        expect(route.allowedRoles).toBeDefined();
        route.allowedRoles!.forEach((role) => {
          expect(['admin', 'manager', 'org_admin']).toContain(role);
        });
      });
    });

    it('should all start with /backoffice', () => {
      backOfficeRoutes.forEach((route) => {
        expect(route.path).toMatch(/^\/backoffice/);
      });
    });

    it('should have a non-null component for each route', () => {
      backOfficeRoutes.forEach((route) => {
        expect(route.component).not.toBeNull();
      });
    });
  });

  describe('catchAllRoute', () => {
    it('should have path "*"', () => {
      expect(catchAllRoute.path).toBe('*');
    });

    it('should have a non-null component', () => {
      expect(catchAllRoute.component).not.toBeNull();
    });
  });
});
