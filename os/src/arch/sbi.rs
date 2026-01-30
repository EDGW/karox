// TODO: Temporarily Used
#![allow(missing_docs)]

pub trait SBITrait {
    fn console_putchr(c: char) -> Result<(), usize>;
    fn init();
}
