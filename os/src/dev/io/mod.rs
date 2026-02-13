//! I/O traits and helper types.
//!
//! Provide a minimal synchronous/asynchronous device abstractions.
//!
//! Notes:
//! - Implementers should never block.
//! - Use [BlockingType] to indicate intent; callers should handle `IOError::WouldBlock`.
pub enum IOError {
    /// Operation would block.
    WouldBlock,
}

pub enum BlockingType {
    /// Request blocking behavior; the caller may handle [IOError::WouldBlock].
    Blocking,
    /// Request non-blocking behavior; the function should never return `IOError::WouldBlock`.
    NonBlocking,
}
/// Minimal character device interface.
/// - read: return one Unicode scalar value or `IOError::WouldBlock`.
/// - write: write one Unicode scalar value or return `IOError::WouldBlock`.
pub trait CharDevice {
    fn read(&mut self, blocking: BlockingType) -> Result<char, IOError>;
    fn write(&mut self, blocking: BlockingType, c: char) -> Result<(), IOError>;
}
