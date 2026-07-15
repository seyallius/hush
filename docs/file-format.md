# Hush File Format Specification (v1)

This document defines the exact binary layout of `.hush` encrypted containers.
All multi-byte integers are **Little-Endian** unless otherwise noted.

## Overview

A `.hush` file consists of three sequential sections:

1. **Plaintext Header** – Unauthenticated, contains KDF parameters
2. **Encrypted Metadata** – AEAD-encrypted blob with seek map
3. **Encrypted Chunks** – Independent AEAD-encrypted data blocks

```text
[Plaintext Header][Encrypted Metadata][Chunk 0][Chunk 1]...[Chunk N-1]
```

---

## 1. Plaintext Header

Readable by anyone. Contains NO sensitive information.
Tampering here causes KDF to derive a wrong key → metadata decryption fails.

| Offset | Size | Field         | Description                                     |
|--------|------|---------------|-------------------------------------------------|
| 0      | 4    | Magic         | `b'VAUL'` (0x56 0x41 0x55 0x4C)                 |
| 4      | 1    | Version       | `0x01`                                          |
| 5      | 4    | Header Length | u32 LE – byte length of remaining header fields |
| 9      | 16   | Salt          | Random bytes for Argon2id                       |
| 25     | 4    | Argon2 M-Cost | u32 LE – memory cost in KiB                     |
| 29     | 4    | Argon2 T-Cost | u32 LE – iteration count                        |
| 33     | 4    | Argon2 P-Cost | u32 LE – parallelism                            |
| 37     | 32   | YK Challenge  | Random bytes (all zeros if KeyMode::Password)   |
| 69     | 1    | Key Mode      | 0x00=Password, 0x01=YubiKey, 0x02=Combined      |

> **Note:** Total plaintext header size = 4 + 1 + 4 + (Header Length)
> The `Header Length` field covers bytes from offset 9 onward.

---

## 2. Encrypted Metadata Blob

AEAD-encrypted with the Master Key derived from KDF.
Contains the seek map that enables instant chunk access.

| Offset | Size | Field       | Description                                      |
|--------|------|-------------|--------------------------------------------------|
| 0      | 4    | Meta Length | u32 LE – total bytes of nonce + ciphertext + tag |
| 4      | 24   | Nonce       | Random XChaCha20-Poly1305 nonce                  |
| 28     | Var  | Ciphertext  | bincode(FileMetadata) + 16-byte Poly1305 tag     |

### Decrypted FileMetadata Structure

Serialized via `bincode`:

```rust
struct FileMetadata {
    original_filename: String,
    mime_type: String,
    original_size: u64,
    chunk_count: u32,
    chunk_offsets: Vec<u64>,  // Absolute byte offset of each chunk in the file
}
```

> ⚠️ `chunk_offsets[i]` points to the **start of the nonce** for chunk `i`,
> not the ciphertext. This includes the 24-byte nonce prefix.

---

## 3. Encrypted Chunks

Each chunk is independently encrypted. No shared state between chunks.

| Offset | Size | Field      | Description                            |
|--------|------|------------|----------------------------------------|
| 0      | 24   | Nonce      | Random XChaCha20-Poly1305 nonce        |
| 24     | Var  | Ciphertext | plaintext_chunk + 16-byte Poly1305 tag |

- Chunk plaintext size = `config.chunk_size` (default 1MB), except last chunk
- Last chunk may be smaller (1 to chunk_size bytes)
- Each chunk uses a **fresh random 24-byte nonce**
- Nonce reuse with the same key = catastrophic security failure

---

## Key Derivation

### Password Mode

```
MasterKey = Argon2id(password, salt, m_cost, t_cost, p_cost, 32)
```

### YubiKey Mode

```
HardwareKey = HKDF-SHA256(yubikey_hmac_response, challenge, "HushYK", 32)
MasterKey = HardwareKey
```

### Combined Mode

```
PasswordKey = Argon2id(password, salt, m_cost, t_cost, p_cost, 32)
HardwareKey = HKDF-SHA256(yubikey_hmac_response, challenge, "HushYK", 32)
XorKey = PasswordKey XOR HardwareKey
MasterKey = HKDF-SHA256(XorKey, b"", "HushMaster", 32)
```

---

## Security Properties

- ✅ Full-file encryption from byte 0 → no thumbnails, no metadata leakage
- ✅ Authenticated encryption (AEAD) → tamper detection on every chunk
- ✅ Independent chunk nonces → safe random generation, no counter management
- ✅ Seekable via encrypted offset map → no sequential decryption needed
- ✅ Memory-hard KDF → GPU/ASIC brute-force resistance
- ✅ Zero plaintext on disk during streaming decryption
