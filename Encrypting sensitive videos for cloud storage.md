# Encrypting sensitive videos for cloud storage

## User

I have a bunch of videos (and maybe image or any file in future as well but for now only video) that are personal and
sensitive. I don't want anyone to see them, know them, know the videos title, thumbnail, etc. Basically anything. And
since they're taking a lot of storage on my physical hard drive, I was thinking of uploading it on cloud. But... the
sensitive. And so, it got me thinking; I know rust yeah? I can build something that could encrypt the title, thumbnail,
audio, video pixels, and everything. Then the output is an obfuscated (correct word?) and encrypted file that if the
user on the cloud plays it, it doesn't works. It'll only be dycrypted by my rust application (CLI) and with a specific
algorithm or something. It's only going to be used by me.
Is it possible? I'm brainstorming and planning with you right now (not coding).

## DeepSeek

**Yes, it is absolutely possible.** In fact, you are describing a classic **"Zero-Knowledge" client-side encryption**
architecture.

Before we dive in: the word you're looking for isn't "obfuscated" – obfuscation is easily reversible (like Base64 or
XOR). You need **cryptographic encryption** (AES-256 or ChaCha20). If done right, the cloud provider will see nothing
but high-entropy random noise.

Here is the architectural blueprint for your Rust CLI, broken down into the core challenges and how to solve them.

### 1. The Simple Truth About Thumbnails & Metadata

- **Thumbnails**: Cloud providers (Google Drive, AWS S3, Dropbox) generate thumbnails by reading the first few bytes of
  the file header (e.g., the `moov` atom in MP4s). If you encrypt the file **from byte 0 to the very last byte**, the
  cloud parser sees garbage, fails immediately, and generates **no thumbnail**. Problem solved.
- **File Name**: Don't upload it as `my_vacation.mp4.enc`. Upload it as a random UUID (e.g., `a1b2c3d4-...bin`). Store
  the *real* filename inside the encrypted data envelope.
- **Timestamps**: Your OS and cloud will still see "Date Modified/Created". You can either ignore this, or set them to a
  fixed epoch time during upload.

---

### 2. The "Video" Challenge: Streaming vs. RAM

The biggest mistake is reading the whole 20GB video into RAM to encrypt it, then uploading it. **Don't do this.**

You need a **streaming encryption** architecture:

- Use a **Chunked Encryption** scheme (e.g., 1MB blocks).
- Each block gets its own unique Nonce/IV and is encrypted independently.
- This allows your decryption CLI to read, decrypt, and write to `stdout` in real-time without ever holding the whole
  file in memory.
- *Caveat*: If you use standard AES-GCM naively on a single stream, you can't seek through the video while watching. By
  using chunked encryption, you enable **seeking**—your player can request byte range `X`, and your CLI can decrypt just
  that 1MB chunk to skip ahead in the video.

---

### 3. The Key Management (The Hardest Part)

Encryption is easy. **Key storage is the real problem.** Since this is just for you:

- **Option A (Recommended)**: Use a strong passphrase. Your CLI will derive a 256-bit key using **Argon2id** (
  memory-hard KDF). You type the password every time you upload or watch. *Risk: You forget the password, the data is
  gone forever.*
- **Option B (Advanced)**: Use a YubiKey or system TPM to unwrap a master key.
- **Crucial Rule**: **Never** store the encryption key in the cloud alongside the file. Not even obfuscated.

---

### 4. The File Format (Your "Envelope")

Don't just dump encrypted bytes. Define a custom binary container format. When you run `my_vault encrypt video.mp4`, the
output file should look like this:

| Magic Bytes (4) | Version (1) | Salt (16) | Nonce (12) | Encrypted Header (Size) | Encrypted Chunks (1..N) |
|-----------------|-------------|-----------|------------|-------------------------|-------------------------|

- **Magic Bytes**: e.g., `[V, A, U, L]` so your CLI knows it's your format.
- **Salt**: Used with your password to derive the key.
- **Encrypted Header**: Contains the real filename, MIME type, total original size, and the *map* of chunk locations.

