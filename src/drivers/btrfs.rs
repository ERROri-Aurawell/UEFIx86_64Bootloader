use crate::drivers::controller::{Find, File, FileType};
use log::info;

use core::error::Error;

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::boxed::Box;

pub struct BTRFS {}

impl Find for BTRFS {
    fn find_file(&self, file_name: &str, ft: FileType) -> Result<File, Box<dyn Error>>{
        info!("Searching from: {}", &file_name);
        
        todo!();
    }
}