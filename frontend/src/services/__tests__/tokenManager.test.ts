import { describe, it, expect, beforeEach } from 'vitest';
import {
  getAccessToken,
  setAccessToken,
  getRefreshToken,
  setRefreshToken,
  clearAccessToken,
  clearTokens,
} from '../auth/tokenManager';

describe('tokenManager', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe('access token', () => {
    it('should return null when no access token is stored', () => {
      expect(getAccessToken()).toBeNull();
    });

    it('should store and retrieve an access token', () => {
      setAccessToken('test-access-token');
      expect(getAccessToken()).toBe('test-access-token');
    });

    it('should overwrite a previously stored access token', () => {
      setAccessToken('first-token');
      setAccessToken('second-token');
      expect(getAccessToken()).toBe('second-token');
    });

    it('should clear only the access token', () => {
      setAccessToken('access');
      setRefreshToken('refresh');
      clearAccessToken();

      expect(getAccessToken()).toBeNull();
      expect(getRefreshToken()).toBe('refresh');
    });
  });

  describe('refresh token', () => {
    it('should return null when no refresh token is stored', () => {
      expect(getRefreshToken()).toBeNull();
    });

    it('should store and retrieve a refresh token', () => {
      setRefreshToken('test-refresh-token');
      expect(getRefreshToken()).toBe('test-refresh-token');
    });
  });

  describe('clearTokens', () => {
    it('should clear both access and refresh tokens', () => {
      setAccessToken('access');
      setRefreshToken('refresh');
      clearTokens();

      expect(getAccessToken()).toBeNull();
      expect(getRefreshToken()).toBeNull();
    });

    it('should not throw when tokens are already empty', () => {
      expect(() => clearTokens()).not.toThrow();
    });
  });
});
