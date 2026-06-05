import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as jose from 'jose';
import generateKeyPair from '../generateKey';

// Mock jose
vi.mock('jose', () => ({
  generateKeyPair: vi.fn(),
  exportJWK: vi.fn(),
}));

describe('generateKey', () => {
  let originalCrypto: any;

  beforeEach(() => {
    vi.clearAllMocks();
    originalCrypto = global.crypto;

    // Mock Web Crypto API
    Object.defineProperty(global, 'crypto', {
      value: {
        subtle: {},
      },
      writable: true,
      configurable: true,
    });
  });

  afterEach(() => {
    // Restore global crypto
    Object.defineProperty(global, 'crypto', {
      value: originalCrypto,
      writable: true,
      configurable: true,
    });
  });

  it('generates a key pair successfully', async () => {
    const mockPublicKey = { kty: 'EC', crv: 'P-256', x: '123', y: '456' };
    const mockPrivateKey = { kty: 'EC', crv: 'P-256', d: '789', x: '123', y: '456' };

    vi.mocked(jose.generateKeyPair).mockResolvedValue({
      publicKey: {} as any,
      privateKey: {} as any,
    });

    vi.mocked(jose.exportJWK)
      .mockResolvedValueOnce(mockPublicKey) // First call exports public
      .mockResolvedValueOnce(mockPrivateKey); // Second call exports private

    const result = await generateKeyPair();

    expect(jose.generateKeyPair).toHaveBeenCalledWith('ES256', { extractable: true });
    expect(jose.exportJWK).toHaveBeenCalledTimes(2);
    expect(result).toEqual({
      publicKey: mockPublicKey,
      privateKey: mockPrivateKey,
      kid: 1,
    });
  });

  it('throws early if window.crypto.subtle is unavailable', async () => {
    // Remove subtle to simulate insecure/unsupported environment
    Object.defineProperty(global, 'crypto', {
      value: {},
      writable: true,
      configurable: true,
    });

    await expect(generateKeyPair()).rejects.toThrow(/Web Crypto API is not available/i);
    expect(jose.generateKeyPair).not.toHaveBeenCalled();
  });

  it('wraps and throws errors from jose', async () => {
    vi.mocked(jose.generateKeyPair).mockRejectedValue(new Error('JOSE execution failed'));

    await expect(generateKeyPair()).rejects.toThrow(
      /Failed to generate cryptographic keys: JOSE execution failed/i
    );
  });
});
