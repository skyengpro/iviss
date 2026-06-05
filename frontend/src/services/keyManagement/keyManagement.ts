import checkKeyPairExists from './checkKeyPairExists';
import { clearAllStoredData } from './storageSetup';
import storeKeyPair, { retrieveKeyPair } from './storeKey';

export async function KeyManagement() {
  try {
    const keyPairExists = await checkKeyPairExists();
    if (!keyPairExists) {
      await storeKeyPair();
    }
    const { publicKey, privateKey } = await retrieveKeyPair(1);
    if (!publicKey || !privateKey) {
      throw new Error('Failed to retrieve key pair.');
    }
    return { publicKey, privateKey };
  } catch (error) {
    // Clear all stored data
    try {
      await clearAllStoredData();
    } catch (clearError) {
      // Silently handle clear error
    }

    // Generate new key pair
    await storeKeyPair();
    const { publicKey, privateKey } = await retrieveKeyPair(1);

    if (!publicKey || !privateKey) {
      throw new Error('Failed to generate new key pair.');
    }

    return { publicKey, privateKey };
  }
}
