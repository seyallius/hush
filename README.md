# hush 🤫

**hush** is a zero-knowledge, client-side encryption tool for sensitive videos and media. It is built in Rust, runs
entirely on your machine, and is designed so that nobody — not a cloud provider, not a USB stick, not a filesystem
indexer — can see what you encrypted.

> **Status: pre-alpha.** This repository is in the skeleton phase. The architecture, contracts, and CLI/TUI surfaces are
> being defined. Nothing works yet. See [Roadmap](#roadmap) for what's coming.

---

## Why hush exists

Cloud providers generate thumbnails, read metadata, and scan your files. They see the title of your video, the first
frame, the codec, the duration. Even if you trust them, you can't trust their automated pipelines.

hush makes all of that impossible. It encrypts the file from byte 0 to the last byte, so a cloud parser sees only
high-entropy noise. No thumbnail. No title. No metadata. No trace.

The same applies to external drives, backup tools, and anything else that might index your files.

---

## What hush is

- **Zero-knowledge by construction.** Everything is encrypted, including the filename and MIME type. The cloud (or USB,
  or NAS) sees a random blob.
- **No plaintext on disk.** Decrypt directly to a media player through a pipe or an HTTP stream. Your SSD never sees the
  decrypted video.
- **Memory-efficient by design.** Chunked streaming encryption means a 50GB video can be encrypted or played back with a
  few megabytes of RAM.
- **Seekable.** Each chunk is encrypted independently, with its own nonce. Seeking into the middle of a movie does not
  require decrypting the chunks before it.

## What hush is not

- Not a cloud upload tool. You upload the encrypted blob however you like — `rclone`, `aws s3 cp`, `scp`, a USB stick.
- Not a password manager. The master key is derived from your password (and optionally a YubiKey). If you lose them, the
  data is gone.
- Not a replacement for full-disk encryption. It's a container format for individual files and directories.

---

## How it will work

### The file format

hush wraps your media in a custom binary container called an **envelope**. It has three parts:

1. **Plaintext header** — magic bytes (`VAUL`), version, Argon2 salt, YubiKey challenge, key mode. Contains no secrets.
2. **Encrypted metadata** — the real filename, MIME type, original size, and a chunk map (byte offsets). Encrypted with
   the master key.
3. **Encrypted chunks** — the media data, split into independent AEAD-encrypted blocks.

The chunk map is what makes seeking possible: a media player can request byte range X, and hush decrypts only the chunk
containing X.

Full specification: [`docs/file-format.md`](docs/file-format.md).

### Key modes

hush will support three ways to derive the master key:

- **Password only** — Argon2id stretches your passphrase into a 256-bit key.
- **YubiKey only** — HMAC-SHA1 challenge-response, stretched with HKDF.
- **Combined** — both factors required. A stolen YubiKey without your password is useless. A keylogged password without
  the YubiKey is useless.

### Interfaces

- **CLI** — `hush encrypt`, `hush decrypt`, `hush stream`.
- **TUI** — a terminal interface to browse encrypted files, preview metadata, and launch playback.
- **Streaming** — pipe decrypted bytes to `mpv`, or serve them over a local HTTP stream to `vlc`.

### Configurability

Cipher, chunk size, Argon2 parameters, and default key mode will be configurable through `~/.config/hush/config.toml`
and overridable per-invocation via CLI flags.

---

## Architecture

Code is organized into strict modules with one-way dependencies:

```
cli   ──► core ──► crypto
              └──► envelope
tui   ──► core
```

- `core` never depends on `cli` or `tui`.
- `crypto` and `envelope` never depend on `core`'s higher-level logic.
- Everything is behind traits so ciphers, key modes, and envelope versions can be swapped without touching business
  logic.

Full architecture doc: [`docs/architecture.md`](docs/architecture.md) _(coming in M5)_.

---

## Roadmap

hush is developed in six milestones. Each milestone has a clear exit criterion. Nothing is claimed as done until its
milestone closes.

| Milestone                       | Goal                                                             | Status      |
|---------------------------------|------------------------------------------------------------------|-------------|
| **M0 — Skeleton**               | Repo layout, contracts, CI, CLI/TUI surfaces (no behavior).      | In progress |
| **M1 — MVP**                    | Password-only encrypt / decrypt / stream. Usable.                | Not started |
| **M2 — TUI + Config**           | TUI, config file, HTTP stream, VLC launch. First stable release. | Not started |
| **M3 — Hardware + Flexibility** | YubiKey, combined mode, AES-GCM, zero-RAM streaming.             | Not started |
| **M4 — Vault**                  | Directory encryption, SSS recovery, FUSE mount.                  | Not started |
| **M5 — Power User**             | Batch ops, key rotation, hooks, ARM, full docs.                  | Not started |

Full roadmap with phases: [`docs/roadmap.md`](docs/roadmap.md) _(coming in M5)_.

---

## Contributing

Not open to contributions yet — the skeleton is being laid down. Once M0 closes, `CONTRIBUTING.md` will explain how to
pick an issue, branch naming, commit format, and the label scheme.

---

## Security notes

hush does not implement cryptographic primitives. It uses audited RustCrypto crates:

- Argon2id for password-based key derivation
- XChaCha20-Poly1305 (default) and AES-256-GCM (alternative) for AEAD
- HKDF for key combination

The threat model, guarantees, and non-guarantees will be documented in `docs/security.md` _(coming in M5)_.

---

## License

TBD.

---

_Keep your secrets safe. Keep them hush._ 🤐
