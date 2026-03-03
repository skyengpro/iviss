import { decryptPrivateKey, encryptPrivateKey } from './encrypt';
import generateKeyPair from './generateKey'; // Import your existing key generation function
import storage from './storageSetup'; // Import the initialized storage

// Function to store a key pair in IndexedDB
export async function storeKeyPair() {
  const { publicKey, privateKey } = await generateKeyPair();

  const encryptedPriv = await encryptPrivateKey(privateKey);

  await storage.insert('keys', {
    pub: publicKey,
    priv: encryptedPriv,
    kid: 1,
  });
}

// Function to retrieve the key pair from IndexedDB
export async function retrieveKeyPair(kid: number) {
  const retrievedRecord = await storage.findOne('keys', kid);

  if (retrievedRecord) {
    // Handle both old and new data structures
    const data = retrievedRecord.value || retrievedRecord;
    const { pub: publicKey, priv: encryptedPriv } = data as {
      pub: JsonWebKey;
      priv: { jwe: string; salt: number[] };
      kid: number;
    };

    const privateKey = await decryptPrivateKey(encryptedPriv);

    return { publicKey, privateKey };
  } else {
    return { publicKey: null, privateKey: null };
  }
}

export default storeKeyPair;
