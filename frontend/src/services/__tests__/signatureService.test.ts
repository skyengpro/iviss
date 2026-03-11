import { describe, it, expect, vi, beforeEach } from 'vitest';
import { webcrypto } from 'node:crypto';
import { signNonce } from '../auth/signatureService';

// Mock the storeKey module
vi.mock('../keyManagement/storeKey', () => ({
  retrieveKeyPair: vi.fn(),
}));

import { retrieveKeyPair } from '../keyManagement/storeKey';

const mockedRetrieveKeyPair = vi.mocked(retrieveKeyPair);

// Generate a real ES256 key pair for testing
async function generateTestKeyPair() {
  const keyPair = await webcrypto.subtle.generateKey({ name: 'ECDSA', namedCurve: 'P-256' }, true, [
    'sign',
    'verify',
  ]);

  const publicJwk = await webcrypto.subtle.exportKey('jwk', keyPair.publicKey);
  const privateJwk = await webcrypto.subtle.exportKey('jwk', keyPair.privateKey);

  return { publicKey: publicJwk, privateKey: privateJwk };
}

describe('signatureService', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Ensure WebCrypto is available for jose in Node test runtime
    globalThis.crypto = webcrypto as unknown as Crypto;
  });

  describe('signNonce', () => {
    it('should produce a valid compact JWS string', async () => {
      const { publicKey, privateKey } = await generateTestKeyPair();
      mockedRetrieveKeyPair.mockResolvedValue({ publicKey, privateKey });

      const nonce = 'test-nonce-12345';
      const jws = await signNonce(nonce);

      // Compact JWS has 3 base64url-encoded parts separated by dots
      expect(jws).toBeDefined();
      expect(typeof jws).toBe('string');
      const parts = jws.split('.');
      expect(parts).toHaveLength(3);
    });

    it('should produce different signatures for different nonces', async () => {
      const { publicKey, privateKey } = await generateTestKeyPair();
      mockedRetrieveKeyPair.mockResolvedValue({ publicKey, privateKey });

      const jws1 = await signNonce('nonce-aaa');
      const jws2 = await signNonce('nonce-bbb');

      // The signatures (3rd part) should differ
      expect(jws1.split('.')[2]).not.toBe(jws2.split('.')[2]);
    });

    it('should include ES256 algorithm in the protected header', async () => {
      const { publicKey, privateKey } = await generateTestKeyPair();
      mockedRetrieveKeyPair.mockResolvedValue({ publicKey, privateKey });

      const jws = await signNonce('test-nonce');
      const headerPart = jws.split('.')[0];

      // Decode the base64url header
      const header = JSON.parse(atob(headerPart.replace(/-/g, '+').replace(/_/g, '/')));
      expect(header.alg).toBe('ES256');
    });

    it('should throw when private key is not found', async () => {
      mockedRetrieveKeyPair.mockResolvedValue({ publicKey: null, privateKey: null });

      await expect(signNonce('test-nonce')).rejects.toThrow('Device private key not found');
    });

    it('should throw when key retrieval fails', async () => {
      mockedRetrieveKeyPair.mockRejectedValue(new Error('IndexedDB unavailable'));

      await expect(signNonce('test-nonce')).rejects.toThrow('IndexedDB unavailable');
    });
  });
});