For the actual encryption primitive, use **AES-256-GCM** or **ChaCha20-Poly1305**. These are **Authenticated Encryption
** (AEAD) – meaning if someone tampers with the file in the cloud, your CLI will detect it and refuse to decrypt (
integrity check).

---

### 5. The Decryption Workflow (Watching)

Here is the killer feature for a video-focused CLI:

Instead of: `decrypt file.bin -> output.mp4` (which writes a sensitive plaintext video to your SSD, leaving traces).

Do this: **Pipe directly to your media player**.

```bash
my_vault stream ~/cloud_mount/a1b2c3.bin --password "..." | mpv -
```

Or, if you want to mount it:

```bash
my_vault mount ~/cloud_mount/ videos/
```

Your CLI acts as a FUSE filesystem (using the `fuse` crate). When you click the video in your file explorer, your CLI
decrypts only the chunks the player requests on-the-fly, and **never writes the decrypted file to disk**. This is the
gold standard for sensitivity.

---

### 6. Rust Ecosystem Pointers (For when you start coding)

- **Streaming Crypto**: Do not use the standard `aes-gcm` crate for huge files; it requires the whole plaintext. Use the
  **`chacha20poly1305`** crate with the `streaming` feature, or use **`aes-ctr`** (Counter mode) which is naturally
  parallelizable/streamable, combined with a separate digest for authentication per chunk.
- **Existing Protocol**: Look at the **`rage`** crate (Rust implementation of the `age` encryption tool). It already
  handles streaming, chunking, and recipient keys beautifully. You could wrap `rage` and just add a custom header to
  store the filename.
- **Argon2**: Use the `argon2` crate for key derivation.
- **Cloud Upload**: Use `rusoto` or `aws-sdk-rust` for S3, or `reqwest` for multipart uploads to Google Drive/Dropbox
  APIs. Encrypt **before** sending the bytes over the network.

---

### 7. The Golden Rule of DIY Crypto

**DO NOT IMPLEMENT THE CRYPTO PRIMITIVES YOURSELF.**
Do not write your own AES round function. Do not roll your own block cipher mode.

- Use `aes-gcm-siv` or `chacha20poly1305` from **RustCrypto** (audited crates).
- Use `rage` for the envelope if you don't want to manage chunk IVs manually.

---

### Summary Checklist for your Plan:

- [x] **Title**: Store encrypted in the header; upload as UUID.
- [x] **Thumbnail**: Encrypt the whole file; cloud parser fails to generate one.
- [x] **Playback**: Decrypt to `stdout` or use a FUSE mount; never write plaintext to disk.
- [x] **Storage**: Cloud sees random binary; impossible to play or identify.
- [x] **Algorithm**: Use Argon2id (password) + XChaCha20-Poly1305 (streaming authenticated encryption).

This is a solid, highly achievable project in Rust. Start by writing a function that takes a `Read` (source) and a
`Write` (destination), and passes the bytes through a `ChaCha20Poly1305` streaming encryptor in 1MB blocks. Once that
works locally, bolt on the cloud upload API.

