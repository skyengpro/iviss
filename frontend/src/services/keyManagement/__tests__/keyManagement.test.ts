import { describe, it, expect, vi, beforeEach } from 'vitest';
import { KeyManagement } from '../keyManagement';
import checkKeyPairExists from '../checkKeyPairExists';
import storeKeyPair, { retrieveKeyPair } from '../storeKey';

// Mock Dependencies
vi.mock('../checkKeyPairExists', () => ({
  default: vi.fn(),
}));

vi.mock('../storeKey', () => ({
  default: vi.fn(),
  retrieveKeyPair: vi.fn(),
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

  it('throws when generation still yields null keys (first run)', async () => {
    vi.mocked(checkKeyPairExists).mockResolvedValue(false);
    vi.mocked(storeKeyPair).mockResolvedValue(undefined);
    vi.mocked(retrieveKeyPair).mockResolvedValue({ publicKey: null, privateKey: null });

    await expect(KeyManagement()).rejects.toThrow('Failed to generate new key pair.');
  });

  it('propagates the error instead of wiping IndexedDB when retrieval fails for an existing key pair', async () => {
    // A key pair exists, but retrieving it fails (transient IndexedDB error,
    // missing decryption password, etc). Regenerating here would desync the
    // device from the backend, which still has the OLD public key — so the
    // error must propagate untouched, not trigger a silent wipe+regenerate.
    vi.mocked(checkKeyPairExists).mockResolvedValue(true);
    vi.mocked(retrieveKeyPair).mockRejectedValue(new Error('IDB transaction aborted'));

    await expect(KeyManagement()).rejects.toThrow('IDB transaction aborted');
    expect(storeKeyPair).not.toHaveBeenCalled();
  });

  it('throws without regenerating when an existing key pair resolves to null', async () => {
    vi.mocked(checkKeyPairExists).mockResolvedValue(true);
    vi.mocked(retrieveKeyPair).mockResolvedValue({ publicKey: null, privateKey: null });

    await expect(KeyManagement()).rejects.toThrow('Failed to retrieve key pair.');
    expect(storeKeyPair).not.toHaveBeenCalled();
  });
});
