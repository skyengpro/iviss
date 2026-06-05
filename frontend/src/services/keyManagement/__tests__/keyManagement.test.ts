import { describe, it, expect, vi, beforeEach } from 'vitest';
import { KeyManagement } from '../keyManagement';
import checkKeyPairExists from '../checkKeyPairExists';
import storeKeyPair, { retrieveKeyPair } from '../storeKey';
import { clearAllStoredData } from '../storageSetup';

// Mock Dependencies
vi.mock('../checkKeyPairExists', () => ({
  default: vi.fn(),
}));

vi.mock('../storeKey', () => ({
  default: vi.fn(),
  retrieveKeyPair: vi.fn(),
}));

vi.mock('../storageSetup', () => ({
  clearAllStoredData: vi.fn(),
}));

describe('keyManagement orchestration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns existing key pair when checkKeyPairExists returns true', async () => {
    vi.mocked(checkKeyPairExists).mockResolvedValue(true);

    const mockKeys = { publicKey: { kty: 'EC' }, privateKey: { kty: 'EC', d: 'd' } };
    vi.mocked(retrieveKeyPair).mockResolvedValue(mockKeys as any);

    const result = await KeyManagement();

    expect(checkKeyPairExists).toHaveBeenCalledTimes(1);
    expect(storeKeyPair).not.toHaveBeenCalled();
    expect(retrieveKeyPair).toHaveBeenCalledTimes(1);
    expect(result).toEqual(mockKeys);
  });

  it('stores a new key pair when checkKeyPairExists returns false', async () => {
    vi.mocked(checkKeyPairExists).mockResolvedValue(false);

    const mockKeys = { publicKey: { kty: 'EC' }, privateKey: { kty: 'EC', d: 'd' } };
    vi.mocked(retrieveKeyPair).mockResolvedValue(mockKeys as any);
    vi.mocked(storeKeyPair).mockResolvedValue(undefined);

    const result = await KeyManagement();

    expect(checkKeyPairExists).toHaveBeenCalledTimes(1);
    expect(storeKeyPair).toHaveBeenCalledTimes(1);
    expect(retrieveKeyPair).toHaveBeenCalledTimes(1);
    expect(result).toEqual(mockKeys);
  });

  it('clears all data and generates a new pair on error', async () => {
    // Make checkKeyPairExists fail to trigger the catch block
    vi.mocked(checkKeyPairExists).mockRejectedValue(new Error('Corrupted DB'));

    vi.mocked(clearAllStoredData).mockResolvedValue(undefined);
    vi.mocked(storeKeyPair).mockResolvedValue(undefined);

    const mockKeys = { publicKey: { kty: 'EC' }, privateKey: { kty: 'EC' } };
    vi.mocked(retrieveKeyPair).mockResolvedValue(mockKeys as any);

    const result = await KeyManagement();

    expect(checkKeyPairExists).toHaveBeenCalledTimes(1);
    expect(clearAllStoredData).toHaveBeenCalledTimes(1);
    expect(storeKeyPair).toHaveBeenCalledTimes(1);
    expect(retrieveKeyPair).toHaveBeenCalledTimes(1);

    expect(result).toEqual(mockKeys);
  });

  it('throws if after error-recovery the retrieved keys are still null', async () => {
    vi.mocked(checkKeyPairExists).mockRejectedValue(new Error('Corrupted DB'));
    vi.mocked(clearAllStoredData).mockResolvedValue(undefined);
    vi.mocked(storeKeyPair).mockResolvedValue(undefined);

    vi.mocked(retrieveKeyPair).mockResolvedValue({ publicKey: null, privateKey: null });

    await expect(KeyManagement()).rejects.toThrow('Failed to generate new key pair.');
  });
});
