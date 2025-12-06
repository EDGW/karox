pub trait SBITrait{
    fn console_putstr(c: &str) -> Result<(),usize>;
}