use alloc::{boxed::Box, vec, vec::Vec};
use core::{slice, str};
use utils::endian::{BigEndian32, BigEndian64, EndianData};

pub struct Property {
    pub name: Box<str>,
    pub data: Box<[u8]>,
}

impl Property {
    fn value_as_ptr<T: Sized>(&self) -> Result<*const T, PropertyError> {
        if self.data.len() < size_of::<T>() {
            return Err(PropertyError::InvalidPropFormat);
        }
        Ok(self.data.as_ref().as_ptr() as *const T)
    }
    fn value_as_ref<T>(&self) -> Result<&T, PropertyError> {
        Ok(unsafe { &*self.value_as_ptr()? })
    }
    fn value_as_arr<T>(&self) -> Result<&[T], PropertyError> {
        let elem_len = size_of::<T>();
        let len = self.data.len();
        let count = len / elem_len;
        let st_ptr = self.value_as_ptr()?;
        let res = unsafe { slice::from_raw_parts(st_ptr, count) };
        Ok(res)
    }
}
impl Property {
    pub fn value_as_u32(&self) -> Result<u32, PropertyError> {
        Ok(self.value_as_ref::<BigEndian32>()?.value())
    }
    pub fn value_as_u64(&self) -> Result<u64, PropertyError> {
        Ok(self.value_as_ref::<BigEndian64>()?.value())
    }
    pub fn value_as_str(&self) -> Result<&str, PropertyError> {
        Ok(unsafe { str::from_utf8_unchecked(self.value_as_arr::<u8>()?).trim_end_matches('\0') })
    }
    pub fn value_as_strlist(&self) -> Result<Vec<&str>, PropertyError> {
        let slice = self.value_as_arr::<u8>()?;
        let mut st = 0;
        let mut res = vec![];
        for i in 0..slice.len() {
            if slice[i] == 0 {
                res.push(unsafe { str::from_utf8_unchecked(&slice[st..i]) });
                st = i + 1;
            }
        }
        if st != slice.len() {
            res.push(unsafe { str::from_utf8_unchecked(&slice[st..slice.len()]) }); // add last if not terminated with 0
        }
        Ok(res)
    }
    pub fn value_as_proplist<TType>(&self) -> Result<&[TType], PropertyError> {
        self.value_as_arr::<TType>()
    }
}

#[derive(Debug)]
pub enum PropertyError {
    InvalidPropFormat,
    PropNotFound,
    DanglingHandle
}
