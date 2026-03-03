import storage from './keyManagement/storageSetup';

/**
 * Service to manage unique device identification.
 * Ensures each device has a persistent UUID for security and binding purposes.
 * Stored in IndexedDB for consistency with cryptographic keys.
 */

const DEVICE_ID_KEY = 'device_id';

/**
 * Returns the unique device ID for this browser instance.
 * Generates and persists a new one in IndexedDB if it doesn't already exist.
 */
export async function getDeviceId(): Promise<string> {
  try {
    const record = (await storage.findOne('metadata', DEVICE_ID_KEY)) as
      | {
          key: string;
          value: string;
        }
      | undefined;

    if (record?.value) {
      return record.value;
    }

    // Fallback/Migration: Check localStorage in case it was created there previously
    const legacyId = localStorage.getItem(`iviss_${DEVICE_ID_KEY}`);
    const deviceId = legacyId || crypto.randomUUID();

    // Persist to IndexedDB
    await storage.insert('metadata', { key: DEVICE_ID_KEY, value: deviceId });

    // Clean up legacy storage
    if (legacyId) {
      localStorage.removeItem(`iviss_${DEVICE_ID_KEY}`);
    }

    return deviceId;
  } catch (error) {
    console.error('Failed to access device identity in IndexedDB:', error);
    // Absolute fallback to hardware UUID or random (less persistent if IDB fails)
    return crypto.randomUUID();
  }
}

/**
 * Reset the device ID. Useful for testing or when clear and re-enrollment is required.
 */
export async function resetDeviceId(): Promise<void> {
  try {
    await storage.clear('metadata');
    localStorage.removeItem(`iviss_${DEVICE_ID_KEY}`);
  } catch (error) {
    console.error('Failed to reset device identity:', error);
  }
}
