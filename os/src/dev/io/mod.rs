pub enum IOError {
    WouldBlock,
}

pub enum BlockingType {
    Blocking,
    NonBlocking,
}
pub trait CharDevice {
    fn read(&mut self, blocking: BlockingType) -> Result<char, IOError>;
    fn write(&mut self, blocking: BlockingType, c: char) -> Result<(), IOError>;
}