Just remember: **key loss = permanent data loss**. Plan how you will back up that passphrase (e.g., a physical safe, or
split via Shamir's Secret Sharing) before you upload terabytes to the cloud!

## User

I'm listing whatever is sounding cool and I think useful based on your response, and what I myself have in mind or an
idea popped:

1. Use rust to build a CLI (in future use TUI - Just for learning and fun and for github showcasing).
2. Support encryption and decryption (duh?).
3. Support real-time de(en)cryption as well (i.e., we do have decrypt whole file and output into
   `<original_name>.<original_format>`, but also have a support to not decrypt and instead watch on stream. This was
   your suggestion).
4. The File Format (Your "Envelope")
   Magic Bytes (4)    Version (1)    Salt (16)    Nonce (12)    Encrypted Header (Size)    Encrypted Chunks (1..N)

   Magic Bytes: e.g., [V, A, U, L] so your CLI knows it's your format.

   Salt: Used with your password to derive the key.

   Encrypted Header: Contains the real filename, MIME type, total original size, and the map of chunk locations.

For the actual encryption primitive, use AES-256-GCM or ChaCha20-Poly1305. These are Authenticated Encryption (AEAD) –
meaning if someone tampers with the file in the cloud, your CLI will detect it and refuse to decrypt (integrity check).

5. The Key Management (The Hardest Part)

Encryption is easy. Key storage is the real problem. Since this is just for you:

    Option A (Recommended): Use a strong passphrase. Your CLI will derive a 256-bit key using Argon2id (memory-hard KDF). You type the password every time you upload or watch. Risk: You forget the password, the data is gone forever.

    Option B (Advanced): Use a YubiKey or system TPM to unwrap a master key.

How about both? Yeah that sounds better.

6. Cloud upload is done by the user not rust. Uploading to cloud was an example, the user may want to store the
   encrypted file on USB. It's basically on him we just encrypt it.
7. Just remember: key loss = permanent data loss. Plan how you will back up that passphrase (e.g., a physical safe, or
   split via Shamir's Secret Sharing) before you upload terabytes to the cloud! Should find a solution for this.

What's the next step?

## DeepSeek

Perfect! You’ve nailed the scope, especially with **#6**. Excluding the cloud upload API from your Rust code is a *
*massive** time-saver and keeps your project purely about data security—you just handle `stdin`/`stdout` and files. The
user (you) can then use `rclone`, `aws s3 cp`, or a USB drag-and-drop to move the resulting `.bin` around.

Since you've locked in the requirements, here is the exact **"Next Step"** blueprint. We need to make 4 critical
architectural decisions **before you write a single `cargo build`**, then map out the project structure.

---

### Decision 1: Settle the Crypto Primitives (Right Now)

- **Algorithm**: Use **ChaCha20-Poly1305** over AES-GCM. Reason: It's faster in software (no hardware acceleration
  needed) and has constant-time guarantees. **Crucially**, use the **XSalsa20 / XChaCha20** variant (192-bit nonce) so
  you can safely generate random nonces without worrying about counter collisions.
- **Chunk Size**: Set it to **1 MB (1,048,576 bytes)**. This is the perfect trade-off: small enough to stream smoothly
  to a video player without huge latency, and large enough to keep the header (chunk map) reasonably small. For a 10GB
  video, you'll have ~10,000 chunks – a tiny map.

---

### Decision 2: How to do "Both" (Password + YubiKey/TPM)

You said "both" – which is smart. Here is exactly how it works architecturally:

1. **Password Path**: You type a passphrase. Your CLI runs **Argon2id** (with a random 16-byte Salt stored in the
   header) to produce a 32-byte `PasswordKey`.
2. **YubiKey Path** (Optional flag, e.g., `--yubikey`):
    - Your CLI sends a random 32-byte `Challenge` (stored in the header) to the YubiKey via the `yubikey` crate (using
      HMAC-SHA1 challenge-response mode). The YubiKey returns a 20-byte `Response`.
    - Your CLI uses HKDF (HMAC-based Extract-and-Expand) on that `Response` to stretch it to a 32-byte `HardwareKey`.
3. **The Combine**: The final 32-byte `MasterKey` is derived by:
   `FinalKey = HKDF(PasswordKey XOR HardwareKey, context="VaultMaster")`.
   *(Note: XOR is safe here because both are already uniformly random bytes from good PRNGs/KDFs).*

**Result**: If someone steals your YubiKey but doesn't know your password, they can't decrypt. If they keylog your
password but don't have the physical YubiKey, they can't decrypt.

---

### Decision 3: The Passphrase Backup Strategy (Your #7)

Stop overthinking this for the MVP. Since you are the **sole user**, do this:

- **Print a Recovery QR code** to the terminal when you first encrypt a file (or via a `--generate-recovery` flag).
- The QR code contains: `{Argon2id Salt, Argon2id Parameters (m_cost, t_cost, p_cost), YubiKey Challenge (if used)}`.
- **Do NOT** put the password in the QR. You memorize the password or write it on paper.
- Store that QR code screenshot in a separate safe location (e.g., printed in a physical safe). If your file header
  corrupts, you have the exact parameters to brute-force derive the key again (though the password remains the only
  secret).

For true paranoia, you can use `sss` (Shamir Secret Sharing) crate to split the **MasterKey** into 3 shares (e.g., give
to 3 family members, need 2 to recover). But for a personal video vault, a strong password + a piece of paper in a safe
is entirely sufficient.

---

### Decision 4: The Exact Header Serialization

Your envelope is great. Let's define it strictly so you can code it:

| Field                   | Size     | Content                                                                                                                                                |
|:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------------------------------------------------|
| Magic                   | 4 u8     | `[0x56, 0x41, 0x55, 0x4C]` (VAUL)                                                                                                                      |
| Version                 | 1 u8     | `0x01`                                                                                                                                                 |
| Argon2 Salt             | 16 bytes | Random                                                                                                                                                 |
| Argon2 Params           | 12 bytes | `u32` (m_cost), `u32` (t_cost), `u32` (p_cost)                                                                                                         |
| YubiKey Challenge       | 32 bytes | Random (zeros if not used)                                                                                                                             |
| YubiKey Flag            | 1 bool   | `0x01` if used                                                                                                                                         |
| Encrypted Header (AEAD) | Variable | Contains the *actual* metadata: real filename, MIME, original size, chunk count, and `Vec<u64>` of chunk byte offsets. Encrypted with the `MasterKey`. |
| Chunks...               | Variable | Each chunk is `<u32:chunk_size><encrypted_bytes><u16:Poly1305_tag>` (though with ChaCha20-Poly1305, the tag is baked into the ciphertext).             |

**Critical rule**: The `Encrypted Header` is decrypted *first*. Your app uses the chunk map inside it to know where each
chunk starts, enabling instant seeking without scanning the whole file.

---

## The Immediate Next Step (Actionable Coding Plan)

Since you clarified #6 (no cloud APIs), your project structure becomes beautifully modular. Here is the exact folder
structure and immediate tasks:

```
my_vault/
├── Cargo.toml
├── src/
│   ├── lib.rs          // Core logic: encrypt, decrypt, stream
│   ├── crypto.rs       // Key derivation, AEAD wrappers
│   ├── envelope.rs     // Reading/Writing the custom binary format
│   ├── header.rs       // Serializing/Deserializing the metadata
│   └── bin/
│       └── vault.rs    // The actual CLI (clap for args)
```

**Your MVP Task (Do this tomorrow)**:

Write a function in `lib.rs` with this exact signature:

```rust
pub fn encrypt_stream<R: Read, W: Write>(
    source: &mut R,
    target: &mut W,
    password: &[u8],
    yubikey_secret: Option<&[u8]>, // for testing, use a static secret first
    filename: &str,
) -> Result<u64, VaultError>;
```

- Inside this function, write the **Magic + Header** immediately to `target`.
- Then loop: read 1MB from `source`, encrypt it with a **fresh 12-byte nonce** (incrementing counter or random - just
  store it per chunk!), write the ciphertext to `target`.
- For decryption, write a function that reads the header, derives the key, and writes decrypted bytes to `stdout` (for
  piping to `mpv`).

**For the TUI future goal** (#1): Keep all encryption logic strictly in `lib.rs` with zero dependency on `stdin`/
`stdout`/terminals. The TUI will just call `lib::decrypt_to_vec()` or manage the streaming handles. This separation
makes your GitHub showcase look deeply professional.

---

### Quick Dependency Checklist for your `Cargo.toml` (MVP)

```toml
[dependencies]
chacha20poly1305 = { version = "0.10", features = ["stream"] }  # for XChaCha20
argon2 = "0.5"          # key derivation
hkdf = "0.12"           # for combining keys
serde = { version = "1", features = ["derive"] }
bincode = "1.3"         # to serialize the header map efficiently
clap = { version = "4", features = ["derive"] } # for CLI args
rand = "0.8"            # for salts, nonces, challenges
anyhow = "1.0"          # for easy error handling in CLI
```

---

### Your Final Bottleneck to Plan For (Not Code Yet)

Since you want **streaming decryption** (#3), you must handle the **Chunk Map** carefully.
If you want to skip to the middle of a 2-hour movie, your CLI must know the byte offset of chunk #4000 *without*
decrypting chunks #1-#3999.
So when you write the `Encrypted Header`, ensure the `Vec<u64>` of offsets is written **as plaintext inside the
authenticated envelope** (meaning it's encrypted but small enough to decrypt instantly). This way, your `seek` function
just decrypts the 2KB header, finds the offset, jumps to that part of the file, and starts decrypting from there.

**Go ahead and scaffold the project.** Write the `encrypt_stream` function but don't hook it up to the CLI yet—just
write a unit test that encrypts a 5MB dummy file, then decrypts it back and asserts equality.

Once that green test passes, you've solved the hardest 80% of the problem. The CLI flags (`--encrypt`, `--decrypt`,
`--stream`, `--with-yubikey`) become trivial wrappers around that library. Let me know when you hit the chunk-offset
serialization or the YubiKey integration—those are the two spots where Rust's borrow checker likes to bite!

## User

Decision 1: Okay, whatever you think is best. But I want to code it in a way that I can swap out algorithm or change
chunk size in future without touching code. Just to test and tweak and have more flexibility. Maybe a toml config file
in `~/.<whatever_project_name_we_choose>/config.toml`? Or even cli command too. That'd be cool as well. Keep that in
mind.

Decision 2: That really sounds cool and awesome. One thing though; Aside from your solution, have separate support as
well. What I mean is, we'd support:
Password only.
Yubikey path only.
Password + Yubikey path (your suggestion).
This way, we have flexibility.

Decision 3: Okay, I'll stop over thinking for now. Let's just go with simple master key and I have to be the one
remembering it, if lost, bye bye! But! I'll want to not forget in future version, I want to add the `sss` you mentioned.
No need for recovery or QR code etc for MVP I guess.

Decision 4: Whatever you say sir!
===

What's next step?

## DeepSeek

Perfect. Locking in those three tweaks:

1. **Configurable everything** (Chunk size, Algorithm) via `~/.config/my_vault/config.toml` **AND** CLI overrides.
2. **Three key modes**: `Password-only`, `YubiKey-only`, and `Password+YubiKey` (combined).
3. **Recovery/SSS is a future feature** – MVP uses simple master key derivation. We'll stub it out.

Since we are officially moving from "brainstorming" to "building", here is the exact **"Next Step"** coding roadmap. I
am giving you the exact file structure, the `Config` struct, and the **first function you must write** to make
everything else fall into place.

---

### Step 1: The Project Scaffold

Run this in your terminal:

```bash
cargo init --lib my_vault
cd my_vault
mkdir src/bin
touch src/config.rs src/crypto.rs src/envelope.rs src/error.rs
```

Your `src/lib.rs` should just re-export the modules:

```rust
pub mod config;
pub mod crypto;
pub mod envelope;
pub mod error;
```

---

### Step 2: Define the Master `Config` (The Brain)

Create `src/config.rs`. This struct will be loaded from TOML **and** merged with CLI arguments using `clap`.

Here is the exact blueprint. Notice the `CipherKind` enum—we are future-proofing for AES-GCM swap:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CipherKind {
    XChaCha20Poly1305, // Default
    Aes256Gcm,         // Future swap
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chunk_size: usize, // In bytes, e.g., 1048576
    pub cipher: CipherKind,
    pub argon2_m_cost: u32, // Memory cost (KiB)
    pub argon2_t_cost: u32, // Iterations
    pub argon2_p_cost: u32, // Parallelism
}

impl Default for Config {
    fn default() -> Self {
        Self {
            chunk_size: 1_048_576, // 1 MB
            cipher: CipherKind::XChaCha20Poly1305,
            argon2_m_cost: 19_456, // ~19 MB memory (standard secure)
            argon2_t_cost: 2,
            argon2_p_cost: 1,
        }
    }
}
```

**For the CLI**: You will use `clap` with `#[arg(long)]` to override these. For example,
`--chunk-size 2097152 --cipher aes256gcm` will override whatever is in `~/.config/my_vault/config.toml`.

---

### Step 3: Define the Key Modes (Your Decision #2)

Still in `src/config.rs`, define how the user wants to authenticate:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyMode {
    Password,            // Just Argon2id
    YubiKey,             // Just hardware challenge-response
    Combined,            // Both (your awesome suggestion)
}
```

*Architecture note for "YubiKey-only"*: Since there is no password, we cannot use Argon2 (it needs a secret to stretch).
Instead, the CLI will generate a **random 32-byte Challenge** during encryption, store it in the file header, and during
decryption, it sends that exact challenge to the YubiKey. The response becomes the Master Key directly (stretched via
HKDF to 32 bytes).

---

### Step 4: The Core Trait (The "Swap Without Touching Code" Trick)

Because you want to swap algorithms without touching business logic, you **must** define a trait in `src/crypto.rs`.
This trait abstracts away the specific AEAD crate:

```rust
use crate::error::VaultError;

pub trait StreamCipher: Send + Sync {
    // Encrypts a single chunk of plaintext. Returns the ciphertext + auth tag.
    fn encrypt_chunk(&self, plaintext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, VaultError>;
    // Decrypts a single chunk. Returns plaintext.
    fn decrypt_chunk(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, VaultError>;
    
    // Required nonce size for this cipher (12 for ChaCha, 12 for AES-GCM).
    fn nonce_size(&self) -> usize;
    // Required tag size (16 for both).
    fn tag_size(&self) -> usize;
}
```

**Why this is gold**: Your `envelope.rs` (which reads/writes the file format) will **only** depend on
`&dyn StreamCipher`. When you load your `Config`, you simply instantiate the right cipher:

```rust
pub fn get_cipher(config: &Config) -> Box<dyn StreamCipher> {
    match config.cipher {
        CipherKind::XChaCha20Poly1305 => Box::new(ChaChaCipher::new()),
        CipherKind::Aes256Gcm => Box::new(AesCipher::new()), // Not implemented yet, but zero code changes in envelope.rs!
    }
}
```

---

### Step 5: The "Header" Serialization (Your Decision #4)

Create `src/envelope.rs`. Let's define the exact on-disk header struct (the metadata that gets encrypted and stored
after the magic bytes).

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct FileMetadata {
    pub original_filename: String,
    pub mime_type: String,          // e.g., "video/mp4"
    pub original_size: u64,
    pub chunk_count: u32,
    // The byte offset in the file where each encrypted chunk starts.
    // This allows instant seeking.
    pub chunk_offsets: Vec<u64>,    
}

// The unencrypted part of the header (written in plaintext to the start of the file).
pub struct FileHeader {
    pub magic: [u8; 4],          // b'VAUL'
    pub version: u8,             // 1
    pub salt: [u8; 16],          // For Argon2
    pub argon2_params: (u32, u32, u32), // m, t, p
    pub yubikey_challenge: [u8; 32], // All zeros if not used
    pub key_mode: KeyMode,       // From config.rs
    pub encrypted_metadata_len: u32, // Length of the encrypted blob
    // Then the encrypted metadata blob (FileMetadata) follows...
}
```

**The Golden Rule for Streaming**: You write the plaintext `FileHeader` to the output file first. Then you encrypt the
`FileMetadata` (which contains `chunk_offsets`) and write that. Then you encrypt the actual video chunks. This way, when
decrypting, you read the plaintext header *first*, derive your key, decrypt the small metadata, get the `chunk_offsets`,
and can instantly `seek()` to any chunk.

---

### Step 6: Your First "Hello World" Test (The Immediate Action)

Before you even build the CLI, write **one integration test** in `tests/integration_test.rs`. This test will force you
to connect all the traits and structs above.

Here is the exact code for that test (you can write this right now to guide your implementation):

```rust
use my_vault::{config::*, crypto::*, envelope::*};
use std::io::{Cursor, Read, Write};

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let config = Config::default();
    let cipher = get_cipher(&config);
    let plaintext = b"The quick brown fox jumps over the lazy dog. This is a test video content.";
    
    // 1. Encrypt
    let mut encrypted_data = Vec::new();
    // ... (call your future encrypt_stream function)
    
    // 2. Decrypt
    let mut decrypted_data = Vec::new();
    // ... (call your future decrypt_stream function)
    
    assert_eq!(&plaintext[..], &decrypted_data[..]);
}
```

**Your immediate goal**: Get this test to compile and pass.

- It will force you to write the `encrypt_stream` and `decrypt_stream` functions in `lib.rs` that accept a `Config`, a
  `KeyMode`, and a password (or mock YubiKey response).
- For now, mock the YubiKey: just read an environment variable `MOCK_YUBIKEY_SECRET` and treat it as the response.

---

### Step 7: The CLI Wrapper (Last Thing, Not Now)

Do not touch `src/bin/vault.rs` until that test passes. Once it passes, you just wrap it with `clap`:

```bash
my_vault encrypt video.mp4 --password "..." --key-mode Combined --output vault.bin
my_vault decrypt vault.bin --password "..." --output video.mp4
my_vault stream vault.bin --password "..." | mpv -  # Streams to stdout!
```

---

### Summary of Your Immediate "Next Step" (Monday Morning Coding)

1. **Copy the structs/enums** I provided into `config.rs` and `envelope.rs`.
2. **Implement the `StreamCipher` trait** for `XChaCha20Poly1305` in `crypto.rs` (use the `chacha20poly1305` crate with
   the `stream` feature).
3. **Write the `encrypt_stream` function**:
    - Accepts `Reader`, `Writer`, `&Config`, `KeyMode`, and password/YubiKey secret.
    - Writes the plaintext `FileHeader`.
    - Encrypts the `FileMetadata` (build the chunk map *while* reading the source, but you don't know offsets until you
      write. Solution: buffer the encrypted chunks in a temp file, or write them, store their offsets, then *rewind* and
      overwrite the header with the complete offsets. Or, simpler for MVP: collect all offsets in a `Vec<u64>` as you
      go, write the header *after* the chunks—but that breaks seeking unless you structure it. Actually, easiest:
      pre-scan the source to count chunks, allocate offsets, then write header, then re-read source and write chunks.
      For MVP, just read the whole file into memory to simplify the offset calculation, then optimize to streaming
      later.)
4. **Pass the test**.

**Warning**: The trickiest part is writing the header *before* the chunks but knowing the offsets beforehand. For your
MVP, just read the entire source file into a `Vec<u8>` so you know its exact size and chunk count. Calculate all
offsets, write the header, then iterate over the chunks. Yes, it uses RAM for the whole file—but for a *prototype*,
that's fine. We'll refactor to true streaming in v0.2 by writing to a temporary file first.

**Go ahead and scaffold this**. If you get stuck on the exact Rust syntax for `chacha20poly1305::stream::Encryptor` or
how to serialize `bincode` into the header, just ask me, and I'll give you the exact implementation snippet for that
specific function. You're now at the "writing code" phase—happy building

## User

Awesome. One thing.
What to name this project? Some of my old repos that I've created (with description for that repo on github):

- snapdb: Boot, snapshot, reset. Fast integration test databases for Go with Docker-backed MySQL/Postgres and native
  SQLite.
- letsgomeeeeeow: 1BRC but... Meeeeeow with rust.
- treegotchi: Feed it a tree structure, watch it grow. A lightweight app that instantly converts textual folder trees
  into real directory structures on disk.
- lastcall: A wrapper type for making long-running service loops cancellable.
- war: war - Offline dependency cache for Go projects. Fight network outages, code anywhere.
- nomnom: YT-DLP GUI Wrapper, but it eats URLs and spits out videos. simple.
- treeclip: A simple CLI Tool to facilitate your AI copy pasting ٩(ˊᗜˋ*)و ♡ (V2 - Rust)
- doppel: Your data’s doppelgänger — deep copies without side effects.
- goxide: Rust oxidization in Go?
- gosaidno: AOP for Go, because I really wanted to use annotations… but Go said no.
- gossip: gossip: decoupled event handling for Go — publish once, let everyone listen in.

## DeepSeek

I love your naming style—it’s witty, short, and usually a clever double-entendre. You’ve got the playful verb (`nomnom`,
`gossip`, `doppel`) and the functional pun (`gosaidno`, `treegotchi`).

For this project, we need a name that captures: **“Hides/obscures everything”** + **“Video/Media”** + *
*“Streaming/Chunks”**.

Here is my curated shortlist, tailored to *your* vibe:

**1. `hush`** *(My top recommendation)*

- **Why it fits**: It's the direct opposite of your `gossip` repo (which broadcasts events). `hush` silences the file—no
  one can hear it, see its thumbnail, or peek at its title. It’s a single, punchy verb.
- **CLI feel**: `hush encrypt vacation.mp4` → `vacation.mp4.hush`. Decrypt it back? `hush whisper vacation.mp4.hush` (
  brings it back quietly).

**2. `veil`**

- **Why it fits**: Short, elegant, and precisely means "to cover or conceal." Videos are visual, so putting a `veil`
  over the pixels, the thumbnail, and metadata makes perfect logical sense.
- **CLI feel**: `veil stream vault.bin | mpv -` – it lifts the veil just enough for *you* to watch.

**3. `obscura`**

- **Why it fits**: Inspired by *Camera Obscura* (the optical device). Since you're dealing with videos and pixels, this
  has a cinematic/technical ring to it, but literally means "dark chamber"—your data goes in and becomes invisible to
  the outside world.
- **CLI feel**: `obscura encrypt --key-mode combined ...` – sounds sexy and mysterious.

**4. `shroud`**

- **Why it fits**: Total, unbreakable covering. This fits your "encrypt the title, audio, video pixels, and everything"
  brief perfectly. It also matches the "serious security" tone.
- **CLI feel**: `shroud wrap video.mp4` and `shroud unwrap vault.bin`.

**5. `muffle`**

- **Why it fits**: A bit sillier, fits your `nomnom` energy. It muffles the audio, muffles the video frames, muffles the
  metadata—leaving a useless blob to anyone but you.
- **CLI feel**: `muffle watch private.bin` – it unmuffles it just for you.

---

**My verdict**: I'd vote **`hush`**.
It's 4 letters (you love short names: `war`, `war`, `veil` fits this length too), it contrasts beautifully with your
`gossip` project (talkative vs. silent), and it perfectly captures the "keep your secrets" vibe. Plus,
`hush stream secret.bin | mpv -` just sounds intuitive.

What vibe are you leaning toward? Or do you want me to generate more based on a specific theme (e.g., vaults, darkness,
magic)?

## User

`hush`! I liked it! What git repo description for it?

## DeepSeek

Excellent choice. Since you already have `gossip` ("publish once, let everyone listen in"), `hush` is the perfect
antithesis—it lets *nothing* out.

Here are three descriptions, ranked by my recommendation:

---

**Top Pick (Playful + Functional + Matches your `gossip` vibe)**
> **hush:** Total silence for your videos. Encrypts the title, thumbnail, audio, and pixels into an unplayable blob.
> Stream-decrypt on the fly with Rust.

---

**Runner-up (More direct, security-focused)**
> **hush:** A Rust CLI that buries your video metadata, frames, and audio in encrypted noise. Stream-decrypt instantly
> when it's just for your eyes.

---

**Short & Punchy (Like your `nomnom` style)**
> **hush:** Rust CLI that eats videos, spits out encrypted noise. Stream-decrypts for your eyes only.

---

I'd go with the **Top Pick** because it directly addresses your original three fears (title, thumbnail, pixels/audio) in
one sentence, name-drops the key feature (stream decrypt), and wears the Rust badge proudly. Plus, it's a subtle nod to
your `gossip` repo—one shouts, the other whispers.
