//! stream.rs - Orchestrates the full encrypt/decrypt pipeline,
//! tying together KDF, StreamCipher, and Envelope into a cohesive streaming workflow.

use crate::{
    config::{Config, KeyMode},
    crypto::{get_cipher, kdf},
    envelope::{FileHeader, FileMetadata},
    error::VaultError,
    progress::ProgressReporter,
};
use rand::RngCore;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

/// Size of the u32 length prefix stored before each encrypted chunk on disk.
const CHUNK_LEN_PREFIX_SIZE: u64 = 4;

/// Encrypts a source file into the hush binary format and writes it to the target path.
///
/// # Arguments
/// * `input_path` - Path to the plaintext source file (e.g., video.mp4).
/// * `output_path` - Path where the encrypted .hush file will be written.
/// * `password` - The user's password bytes for key derivation.
/// * `config` - Application configuration (chunk size, cipher, argon2 params).
///
/// # Returns
/// The total number of bytes written to the output file.
pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    password: &[u8],
    config: &Config,
    progress: &dyn ProgressReporter,
) -> Result<u64, VaultError> {
    let source_file = File::open(input_path)?;
    let mut reader = BufReader::new(&source_file);
    let original_size = source_file.metadata()?.len();

    let target_file = File::create(output_path)?;
    let mut writer = BufWriter::new(target_file);

    let original_filename = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mime_type = guess_mime_type(&original_filename);

    encrypt_stream(
        &mut reader,
        &mut writer,
        password,
        config,
        &original_filename,
        &mime_type,
        original_size,
        progress,
    )
}

