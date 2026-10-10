//! Versioned Script IR: a pure validator and immutable validated content.

mod canonical;
mod model;
mod validation;
mod wire;

pub use model::{ContentDigest, ScriptContent, SpokenLine};
pub use validation::{Diagnostic, ValidationIssue};
pub use wire::{ReadError, ShapeDiagnostic, ShapeRule, WRITE_VERSION};

/// Admit an untrusted structured draft, normalizing derived text once.
/// This does not grant rights, save a revision, approve or start production.
pub fn read_script(bytes: &[u8]) -> Result<ScriptContent, ReadError> {
    let raw = wire::decode(bytes)?;
    validation::validate(raw).map_err(ReadError::Semantic)
}

/// Check an immutable stored/exported value without silently normalizing it.
/// The persistence adapter must also verify the stored digest and revision ID.
pub fn read_canonical_script(bytes: &[u8]) -> Result<ScriptContent, ReadError> {
    let script = read_script(bytes)?;
    if script.export_bytes() != bytes {
        return Err(ReadError::NonCanonicalDocument);
    }
    Ok(script)
}
