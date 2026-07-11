//! In-house TOON codec: spec-conformant encode/decode and shared emitter core.

pub mod error;
pub mod emit;
pub mod encode;
pub mod decode;

pub use error::{DecodeError, DecodeResult, EncodeError, EncodeResult};
