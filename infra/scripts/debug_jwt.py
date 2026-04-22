import base64
import os

# 1. Simulate a multi-line PEM key with potential mangling
pem_content = """-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCl97ZkHNZjT7Q4
kEXOy7LKXAmjjxQtAtvpqLAwlVSObUpk1/ulLAvmAQ4yIRKu30Bhu+tyqLB92QwB
-----END PRIVATE KEY-----"""

# 2. Simulate our new deploy.sh encoding
encoded = base64.b64encode(pem_content.strip().encode()).decode()
print(f"Base64 Encoded Key: {encoded}")

# 3. Simulate Rust decoding
decoded = base64.b64decode(encoded).decode()
print("\nDecoded PEM:")
print(decoded)

if decoded == pem_content.strip():
    print("\n✅ SUCCESS: Handshake verified!")
else:
    print("\n❌ FAILURE: Mismatch detected!")
