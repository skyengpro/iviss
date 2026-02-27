/**
 * Service to manage unique device identification.
 * Ensures each device has a persistent UUID for security and binding purposes.
 */

const DEVICE_ID_KEY = 'iviss_device_id';

/**
 * Returns the unique device ID for this browser instance.
 * Generates and persists a new one if it doesn't already exist.
 */
export function getDeviceId(): string {
  let deviceId = localStorage.getItem(DEVICE_ID_KEY);

  if (!deviceId) {
    deviceId = crypto.randomUUID();
    localStorage.setItem(DEVICE_ID_KEY, deviceId);
  }

  return deviceId;
}

/**
 * Reset the device ID. Useful for testing or when clear and re-enrollment is required.
 */
export function resetDeviceId(): void {
  localStorage.removeItem(DEVICE_ID_KEY);
}
