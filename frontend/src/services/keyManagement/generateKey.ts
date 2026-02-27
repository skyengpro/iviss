import * as jose from 'jose';

async function generateKeyPair() {
  // Check if Web Crypto API is available
  if (!window.crypto || !window.crypto.subtle) {
    throw new Error(
      'Web Crypto API is not available. This application requires HTTPS or a secure context to function properly.'
    );
  }

  try {
    // Generate an ECDSA key pair (not RSA as the comment suggests)
    const { publicKey, privateKey } = await jose.generateKeyPair('ES256', {
      extractable: true,
    });

    // Convert keys to JSON format
    const publicJWK = await jose.exportJWK(publicKey);
    const privateJWK = await jose.exportJWK(privateKey);

    // Return both keys
    const kid = 1;
    return { publicKey: publicJWK, privateKey: privateJWK, kid };
  } catch (error) {
    console.error('Key generation failed:', error);
    throw new Error(
      `Failed to generate cryptographic keys: ${error instanceof Error ? error.message : 'Unknown error'}`
    );
  }
}

export default generateKeyPair;
