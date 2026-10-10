use alloc::string::String;
use alloc::vec::Vec;

use uefi::Error;

pub trait Find {
    fn find_file(&self, file_name: &str, ft: FileType) -> Result<File, Error>;
}

// Fat Pointer que chama?
pub struct File {
    content: Vec<u8>,
}

pub enum FileType {
    Linux,
    RamFS,
    Any
}