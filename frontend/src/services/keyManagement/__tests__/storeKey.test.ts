import { describe, it, expect, vi, beforeEach } from 'vitest';
import storeKeyPair, { retrieveKeyPair } from '../storeKey';
import generateKeyPair from '../generateKey';
import { decryptPrivateKey, encryptPrivateKey } from '../encrypt';
import storage from '../storageSetup';

// Mock Dependencies
vi.mock('../generateKey', () => ({
  default: vi.fn(),
}));

vi.mock('../encrypt', () => ({
  encryptPrivateKey: vi.fn(),
  decryptPrivateKey: vi.fn(),
}));

vi.mock('../storageSetup', () => ({
  default: {
    insert: vi.fn(),
    findOne: vi.fn(),
  },
}));

describe('storeKey operations', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('storeKeyPair()', () => {
    it('calls generateKeyPair, encryptPrivateKey, and storage.insert', async () => {
      const mockKey = {
        publicKey: { kty: 'EC', crv: 'P-256' },
        privateKey: { kty: 'EC', d: 'secret' },
        kid: 1,
      };

      vi.mocked(generateKeyPair).mockResolvedValue(mockKey as any);

      const mockedEncrypted = { jwe: 'jwe.string', salt: [1, 2, 3] };
      vi.mocked(encryptPrivateKey).mockResolvedValue(mockedEncrypted);

      await storeKeyPair();

      expect(generateKeyPair).toHaveBeenCalledTimes(1);
      expect(encryptPrivateKey).toHaveBeenCalledWith(mockKey.privateKey);

      expect(storage.insert).toHaveBeenCalledWith('keys', {
        pub: mockKey.publicKey,
        priv: mockedEncrypted,
        kid: 1,
      });
    });
  });

  describe('retrieveKeyPair()', () => {
    it('calls storage.findOne and decryptPrivateKey', async () => {
      const mockStorageData = {
        value: {
          pub: { kty: 'EC', crv: 'P-256' },
          priv: { jwe: 'some-jwe', salt: [1, 2, 3] },
          kid: 1,
        },
      };

      vi.mocked(storage.findOne).mockResolvedValue(mockStorageData as any);

      const decryptedMock = { kty: 'EC', d: 'decrypted-private-key' };
      vi.mocked(decryptPrivateKey).mockResolvedValue(decryptedMock as any);

      const result = await retrieveKeyPair(1);

      expect(storage.findOne).toHaveBeenCalledWith('keys', 1);
      expect(decryptPrivateKey).toHaveBeenCalledWith(mockStorageData.value.priv);

      expect(result.publicKey).toEqual(mockStorageData.value.pub);
      expect(result.privateKey).toEqual(decryptedMock);
    });

    it('handles old `.value` wrapper vs new data shapes', async () => {
      // Missing `.value` wrapper (new shape)
      const mockStorageData = {
        pub: { kty: 'EC', crv: 'P-256' },
        priv: { jwe: 'some-jwe', salt: [1, 2, 3] },
        kid: 1,
      };

      vi.mocked(storage.findOne).mockResolvedValue(mockStorageData as any);

      const decryptedMock = { kty: 'EC', d: 'decrypted-private-key' };
      vi.mocked(decryptPrivateKey).mockResolvedValue(decryptedMock as any);

      const result = await retrieveKeyPair(1);

      expect(decryptPrivateKey).toHaveBeenCalledWith(mockStorageData.priv);
      expect(result.publicKey).toEqual(mockStorageData.pub);
      expect(result.privateKey).toEqual(decryptedMock);
    });

    it('returns { publicKey: null, privateKey: null } when no record found', async () => {
      vi.mocked(storage.findOne).mockResolvedValue(undefined as any);

      const result = await retrieveKeyPair(1);

      expect(storage.findOne).toHaveBeenCalledWith('keys', 1);
      expect(decryptPrivateKey).not.toHaveBeenCalled();

      expect(result).toEqual({ publicKey: null, privateKey: null });
    });
  });
});
