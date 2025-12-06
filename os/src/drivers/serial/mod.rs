pub mod ns16550a;


pub trait Uart: Send{
    fn read(&mut self) -> Option<u8>;
    fn write(&mut self, word: u8);
    fn flush(&mut self);
}