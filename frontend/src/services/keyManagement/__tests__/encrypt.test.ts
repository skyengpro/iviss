import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { PasswordManager } from '../passwordManager';
import { encryptPrivateKey, decryptPrivateKey } from '../encrypt';
import * as jose from 'jose';

// Mock Dependencies
vi.mock('../passwordManager', () => ({
  PasswordManager: {
    getPassword: vi.fn(),
  },
}));

// Mock sonner toast
vi.mock('sonner', () => ({
  toast: { error: vi.fn() },
}));

// Mock JOSE library
vi.mock('jose', () => ({
  CompactEncrypt: class {
    private plaintext: Uint8Array;
    constructor(plaintext: Uint8Array) {
      this.plaintext = plaintext;
    }
    setProtectedHeader() {
      return this;
    }
    async encrypt() {
      return 'mocked.jwe.string';
    }
  },
  compactDecrypt: vi.fn(),
}));

describe('encrypt / decrypt', () => {
  let originalCrypto: any;

  beforeEach(() => {
    vi.clearAllMocks();
    originalCrypto = global.crypto;

    // Stub Web Crypto API
    Object.defineProperty(global, 'crypto', {
      value: {
        getRandomValues: vi.fn((arr) => {
          for (let i = 0; i < arr.length; i++) arr[i] = i;
          return arr;
        }),
        subtle: {
          importKey: vi.fn().mockResolvedValue('imported-key'),
          deriveKey: vi.fn().mockResolvedValue('derived-key'),
        },
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

  describe('encryptPrivateKey', () => {
    it('returns { jwe, salt } on success', async () => {
      vi.mocked(PasswordManager.getPassword).mockResolvedValue('strong-password');

      const mockJwk: JsonWebKey = { kty: 'EC', crv: 'P-256', d: 'secret' };

      const result = await encryptPrivateKey(mockJwk);

      expect(PasswordManager.getPassword).toHaveBeenCalledTimes(1);
      expect(global.crypto.getRandomValues).toHaveBeenCalledTimes(1);
      expect(global.crypto.subtle.importKey).toHaveBeenCalledTimes(1);
      expect(global.crypto.subtle.deriveKey).toHaveBeenCalledTimes(1);

      expect(result).toHaveProperty('jwe', 'mocked.jwe.string');
      expect(result).toHaveProperty('salt');
      expect(Array.isArray(result.salt)).toBe(true);
      expect(result.salt.length).toBe(16);
    });

    it('throws when PasswordManager.getPassword returns null or empty', async () => {
      vi.mocked(PasswordManager.getPassword).mockResolvedValue('');

      const mockJwk: JsonWebKey = { kty: 'EC' };

      await expect(encryptPrivateKey(mockJwk)).rejects.toThrow('Password retrieval failed.');
      expect(global.crypto.subtle.importKey).not.toHaveBeenCalled();
    });
  });

  describe('decryptPrivateKey', () => {
    it('returns parsed JWK on success', async () => {
      vi.mocked(PasswordManager.getPassword).mockResolvedValue('strong-password');

      const mockJwkDecoded = { kty: 'EC', d: 'decrypted' };
      const plaintextBuffer = new TextEncoder().encode(JSON.stringify(mockJwkDecoded));

      vi.mocked(jose.compactDecrypt).mockResolvedValue({
        plaintext: plaintextBuffer,
        protectedHeader: { alg: 'dir', enc: 'A256GCM' },
      });

      const encryptedData = {
        jwe: 'some.valid.jwe',
        salt: Array.from(new Uint8Array(16)),
      };

      const result = await decryptPrivateKey(encryptedData);

      expect(PasswordManager.getPassword).toHaveBeenCalledTimes(1);
      expect(global.crypto.subtle.importKey).toHaveBeenCalledTimes(1);
      expect(global.crypto.subtle.deriveKey).toHaveBeenCalledTimes(1);
      expect(jose.compactDecrypt).toHaveBeenCalledWith('some.valid.jwe', 'derived-key');

      expect(result).toEqual(mockJwkDecoded);
    });

    it('throws on invalid JWE format', async () => {
      vi.mocked(PasswordManager.getPassword).mockResolvedValue('strong-password');

      const encryptedData = {
        jwe: null as any, // Invalid format
        salt: Array.from(new Uint8Array(16)),
      };

      await expect(decryptPrivateKey(encryptedData)).rejects.toThrow('Invalid JWE format');
      expect(global.crypto.subtle.deriveKey).toHaveBeenCalled();
      expect(jose.compactDecrypt).not.toHaveBeenCalled();
    });

    it('throws when password retrieval fails', async () => {
      vi.mocked(PasswordManager.getPassword).mockResolvedValue('');

      const encryptedData = {
        jwe: 'some.valid.jwe',
        salt: Array.from(new Uint8Array(16)),
      };

      await expect(decryptPrivateKey(encryptedData)).rejects.toThrow('Password retrieval failed.');
      expect(global.crypto.subtle.importKey).not.toHaveBeenCalled();
    });
  });
});
