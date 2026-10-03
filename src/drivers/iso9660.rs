use crate::drivers::controller::{Find, File, FileType};
use log::info;

pub struct ISO9660 {}

impl Find for ISO9660 {
    fn find_file(&self, file_name: &str, ft: FileType) -> Option<File> {
        info!("Searching from: {}", &file_name);

        todo!();
    }
}