use core::error::Error;

extern crate alloc;
use alloc::boxed::Box;
use alloc::{fmt, vec};
use alloc::vec::Vec;
pub trait Find {
    fn find_file(&self, file_name: &str, ft: FileType) -> Result<File, Box<dyn Error>>;
}

// Fat Pointer que chama?
pub struct File {
    content: Vec<u8>,
}

pub enum FileType {
    Linux,
    RamFS,
    Any,
}

#[derive(Debug)]
pub enum ErrorType {
    InvalidPvd
}

#[derive(Debug)]
pub struct FileError {
    pub error: ErrorType
}

impl fmt::Display for FileError{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Aurawell Controller Error : {:?}", self.error)
    }
}

impl Error for FileError {}