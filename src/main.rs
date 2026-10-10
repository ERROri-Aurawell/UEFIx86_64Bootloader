#![no_std]
#![no_main]

use log::info;

use uefi::boot::{self, MemoryType};
use uefi::mem::memory_map::{self, MemoryMap};
use uefi::prelude::*;

mod allocator;
use allocator::alloc::{Locked, MemAllocator};

mod drivers;
use drivers::controller::{File, FileType};
use drivers::iso9660::ISO9660;
use drivers::controller::Find;

#[global_allocator]
static ALLOCATOR: Locked<MemAllocator> = Locked::new(MemAllocator::empty());

extern crate alloc;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;


#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    //info!("Hello World from UEFI!");

    let memory_map = match boot::memory_map(MemoryType::LOADER_DATA) {
        Ok(map) => map,
        Err(err) => {
            info!("Falha ao obter o mapa da memória: {:?}", err);
            return Status::LOAD_ERROR;
        }
    };

    let mut chosen_start = 0;
    let mut chosen_size = 0;

    for desc in memory_map.entries() {
        if desc.ty == MemoryType::CONVENTIONAL {
            let size = desc.page_count as usize * 4096;
            if size >= 16 * 1024 * 1024 {
                chosen_start = desc.phys_start as usize;
                chosen_size = size;
                break;
            }
        }
    }

    if chosen_start == 0 {
        panic!("Nenhum bloco de memória convencional suficiente encontrado")
    }

    // O primeiro, e esperamos que seja o único bloco Unsafe do Main.
    unsafe { ALLOCATOR.lock().init(chosen_start, chosen_size) };
    info!("Heap estruturado!");

    /*

        {
        let string: String = String::from("BOOTLOADER");
        let content = format!("{} : STRINGS EM BARE METAL PORAAAAAAA!", string);

        info!("{}", content);
    }
    */

    let cd_rom = ISO9660 {};

    let _ = cd_rom.find_file("bzImage", FileType::Linux);

    loop {}
}

fn linux_jump() -> ! {
    loop {}
}
