use alloc::string::String;
use alloc::vec::Vec;

pub trait Find {
    fn find_file(&self, file_name: &str, ft: FileType) -> Option<File>;
}

// Fat Pointer que chama?
pub struct File {
    content: Vec<u8>,
    size: String,
}

pub enum FileType {
    Linux,
    RamFS,
    Any
}