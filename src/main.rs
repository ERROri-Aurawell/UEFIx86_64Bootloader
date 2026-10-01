#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use log::info;
use spin::Mutex;

use uefi::boot::{self, MemoryType};
use uefi::mem::memory_map::{self, MemoryMap};
use uefi::prelude::*;

mod allocator;
use allocator::alloc::{Locked, MemAllocator};

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

    unsafe {
        ALLOCATOR.lock().init(chosen_start, chosen_size);
    }
    info!("Heap estruturado!");

    /*
    {
    let test_alloc: Box<i32> = Box::new(8);

    let test_alloc2: Box<usize> = Box::new(usize::MAX);

        if *test_alloc == 8 {
            info!("Alloc 'Box<i32>' funcional!");
        }

        if *test_alloc2 == usize::MAX {
            info!("Alloc 'Box<usize>' funcional!");
        }
    }
    info!("Dealloc (teoricamente) funcional!");
    */

    {
        let string: String = String::from("BOOTLOADER");
        let content = format!("{} : STRINGS EM BARE METAL PORAAAAAAA!", string);

        info!("{}", content);
    }

    loop {}
}
