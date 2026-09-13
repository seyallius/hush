//! lib.rs defines the on-disk binary contracts (the VAUL envelope format).
//! It handles reading and writing the plaintext headers and encrypted metadata blobs.

/// Represents the unencrypted plaintext header at the start of a `.hush` file.
/// Contains KDF parameters and routing information, but NO secrets.
pub struct FileHeader {
    /// The magic bytes identifying this as a hush file (b"VAUL").
    pub magic: [u8; 4],
    /// The format version number.
    pub version: u8,
}

/// The trait responsible for writing a complete hush envelope to a byte sink.
pub trait EnvelopeWriter {
    /// Writes the plaintext header to the output.
    fn write_header(&mut self, header: &FileHeader) -> Result<(), EnvelopeError>;
}

/// Internal errors specific to parsing or writing the binary envelope.
#[derive(Debug)]
pub enum EnvelopeError {
    /// The magic bytes did not match b"VAUL".
    InvalidMagic,
    /// The file was truncated or malformed.
    UnexpectedEof,
}
