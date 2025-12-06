use core::fmt::{Debug, Display, Write};

pub trait MessageError : Debug{
    fn print_to_writer(&self, f: &mut dyn Write){
        if let Err(err) = f.write_fmt(format_args!("{:?}",self)){
            f.write_fmt(format_args!("Error on printing error message: {:?}",err)).unwrap();
        }
    }
}

impl Display for dyn MessageError{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.print_to_writer(f);
        Ok(())
    }
}