/// Core streaming encryption: reads plaintext from `source`, writes hush format to `target`.
///
/// # File Layout Written:
/// ```text
/// [u32: header_len][FileHeader bytes]
/// [u32: encrypted_metadata_len][encrypted_metadata bytes]
/// [u32: chunk_0_len][nonce_0][ciphertext_0 + tag_0]
/// [u32: chunk_1_len][nonce_1][ciphertext_1 + tag_1]
/// ...
/// [u32: chunk_N_len][nonce_N][ciphertext_N + tag_N]
/// ```
pub fn encrypt_stream<R: Read, W: Write>(
    source: &mut R,
    target: &mut W,
    password: &[u8],
    config: &Config,
    original_filename: &str,
    mime_type: &str,
    original_size: u64,
    progress: &dyn ProgressReporter,
) -> Result<u64, VaultError> {
    // --- Step 1: Generate cryptographic randomness ---
    let mut salt = [0u8; 16];
    let yubikey_challenge = [0u8; 32]; // Zeros for MVP (password-only mode)
    rand::thread_rng().fill_bytes(&mut salt);

    // --- Step 2: Derive the master key via Argon2id ---
    let master_key = kdf::derive_key(password, &salt, config)?;

    // --- Step 3: Instantiate the stream cipher ---
    let cipher = get_cipher(config, &master_key);
    let nonce_size = cipher.nonce_size();
    let tag_size = cipher.tag_size();

    // --- Step 4: Compute chunk layout ---
    let chunk_size = config.chunk_size as u64;
    let chunk_count = if original_size == 0 {
        0
    } else {
        ((original_size + chunk_size - 1) / chunk_size) as u32
    };

    // --- Step 5: Build metadata with dummy offsets to measure encrypted size ---
    let dummy_offsets = vec![0u64; chunk_count as usize];
    let dummy_metadata = FileMetadata {
        original_filename: original_filename.to_string(),
        mime_type: mime_type.to_string(),
        original_size,
        chunk_count,
        chunk_offsets: dummy_offsets,
    };

    let dummy_meta_bytes = bincode::serialize(&dummy_metadata)
        .map_err(|e| VaultError::Encryption(format!("Metadata serialization failed: {}", e)))?;

    // Encrypt dummy metadata to measure its size (nonce + ciphertext + tag)
    let mut meta_nonce = vec![0u8; nonce_size];
    rand::thread_rng().fill_bytes(&mut meta_nonce);
    let encrypted_dummy_meta = cipher.encrypt_chunk(&dummy_meta_bytes, &meta_nonce)?;
    let encrypted_metadata_total_size = (nonce_size + encrypted_dummy_meta.len()) as u64;

    // --- Step 6: Build the plaintext header ---
    let argon2_params = (
        config.argon2_m_cost,
        config.argon2_t_cost,
        config.argon2_p_cost,
    );
    let header = FileHeader::new(salt, argon2_params, yubikey_challenge, KeyMode::Password);
    let header_bytes = header.to_bytes()?;
    let header_total_size = (4 + header_bytes.len()) as u64; // 4 bytes for the u32 length prefix

    // --- Step 7: Compute real chunk offsets ---
    // Chunk data section starts after: header + encrypted_metadata_len_prefix(4) + encrypted_metadata
    let chunk_data_start = header_total_size + 4 + encrypted_metadata_total_size;

    let mut chunk_offsets: Vec<u64> = Vec::with_capacity(chunk_count as usize);
    let mut current_offset = chunk_data_start;

    for i in 0..chunk_count {
        chunk_offsets.push(current_offset);
        let plaintext_chunk_size =
            std::cmp::min(chunk_size, original_size - (i as u64 * chunk_size));
        // On disk: [u32 len prefix][nonce][ciphertext + tag]
        let encrypted_chunk_size = nonce_size as u64 + plaintext_chunk_size + tag_size as u64;
        current_offset += CHUNK_LEN_PREFIX_SIZE + encrypted_chunk_size;
    }

    // --- Step 8: Build real metadata and encrypt it ---
    let metadata = FileMetadata {
        original_filename: original_filename.to_string(),
        mime_type: mime_type.to_string(),
        original_size,
        chunk_count,
        chunk_offsets,
    };

    let meta_bytes = bincode::serialize(&metadata)
        .map_err(|e| VaultError::Encryption(format!("Metadata serialization failed: {}", e)))?;

    // Fresh nonce for the real metadata encryption
    let mut meta_nonce = vec![0u8; nonce_size];
    rand::thread_rng().fill_bytes(&mut meta_nonce);
    let encrypted_meta = cipher.encrypt_chunk(&meta_bytes, &meta_nonce)?;

    // --- Step 9: Write header to output ---
    header.write_to(target)?;

    // --- Step 10: Write encrypted metadata (nonce + ciphertext) ---
    let enc_meta_total = (nonce_size + encrypted_meta.len()) as u32;
    target.write_all(&enc_meta_total.to_le_bytes())?;
    target.write_all(&meta_nonce)?;
    target.write_all(&encrypted_meta)?;

    // --- Step 11: Encrypt and write chunks ---
    let mut buffer = vec![0u8; config.chunk_size];
    let mut total_written: u64 = header_total_size + 4 + encrypted_metadata_total_size;
    let mut bytes_processed: u64 = 0;

    for _ in 0..chunk_count {
        let bytes_read = read_exact_or_eof(source, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        // Generate a fresh random nonce for this chunk
        let mut chunk_nonce = vec![0u8; nonce_size];
        rand::thread_rng().fill_bytes(&mut chunk_nonce);

        // Encrypt the chunk
        let encrypted_chunk = cipher.encrypt_chunk(&buffer[..bytes_read], &chunk_nonce)?;

        // Write: [u32: total_chunk_bytes][nonce][ciphertext+tag]
        let chunk_total = (nonce_size + encrypted_chunk.len()) as u32;
        target.write_all(&chunk_total.to_le_bytes())?;
        target.write_all(&chunk_nonce)?;
        target.write_all(&encrypted_chunk)?;

        total_written += CHUNK_LEN_PREFIX_SIZE + chunk_total as u64;
        bytes_processed += bytes_read as u64;
        progress.report(bytes_processed, original_size);
    }

    target.flush()?;
    progress.finish();
    Ok(total_written)
}

/// Decrypts a hush file and writes the plaintext to the target writer.
///
/// # Arguments
/// * `input_path` - Path to the encrypted .hush file.
/// * `target` - Any `Write` implementor (File, stdout, pipe, etc.).
/// * `password` - The user's password bytes for key derivation.
///
/// # Returns
/// The FileMetadata of the decrypted file (contains original filename, size, etc.).
pub fn decrypt_to_writer<W: Write>(
    input_path: &Path,
    target: &mut W,
    password: &[u8],
    progress: &dyn ProgressReporter,
) -> Result<FileMetadata, VaultError> {
    let source_file = File::open(input_path)?;
    let mut reader = BufReader::new(source_file);

    decrypt_stream(&mut reader, target, password, progress)
}

/// Core streaming decryption: reads hush format from `source`, writes plaintext to `target`.
///
/// This function is designed to work with stdout for piping to media players:
/// `hush stream secret.hush | mpv -`
pub fn decrypt_stream<R: Read, W: Write>(
    source: &mut R,
    target: &mut W,
    password: &[u8],
    progress: &dyn ProgressReporter,
) -> Result<FileMetadata, VaultError> {
    // --- Step 1: Read and parse the plaintext header ---
    let header = FileHeader::read_from(source)?;

    // --- Step 2: Derive the master key using stored salt + params ---
    let config = Config {
        argon2_m_cost: header.argon2_params.0,
        argon2_t_cost: header.argon2_params.1,
        argon2_p_cost: header.argon2_params.2,
        ..Config::default()
    };
    let master_key = kdf::derive_key(password, &header.salt, &config)?;

    // --- Step 3: Instantiate cipher ---
    let cipher = get_cipher(&config, &master_key);
    let nonce_size = cipher.nonce_size();

    // --- Step 4: Read and decrypt the metadata ---
    let mut enc_meta_len_buf = [0u8; 4];
    source.read_exact(&mut enc_meta_len_buf)?;
    let enc_meta_total = u32::from_le_bytes(enc_meta_len_buf) as usize;

    let mut enc_meta_blob = vec![0u8; enc_meta_total];
    source.read_exact(&mut enc_meta_blob)?;

    // Split nonce from ciphertext
    let (meta_nonce, meta_ciphertext) = enc_meta_blob.split_at(nonce_size);
    let meta_plaintext = cipher.decrypt_chunk(meta_ciphertext, meta_nonce)?;

    let metadata: FileMetadata = bincode::deserialize(&meta_plaintext)
        .map_err(|e| VaultError::Decryption(format!("Metadata deserialization failed: {}", e)))?;

    // --- Step 5: Decrypt chunks sequentially ---
    let mut bytes_processed: u64 = 0;
    for _ in 0..metadata.chunk_count {
        // Read the u32 chunk length prefix
        let mut chunk_len_buf = [0u8; 4];
        source.read_exact(&mut chunk_len_buf)?;
        let chunk_total = u32::from_le_bytes(chunk_len_buf) as usize;

        // Read nonce + ciphertext
        let mut chunk_blob = vec![0u8; chunk_total];
        source.read_exact(&mut chunk_blob)?;

        let (chunk_nonce, chunk_ciphertext) = chunk_blob.split_at(nonce_size);
        let plaintext_chunk = cipher.decrypt_chunk(chunk_ciphertext, chunk_nonce)?;

        target.write_all(&plaintext_chunk)?;
        bytes_processed += plaintext_chunk.len() as u64;
        progress.report(bytes_processed, metadata.original_size);
    }

    target.flush()?;
    progress.finish();
    Ok(metadata)
}

/// Reads up to `buf.len()` bytes, returning the number actually read.
/// Returns 0 on EOF instead of erroring (unlike `read_exact`).
fn read_exact_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<usize, VaultError> {
    let mut total_read = 0;
    while total_read < buf.len() {
        match reader.read(&mut buf[total_read..]) {
            Ok(0) => break, // EOF
            Ok(n) => total_read += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(VaultError::Io(e)),
        }
    }
    Ok(total_read)
}

/// Simple MIME type guesser based on file extension.
/// For MVP, covers common video/image types. Extend as needed.
fn guess_mime_type(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "mp4" => "video/mp4",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
    .to_string()
}
