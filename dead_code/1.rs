const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

struct Element {
    pointer: usize,
    size: usize,
}

struct MemAllocator {
    heap_start: usize,
    heap_end: usize,
    memory: [Option<Element>; 100],
}

impl MemAllocator {
    pub const fn empty() -> Self {
        const NONE_ELEMENT: Option<Element> = None;

        Self {
            heap_start: 0,
            heap_end: 0,
            memory: [NONE_ELEMENT; 100],
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
    }

    fn find_free_slot(&mut self) -> Option<&mut Option<Element>> {
        self.memory.iter_mut().find(|slot| slot.is_none())
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // 1. Coleta referências apenas dos blocos válidos (ignora None)
        let mut blocks: [Option<&Element>; 100] = [None; 100];
        let mut count = 0;

        for elem in self.memory.iter().flatten() {
            blocks[count] = Some(elem);
            count += 1;
        }

        let active_blocks: &mut [Option<&Element>] = &mut blocks[..count];

        // 2. Ordena as referências pelo endereço de memória (pointer)
        active_blocks.sort_unstable_by_key(|e: &Option<&Element>| e.unwrap().pointer);

        // 3. Varredura para encontrar os espaços vazios (gaps)
        let mut current_address: usize = self.heap_start;

        for item in active_blocks {
            let elem: &Element = item.unwrap();
            let aligned_address: usize = align_up(current_address, align);

            // Verifica se cabe no espaço entre o ponteiro atual e o próximo bloco ocupado
            if aligned_address + size <= elem.pointer {
                if let Some(slot) = self.find_free_slot() {
                    *slot = Some(Element {
                        pointer: aligned_address,
                        size,
                    });
                    return aligned_address as *mut u8;
                } else {
                    return ptr::null_mut(); // Array de controle cheio
                }
            }

            current_address = elem.pointer + elem.size;
        }

        // 4. Verifica o espaço após o último bloco até o fim do heap
        let aligned_address: usize = align_up(current_address, align);
        if aligned_address + size <= self.heap_end {
            if let Some(slot) = self.find_free_slot() {
                *slot = Some(Element {
                    pointer: aligned_address,
                    size,
                });
                return aligned_address as *mut u8;
            }
        }

        ptr::null_mut()
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        let target_pointer = ptr as usize;
        for slot in self.memory.iter_mut() {
            if let Some(element) = slot {
                if element.pointer == target_pointer {
                    *slot = None;
                    return;
                }
            }
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        loop {}
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        loop {}
    }
}

struct Locked<T> {
    inner: Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<'_, T> {
        self.inner.lock()
    }
}

unsafe impl GlobalAlloc for Locked<MemAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.lock().alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.lock().dealloc(ptr, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { self.lock().alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { self.lock().realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Locked<MemAllocator> = Locked::new(MemAllocator::empty());

extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;
