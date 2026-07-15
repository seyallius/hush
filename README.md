# hush 🤫

**hush** is a zero-knowledge, client-side encryption CLI tool built in Rust, designed specifically for securing
sensitive videos and media before uploading them to the cloud or storing them on external drives.

Cloud providers love to generate thumbnails, read metadata, and scan your files. `hush` ensures they see nothing but
high-entropy random noise. No thumbnails, no titles, no traces.

## ✨ Core Philosophy

- **Zero-Knowledge**: Encrypts everything from byte 0 to the end. Cloud parsers fail immediately and generate no
  thumbnails.
- **No Plaintext on Disk**: Stream decrypted video directly to your media player (`mpv`, `vlc`) via `stdout`. Your
  sensitive files never touch your SSD in an unencrypted state.
- **Memory-Efficient**: Uses chunked streaming encryption. Encrypt/decrypt a 50GB video using only a few megabytes of
  RAM.

## 🛠️ Architecture & Features

### 1. The Custom "Envelope" Format

`hush` doesn't just dump encrypted bytes. It wraps your media in a custom binary container designed for instant seeking:

- **Plaintext Header**: Magic bytes (`VAUL`), version, Argon2 salt, and YubiKey challenges.
- **Encrypted Metadata**: The real filename, MIME type, original size, and a **Chunk Map** (byte offsets). This map
  allows instant seeking in a 2-hour movie without decrypting the preceding chunks.
- **Encrypted Chunks**: The actual media data, encrypted in independent, authenticated blocks (AEAD).

📖 **Full specification**: [docs/file-format.md](docs/file-format.md)

### 2. Flexible Key Management

Supports three distinct authentication modes to derive the Master Key:

- 🔑 **Password Only**: Uses Argon2id (memory-hard KDF) to stretch your passphrase.
- 🛡️ **YubiKey / TPM Only**: Uses HMAC-SHA1 challenge-response + HKDF.
- 🔐 **Combined (2FA)**: Combines Password + YubiKey using HKDF. If someone steals your YubiKey, they still need your
  password. If they keylog your password, they still need your physical key.

### 3. Swap & Tweak (Configurable)

Don't like the defaults? `hush` is designed to be tweaked without touching the source code.

- Configure chunk sizes, Argon2 parameters, and cipher algorithms via `~/.config/hush/config.toml`.
- Override any config file setting directly via CLI flags.
- Abstracted `StreamCipher` trait allows swapping between **XChaCha20-Poly1305** (Default) and **AES-256-GCM**
  seamlessly.

## 🗺️ Roadmap

### MVP (Current Focus)

- [x] Project scaffold, error handling, and CLI structure (`clap`).
- [ ] Chunked streaming encryption/decryption (XChaCha20-Poly1305).
- [ ] Custom binary envelope serialization (`bincode`).
- [ ] Password-only key derivation (Argon2id).
- [ ] `stdout` streaming for direct-to-player playback.

### v1.0.0: Hardware & Flexibility

- [ ] YubiKey / Hardware TPM integration (Challenge-Response).
- [ ] Combined Key Mode (Password + YubiKey via HKDF).
- [ ] True zero-RAM streaming encryption (handling chunk offsets dynamically without pre-scanning).
- [ ] AES-256-GCM cipher implementation.

### v2.0.0: The Ultimate Vault

- [ ] Shamir's Secret Sharing (SSS) for master key splitting and backup.
- [ ] FUSE filesystem mounting (`hush mount ~/vault ~/mnt` - decrypt on-the-fly as you click files).
- [ ] TUI (Terminal User Interface) for visual file management.
- [ ] Recovery QR code generation for secure offline parameter backups.

## 🚀 Getting Started

### Prerequisites

- Rust (1.70+)

### Installation

```bash
cargo build --release
```

### Usage

```bash
# Encrypt a file (creates a UUID-named .bin file)
hush encrypt --input my_secret_video.mp4

# Decrypt a file to disk
hush decrypt --input a1b2c3d4.bin --output video.mp4

# Stream directly to mpv (no plaintext written to disk!)
hush stream --input a1b2c3d4.bin | mpv -
```

---

*Keep your secrets safe. Keep them hush.* 🤐
