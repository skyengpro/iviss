import { describe, it, expect, beforeEach } from 'vitest';
import { mockAuthService } from '../mock/mockAuth';

describe('mockAuthService', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe('login', () => {
    it('should return a session on valid credentials', async () => {
      const result = await mockAuthService.login('agent01', 'agent123');

      expect(result.success).toBe(true);
      expect(result.session).toBeDefined();
      expect(result.session!.user.username).toBe('agent01');
      expect(result.session!.user.role).toBe('agent');
      expect(result.session!.token).toBeDefined();
      expect(result.session!.token.startsWith('tok_')).toBe(true);
    });

    it('should fail with unknown username', async () => {
      const result = await mockAuthService.login('nonexistent', 'password');

      expect(result.success).toBe(false);
      expect(result.error).toContain('User not found');
      expect(result.session).toBeUndefined();
    });

    it('should fail with wrong password', async () => {
      const result = await mockAuthService.login('agent01', 'wrong-password');

      expect(result.success).toBe(false);
      expect(result.error).toContain('Incorrect password');
    });

    it('should be case-insensitive for username', async () => {
      const result = await mockAuthService.login('Agent01', 'agent123');

      expect(result.success).toBe(true);
      expect(result.session!.user.username).toBe('agent01');
    });

    it('should store session in localStorage on success', async () => {
      await mockAuthService.login('agent01', 'agent123');

      const stored = localStorage.getItem('iviss_session');
      expect(stored).not.toBeNull();

      const parsed = JSON.parse(stored!);
      expect(parsed.user.username).toBe('agent01');
    });
  });

  describe('logout', () => {
    it('should clear session from localStorage', async () => {
      await mockAuthService.login('agent01', 'agent123');
      expect(localStorage.getItem('iviss_session')).not.toBeNull();

      await mockAuthService.logout();
      expect(localStorage.getItem('iviss_session')).toBeNull();
    });
  });

  describe('getSession', () => {
    it('should return null when no session exists', () => {
      expect(mockAuthService.getSession()).toBeNull();
    });

    it('should return the stored session', async () => {
      await mockAuthService.login('admin01', 'admin123');
      const session = mockAuthService.getSession();

      expect(session).not.toBeNull();
      expect(session!.user.role).toBe('admin');
    });

    it('should return null when localStorage contains invalid JSON', () => {
      localStorage.setItem('iviss_session', 'not-valid-json');
      expect(mockAuthService.getSession()).toBeNull();
    });
  });

  describe('isAuthenticated', () => {
    it('should return false when no session exists', () => {
      expect(mockAuthService.isAuthenticated()).toBe(false);
    });

    it('should return true after login', async () => {
      await mockAuthService.login('agent01', 'agent123');
      expect(mockAuthService.isAuthenticated()).toBe(true);
    });
  });

  describe('getCurrentUser', () => {
    it('should return null when not authenticated', () => {
      expect(mockAuthService.getCurrentUser()).toBeNull();
    });

    it('should return the current user after login', async () => {
      await mockAuthService.login('supervisor01', 'supervisor123');
      const user = mockAuthService.getCurrentUser();

      expect(user).not.toBeNull();
      expect(user!.role).toBe('supervisor');
    });
  });

  describe('getAllUsers', () => {
    it('should return all mock users', async () => {
      const users = await mockAuthService.getAllUsers();
      expect(users.length).toBe(3);

      const roles = users.map((u) => u.role);
      expect(roles).toContain('agent');
      expect(roles).toContain('supervisor');
      expect(roles).toContain('admin');
    });
  });

  describe('getMockCredentials', () => {
    it('should return credentials for all 3 roles', () => {
      const creds = mockAuthService.getMockCredentials();
      expect(creds).toHaveLength(3);

      const roles = creds.map((c) => c.role);
      expect(roles).toContain('Agent');
      expect(roles).toContain('Supervisor');
      expect(roles).toContain('Admin');
    });
  });
});
