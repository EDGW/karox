// TODO: Temporarily Used
#![allow(missing_docs)]

pub trait SBITrait{
    fn console_putstr(c: &str) -> Result<(),usize>;
    fn init();
}