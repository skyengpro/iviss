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

    it('should have a non-null component for each route', () => {
      publicRoutes.forEach((route) => {
        expect(route.component).not.toBeNull();
      });
    });
  });

  describe('mobileRoutes', () => {
    it('should all require agent or supervisor roles', () => {
      mobileRoutes.forEach((route) => {
        expect(route.allowedRoles).toBeDefined();
        expect(route.allowedRoles).toContain('agent');
        expect(route.allowedRoles).toContain('supervisor');
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
    it('should all require admin or supervisor roles', () => {
      backOfficeRoutes.forEach((route) => {
        expect(route.allowedRoles).toBeDefined();
        route.allowedRoles!.forEach((role) => {
          expect(['admin', 'supervisor']).toContain(role);
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
