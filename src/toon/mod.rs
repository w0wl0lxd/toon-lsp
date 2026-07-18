//! In-house TOON codec: spec-conformant encode/decode and shared emitter core.

pub mod decode;
pub mod emit;
pub mod encode;
pub mod error;
pub mod fold;

pub use decode::{decode, decode_with_config};
pub use emit::Delimiter;
pub use encode::{encode, encode_with_config, encode_with_indent};
pub use error::{DecodeError, DecodeResult, EncodeError, EncodeResult};
pub use fold::{expand_paths, flatten_keys, fold_keys};

/// Configuration options for the TOON encoder/decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct ToonConfig {
    /// Number of spaces to use for each indentation level (default: 2).
    pub indent: usize,
    /// Delimiter character used for arrays (default: Delimiter::Comma).
    pub delimiter: Delimiter,
    /// Encode: collapse single-key object chains into dotted paths
    /// (key folding, spec v1.5). Default off so output stays spec-conformant.
    pub fold_keys: bool,
    /// Encode: flatten nested objects into dotted keys (more aggressive than
    /// `fold_keys`; inverse is `expand_paths`). Default off.
    pub flatten_keys: bool,
    /// Decode: expand dotted keys back into nested objects (inverse of
    /// `fold_keys` and `flatten_keys`). Default off so undotted parsing is unchanged.
    pub expand_paths: bool,
    /// Decode: preserve the int/float distinction from the source literal
    /// instead of normalizing whole-number floats and exponents to integers
    /// (default TOON spec behavior). Default off.
    pub preserve_number_types: bool,
}

impl Default for ToonConfig {
    fn default() -> Self {
        Self {
            indent: 2,
            delimiter: Delimiter::Comma,
            fold_keys: false,
            flatten_keys: false,
            expand_paths: false,
            preserve_number_types: false,
        }
    }
}
