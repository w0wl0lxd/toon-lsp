//! In-house TOON codec: spec-conformant encode/decode and shared emitter core.

pub mod decode;
pub mod emit;
pub mod encode;
pub mod error;

pub use decode::decode;
pub use encode::{encode, encode_with_indent, encode_with_config};
pub use emit::Delimiter;
pub use error::{DecodeError, DecodeResult, EncodeError, EncodeResult};

/// Configuration options for the TOON encoder/decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToonConfig {
    /// Number of spaces to use for each indentation level (default: 2).
    pub indent: usize,
    /// Delimiter character used for arrays (default: Delimiter::Comma).
    pub delimiter: Delimiter,
}

impl Default for ToonConfig {
    fn default() -> Self {
        Self {
            indent: 2,
            delimiter: Delimiter::Comma,
        }
    }
}
