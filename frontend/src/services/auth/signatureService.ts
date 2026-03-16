/**
 * Signature Service
 *
 * Provides cryptographic nonce signing for the device-bound
 * token refresh challenge-response flow.
 *
 * Uses the ECDSA ES256 private key stored in IndexedDB to create
 * a compact JWS (JSON Web Signature) of the backend-provided nonce.
 */

import * as jose from 'jose';
import { retrieveKeyPair } from '../keyManagement/storeKey';

/**
 * Signs a nonce string using the device's ES256 private key.
 *
 * @param nonce - The nonce challenge string received from the backend
 * @returns A compact JWS string containing the signed nonce
 * @throws If the private key cannot be retrieved or signing fails
 */
export async function signNonce(nonce: string): Promise<string> {
  const { privateKey: privateJwk } = await retrieveKeyPair(1);

  if (!privateJwk) {
    throw new Error('Device private key not found. Device may need to be re-enrolled.');
  }

  // Import the JWK as a CryptoKey for signing
  const privateKey = await jose.importJWK(privateJwk, 'ES256');

  // Create a compact JWS with the nonce as the payload
  // Use Uint8Array.from() to ensure strict Uint8Array type (required by jose)
  const payload = Uint8Array.from(new TextEncoder().encode(nonce));
  const jws = await new jose.CompactSign(payload)
    .setProtectedHeader({ alg: 'ES256' })
    .sign(privateKey);

  return jws;
}
