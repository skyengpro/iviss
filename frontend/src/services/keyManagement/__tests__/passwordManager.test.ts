import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { PasswordManager } from '../passwordManager';

describe('PasswordManager', () => {
  let originalCrypto: any;

  beforeEach(() => {
    vi.clearAllMocks();
    sessionStorage.clear();
    localStorage.clear();

    originalCrypto = global.crypto;

    // Mock Web Crypto API
    Object.defineProperty(global, 'crypto', {
      value: {
        getRandomValues: vi.fn((array: Uint8Array) => {
          // Fill with dummy data for testing length and generation
          for (let i = 0; i < array.length; i++) {
            array[i] = i;
          }
          return array;
        }),
      },
      writable: true,
      configurable: true,
    });
  });

  afterEach(() => {
    Object.defineProperty(global, 'crypto', {
      value: originalCrypto,
      writable: true,
      configurable: true,
    });
  });

  it('returns password from sessionStorage if present', async () => {
    sessionStorage.setItem('password', 'session-password-123');

    const result = await PasswordManager.getPassword();

    expect(result).toBe('session-password-123');
    // Ensure crypto was not called to generate new one
    expect(global.crypto.getRandomValues).not.toHaveBeenCalled();
  });

  it('falls back to localStorage and copies to sessionStorage', async () => {
    localStorage.setItem('password', 'local-password-456');

    const result = await PasswordManager.getPassword();

    expect(result).toBe('local-password-456');
    expect(sessionStorage.getItem('password')).toBe('local-password-456');
    expect(global.crypto.getRandomValues).not.toHaveBeenCalled();
  });

  it('generates and stores new password when neither storage has one', async () => {
    const result = await PasswordManager.getPassword();

    // Since our mock fills Uint8Array with 0,1,2..., generating base64 of it
    // We just want to check it generated something and stored it
    expect(result).toBeDefined();
    expect(result.length).toBe(32); // slice(0, 32)

    expect(sessionStorage.getItem('password')).toBe(result);
    expect(localStorage.getItem('password')).toBe(result);

    expect(global.crypto.getRandomValues).toHaveBeenCalledTimes(1);
  });
});
