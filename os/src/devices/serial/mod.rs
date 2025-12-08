pub mod uart16550;

pub trait Uart: Send {
    fn read(&mut self) -> Option<u8>;
    fn write(&mut self, word: u8);
    fn flush(&mut self);
}
