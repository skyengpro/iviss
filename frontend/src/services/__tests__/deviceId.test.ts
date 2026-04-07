import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the storage module before importing the module under test
vi.mock('../keyManagement/storageSetup', () => ({
  default: {
    findOne: vi.fn(),
    insert: vi.fn(),
    clear: vi.fn(),
  },
}));

import storage from '../keyManagement/storageSetup';
import { getDeviceId, resetDeviceId } from '../device/deviceId';

const mockedStorage = vi.mocked(storage);

describe('deviceId', () => {
  const FIXED_UUID = '550e8400-e29b-41d4-a716-446655440000';

  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    vi.stubGlobal('crypto', {
      randomUUID: vi.fn().mockReturnValue(FIXED_UUID),
    });
  });

  describe('getDeviceId', () => {
    it('should return existing device ID from IndexedDB', async () => {
      mockedStorage.findOne.mockResolvedValue({
        key: 'device_id',
        value: 'existing-device-id',
      });

      const id = await getDeviceId();
      expect(id).toBe('existing-device-id');
      expect(mockedStorage.insert).not.toHaveBeenCalled();
    });

    it('should generate and persist a new device ID when none exists', async () => {
      mockedStorage.findOne.mockResolvedValue(undefined);

      const id = await getDeviceId();

      expect(id).toBe(FIXED_UUID);
      expect(mockedStorage.insert).toHaveBeenCalledWith('metadata', {
        key: 'device_id',
        value: FIXED_UUID,
      });
    });

    it('should migrate legacy localStorage device ID to IndexedDB', async () => {
      const legacyId = 'legacy-device-id-123';
      localStorage.setItem('iviss_device_id', legacyId);
      mockedStorage.findOne.mockResolvedValue(undefined);

      const id = await getDeviceId();

      expect(id).toBe(legacyId);
      expect(mockedStorage.insert).toHaveBeenCalledWith('metadata', {
        key: 'device_id',
        value: legacyId,
      });
      // Legacy entry should be cleaned up
      expect(localStorage.getItem('iviss_device_id')).toBeNull();
    });

    it('should fallback to crypto.randomUUID when IndexedDB fails', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      mockedStorage.findOne.mockRejectedValue(new Error('IndexedDB unavailable'));

      const id = await getDeviceId();

      expect(id).toBe(FIXED_UUID);
      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });

  describe('resetDeviceId', () => {
    it('should clear metadata store and localStorage', async () => {
      localStorage.setItem('iviss_device_id', 'some-id');
      mockedStorage.clear.mockResolvedValue(undefined);

      await resetDeviceId();

      expect(mockedStorage.clear).toHaveBeenCalledWith('metadata');
      expect(localStorage.getItem('iviss_device_id')).toBeNull();
    });

    it('should not throw when clear fails', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      mockedStorage.clear.mockRejectedValue(new Error('clear failed'));

      await expect(resetDeviceId()).resolves.not.toThrow();
      consoleSpy.mockRestore();
    });
  });
});
