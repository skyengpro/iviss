import checkKeyPairExists from './checkKeyPairExists';
import storeKeyPair, { retrieveKeyPair } from './storeKey';

export async function KeyManagement() {
  const keyPairExists = await checkKeyPairExists();

  // No key pair yet — legitimate first-run case, safe to generate one.
  if (!keyPairExists) {
    await storeKeyPair();
    const { publicKey, privateKey } = await retrieveKeyPair(1);
    if (!publicKey || !privateKey) {
      throw new Error('Failed to generate new key pair.');
    }
    return { publicKey, privateKey };
  }

  // A key pair exists — retrieval failing here (transient IndexedDB error,
  // missing decryption password, etc.) does NOT mean the keys are corrupt.
  // Regenerating would create a new key pair the backend never sees the
  // public half of (only the activation flow re-enrolls it), permanently
  // desynchronizing this device and forcing a full re-activation. Propagate
  // the error instead so the caller can retry later with the same identity.
  const { publicKey, privateKey } = await retrieveKeyPair(1);
  if (!publicKey || !privateKey) {
    throw new Error('Failed to retrieve key pair.');
  }
  return { publicKey, privateKey };
}
