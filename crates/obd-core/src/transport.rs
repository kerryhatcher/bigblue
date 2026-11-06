use crate::Result;

/// A link to an OBD-II adapter.
///
/// Implementations send one ASCII command (without a line terminator) and
/// return the adapter's full textual reply, up to but not including the
/// prompt character. Higher layers parse that text with [`crate::response`].
pub trait Transport {
    fn command(&mut self, cmd: &str) -> Result<String>;
}
