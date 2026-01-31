// TODO: Temporarily Used
#![allow(missing_docs)]

pub trait SbiTrait {
    fn console_putchr(c: char) -> Result<(), usize>;
    fn init();
}
