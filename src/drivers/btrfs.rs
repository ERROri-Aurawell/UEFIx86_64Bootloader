use crate::drivers::controller::{Find, File, FileType};
use log::info;

pub struct BTRFS {}

impl Find for BTRFS {
    fn find_file(&self, file_name: &str, ft: FileType) -> Option<File> {
        info!("Searching from: {}", &file_name);
        
        todo!();
    }
}