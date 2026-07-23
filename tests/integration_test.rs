//! Integration test for hush. Verifies full encrypt → decrypt roundtrip on real file I/O.

use hush::config::Config;
use hush::stream;
use std::io::Cursor;

#[test]
fn test_encrypt_decrypt_roundtrip_in_memory() {
    // Simulate a "video file" with 3.5 MB of pseudo-random data
    let original_data: Vec<u8> = (0..3_670_016u64).map(|i| (i % 251) as u8).collect();

    let config = Config::default(); // 1 MB chunks → 4 chunks (last one partial)
    let password = b"super-secret-password-123!";

    // --- Encrypt ---
    let mut encrypted_buf: Vec<u8> = Vec::new();
    let mut source = Cursor::new(&original_data);

    let bytes_written = stream::encrypt_stream(
        &mut source,
        &mut encrypted_buf,
        password,
        &config,
        "test_video.mp4",
        "video/mp4",
        original_data.len() as u64,
    )
    .unwrap();

    assert!(bytes_written > original_data.len() as u64); // Encrypted should be larger
    assert_ne!(&encrypted_buf[..100], &original_data[..100]); // Must not be plaintext

    // --- Decrypt ---
    let mut decrypted_buf: Vec<u8> = Vec::new();
    let mut enc_source = Cursor::new(&encrypted_buf);

    let metadata = stream::decrypt_stream(&mut enc_source, &mut decrypted_buf, password).unwrap();

    // --- Assert ---
    assert_eq!(decrypted_buf, original_data);
    assert_eq!(metadata.original_filename, "test_video.mp4");
    assert_eq!(metadata.mime_type, "video/mp4");
    assert_eq!(metadata.original_size, original_data.len() as u64);
    assert_eq!(metadata.chunk_count, 4); // 3.5MB / 1MB = 4 chunks
}

#[test]
fn test_wrong_password_fails() {
    let original_data = b"secret content that must not leak";
    let config = Config::default();

    let mut encrypted_buf: Vec<u8> = Vec::new();
    let mut source = Cursor::new(&original_data[..]);

    stream::encrypt_stream(
        &mut source,
        &mut encrypted_buf,
        b"correct-password",
        &config,
        "secret.txt",
        "text/plain",
        original_data.len() as u64,
    )
    .unwrap();

    // Attempt decryption with wrong password
    let mut decrypted_buf: Vec<u8> = Vec::new();
    let mut enc_source = Cursor::new(&encrypted_buf);

    let result = stream::decrypt_stream(&mut enc_source, &mut decrypted_buf, b"wrong-password");

    assert!(result.is_err()); // Must fail! AEAD tag verification catches this.
}
