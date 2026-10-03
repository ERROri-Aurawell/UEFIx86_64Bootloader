use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use log::info;
use spin::Mutex;

// Estrutura Pai que envelopa o verdadeiro alocador de memória em um Mutex
pub struct Locked<T> {
    inner: Mutex<T>,
}

impl<T> Locked<T> {
    //Função New para o conteúdo interior
    pub const fn new(inner: T) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    //Função Lock para o conteúdo interior
    pub fn lock(&self) -> spin::MutexGuard<'_, T> {
        self.inner.lock()
    }
}
/*
    |---------------------------------------------------------------------------|
    |                      Implementando a Trait GlobalAlloc                    |
    |                             Funções necessárias:                          |
    |---------------------------------------------------------------------------|
    | alloc(&self, layout:Layout) -> *mut u8                                    |
    | --    Alocar um bloco de memória seguindo um layout determinado           |
    |---------------------------------------------------------------------------|
    | dealloc(&self, ptr: *mut u8, layout: Layout)                              |
    | --    Liberar um bloco de memória                                         |
    |---------------------------------------------------------------------------|
    | alloc_zeroed(&self, layout: Layout) -> *mut u8                            |
    | --    Alocar um bloco completamente limpo                                 |
    |---------------------------------------------------------------------------|
    | realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8  |
    | --    Realocar um bloco de memória, preservando seu conteúdo              |
    |---------------------------------------------------------------------------|
*/
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

/*
    |---------------------------------------------------------------------------|
    |               Estrutura do Nó de Memória Livre (FreeNode)                 |
    |---------------------------------------------------------------------------|
    | O FreeNode é gravado diretamente no início do bloco de memória livre:     |
    |                                                                           |
    |  Endereço do Bloco Libre                                                  |
    |  +-----------------------+------------------------------------------+     |
    |  |  FreeNode (Header)    |          Payload Não Utilizado           |     |
    |  |  [next]  | [prev]     |                                          |     |
    |  |  Option  | Option     |                                          |     |
    |  +----+---------+--------+------------------------------------------+     |
    |       |         |                                                         |
    |       |         +----> Ponteiro (usize) para nó anterior                  |
    |       +--------------> Ponteiro (usize) para próximo nó                   |
    |---------------------------------------------------------------------------|
    | Campos:                                                                   |
    |   next - Endereço do próximo bloco livre de mesma ordem k na lista.       |
    |   prev - Endereço do bloco livre anterior de mesma ordem k na lista.      |
    |---------------------------------------------------------------------------|
*/
#[repr(C)]
pub struct FreeNode {
    pub next: Option<usize>,
    pub prev: Option<usize>,
}

/*
    |---------------------------------------------------------------------------|
    |   Struct de controle do Heap - Alocador Buddy (Buddy Allocator)           |
    |---------------------------------------------------------------------------|
    |   heap_start   - Endereço físico/virtual do início da região do Heap.     |
    |   heap_end     - Endereço físico/virtual do fim do Heap (start + size).   |
    |   leaf_size    - Tamanho mínimo de um bloco livre (Folha = 64 bytes).     |
    |   leaf_maximum - Maior ordem (k) permitida baseada no tamanho do Heap.    |
    |   free_lists   - Array de listas encadeadas. O índice 'k' guarda a cabeça |
    |                  da lista de blocos livres de tamanho (leaf_size << k).   |
    |---------------------------------------------------------------------------|
    |                         Funções de controle                               |
    |---------------------------------------------------------------------------|
    | empty() -> Self                                                           |
    | -- Inicializa a estrutura do alocador zerada e sem blocos associados.     |
    |---------------------------------------------------------------------------|
    | log_free_list(&self)                                                      |
    | -- Imprime no log o estado atual das listas de blocos livres.             |
    |---------------------------------------------------------------------------|
    | init(&mut self, heap_start: usize, heap_size: usize)                      |
    | -- Mapeia a memória do heap e popula as free_lists dividindo em blocos.   |
    |---------------------------------------------------------------------------|
    | push_front(&mut self, k: usize, block_addr: usize)                        |
    | -- Insere um bloco de ordem k no início da lista encadeada free_lists[k]. |
    |---------------------------------------------------------------------------|
    | pop_front(&mut self, k: usize) -> Option<usize>                           |
    | -- Remove e retorna o primeiro bloco livre disponível na ordem k.         |
    |---------------------------------------------------------------------------|
    | remove_from_list(&mut self, k: usize, target_addr: usize) -> bool         |
    | -- Remove um bloco específico (target_addr) da lista de ordem k.          |
    |---------------------------------------------------------------------------|
    | alloc(&mut self, layout: Layout) -> *mut u8                               |
    | -- Encontra/divide um bloco ideal para suprir o Layout solicitado.        |
    |---------------------------------------------------------------------------|
    | dealloc(&mut self, ptr: *mut u8, layout: Layout)                          |
    | -- Libera o bloco apontado e realiza fusão (coalescing) com seu Buddy.    |
    |---------------------------------------------------------------------------|
    | alloc_zeroed(&mut self, layout: Layout) -> *mut u8                        |
    | -- Aloca memória e garante que todo o conteúdo seja preenchido com zero.  |
    |---------------------------------------------------------------------------|
    | realloc(&mut self, ptr: *mut u8, layout: Layout, new_size: usize)         |
    | -- Realoca memória preservando os dados do bloco antigo para o novo.      |
    |---------------------------------------------------------------------------|
*/
pub struct MemAllocator {
    heap_start: usize,
    heap_end: usize,
    leaf_size: usize,
    leaf_maximum: usize,
    free_lists: [Option<usize>; 64],
}
impl MemAllocator {
    pub const fn empty() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            leaf_size: 64,
            leaf_maximum: 0,
            free_lists: [None; 64],
        }
    }
    
    pub fn log_free_list(&self) {
        for f in self.free_lists.into_iter().flatten() {
            info!("{:#?}", f);
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;

        info!("Heap start : {}", heap_start);
        info!("Heap size : {}", heap_size);

        let max: u32 = {
            let n: usize = heap_size / self.leaf_size;
            n.ilog2()
        };

        info!("leaf maximum size: {}", max);

        self.leaf_maximum = max as usize;

        let mut position = 0; // OFFSET relativo

        while position < heap_size {
            let remaining = heap_size - position;

            let k_size = (remaining.ilog2() as usize) - 6;
            let k_align = if position == 0 {
                self.leaf_maximum
            } else {
                (position.trailing_zeros() as usize) - 6
            };

            let k = k_size.min(k_align).min(self.leaf_maximum);
            let block_size = 64 << k;

            let block_addr = heap_start + position; // Endereço físico
            let node_ptr: *mut FreeNode = block_addr as *mut FreeNode; //Converte o endereço para um ponteiro bruto

            let old_head: Option<usize> = self.free_lists[k]; //Lê quem ocupa esse espaço

            unsafe {
                node_ptr.write(FreeNode {
                    next: old_head,
                    prev: None,
                });

                if let Some(old_addr) = old_head {
                    let old_none = &mut *(old_addr as *mut FreeNode);
                    old_none.prev = Some(block_addr);
                }
            }

            self.free_lists[k] = Some(block_addr);

            position += block_size;
        }

        self.log_free_list();
    }

    pub unsafe fn push_front(&mut self, k: usize, block_addr: usize) {
        let old_head = self.free_lists[k];
        let node_ptr = block_addr as *mut FreeNode;

        unsafe {
            node_ptr.write(FreeNode {
                next: old_head,
                prev: None,
            });
        }

        if let Some(old_addr) = old_head {
            unsafe {
                let old_node = &mut *(old_addr as *mut FreeNode);
                old_node.prev = Some(block_addr);
            }
        }

        self.free_lists[k] = Some(block_addr);
    }

    unsafe fn pop_front(&mut self, k: usize) -> Option<usize> {
        let head_addr: usize = self.free_lists[k]?;

        let head_node: &mut FreeNode = unsafe { &mut *(head_addr as *mut FreeNode) };

        let next_addr: Option<usize> = head_node.next;
        self.free_lists[k] = next_addr;

        if let Some(next_ptr) = next_addr {
            let next_node = unsafe { &mut *(next_ptr as *mut FreeNode) };
            next_node.prev = None;
        }

        Some(head_addr)
    }

    unsafe fn remove_from_list(&mut self, k: usize, target_addr: usize) -> bool {
        let mut current: Option<usize> = self.free_lists[k];

        while let Some(curr_addr) = current {
            let curr_node: &mut FreeNode = unsafe { &mut *(curr_addr as *mut FreeNode) };

            if curr_addr == target_addr {
                // Unlink
                let prev_addr: Option<usize> = curr_node.prev;
                let next_addr: Option<usize> = curr_node.next;

                if let Some(p) = prev_addr {
                    let prev_node: &mut FreeNode = unsafe { &mut *(p as *mut FreeNode) };
                    prev_node.next = next_addr;
                } else {
                    self.free_lists[k] = next_addr;
                }

                if let Some(n) = next_addr {
                    let next_node: &mut FreeNode = unsafe { &mut *(n as *mut FreeNode) };
                    next_node.prev = prev_addr;
                }
                return true;
            }
            current = curr_node.next;
        }

        false
    }

    /*
        |---------------------------------------------------------------------------|
        |                      Operação de Divisão (Splitting)                      |
        |---------------------------------------------------------------------------|
        | Quando não há blocos disponíveis na ordem requisitada (k_req), o alocador |
        | busca na ordem superior (current_k) e divide o bloco ao meio (Buddies):  |
        |                                                                           |
        | Ordem k+1:  [                   Bloco Original                    ]       |
        |                                       |                                   |
        |                                 (Divisão/Split)                           |
        |                                       v                                   |
        | Ordem k:    [   Bloco Alocado / Principal   ] [      Bloco Buddy      ]   |
        |                                                   (Enviado p/ free_list)  |
        |---------------------------------------------------------------------------|
    */
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size: usize = layout.size().max(layout.align()).max(self.leaf_size);
        // Arredondando para a próxima potência de 2
        let size_pow2: usize = size.next_power_of_two();
        let k_req: usize = (size_pow2.ilog2() as usize) - (self.leaf_size.ilog2() as usize);

        if k_req > self.leaf_maximum {
            return ptr::null_mut(); // Maior que o máximo
        }

        let mut current_k = k_req;
        while current_k <= self.leaf_maximum && self.free_lists[current_k].is_none() {
            current_k += 1;
        }

        if current_k > self.leaf_maximum {
            return ptr::null_mut(); //Out Of Memory (OOM)
        }

        let block_addr = unsafe { self.pop_front(current_k).unwrap() };

        while current_k > k_req {
            current_k -= 1;
            let buddy_addr = block_addr + (self.leaf_size << current_k);

            unsafe {
                self.push_front(current_k, buddy_addr);
            }
        }

        block_addr as *mut u8
    }

    /*
        |---------------------------------------------------------------------------|
        |                     Operação de Fusão (Coalescing)                        |
        |---------------------------------------------------------------------------|
        | Na desalocação, calcula-se o endereço do Buddy via operação XOR:          |
        |   buddy_offset = relative_offset ^ block_size                             |
        |                                                                           |
        | Se o Buddy estiver livre (encontrado em free_lists[k]):                   |
        | 1. Remove o Buddy da free_lists[k].                                       |
        | 2. Unifica os dois blocos formando um bloco de ordem superior (k + 1).    |
        | 3. Repete o processo iterativamente.                                      |
        |                                                                           |
        | Ordem k:    [ Bloco Desalocado ] + [ Buddy Livre Encontrado ]             |
        |                                    |                                      |
        |                                 (Fusão)                                   |
        |                                    v                                      |
        | Ordem k+1:  [                Bloco Fundido Resultante                 ]   |
        |---------------------------------------------------------------------------|
    */
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let mut block_addr: usize = ptr as usize;

        // Ordem K
        let size: usize = layout.size().max(layout.align()).max(self.leaf_size);
        let size_pow2 = size.next_power_of_two();
        let mut k = (size_pow2.ilog2() as usize) - (self.leaf_size.ilog2() as usize);

        // Tenta Fundir
        while k < self.leaf_maximum {
            let relative_offset = block_addr - self.heap_start;
            let block_size = self.leaf_size << k;
            let buddy_offset = relative_offset ^ block_size;
            let buddy_addr = self.heap_start + buddy_offset;

            let res: bool = unsafe { self.remove_from_list(k, buddy_addr) };

            if res {
                block_addr = block_addr.min(buddy_addr);
                k += 1;
            } else {
                // Bloco ocupado
                break;
            }
        }

        unsafe { self.push_front(k, block_addr) };
    }

    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> *mut u8 {
        let ptr: *mut u8 = unsafe { self.alloc(layout) };

        if !ptr.is_null() {
            unsafe {
                ptr::write_bytes(ptr, 0x0, layout.size());
            }
        }
        ptr
    }

    unsafe fn realloc(&mut self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        //bloco velho
        let old_size: usize = layout.size().max(layout.align()).max(self.leaf_size);
        let k_old =
            (old_size.next_power_of_two().ilog2() as usize) - (self.leaf_size.ilog2() as usize);

        //bloco novo
        let new_size_adj = new_size.max(layout.align()).max(self.leaf_size);
        let k_new =
            (new_size_adj.next_power_of_two().ilog2() as usize) - (self.leaf_size.ilog2() as usize);

        //O bloco atual já é suficiente
        if k_new <= k_old {
            return ptr;
        }

        let new_layout = match core::alloc::Layout::from_size_align(new_size, layout.align()) {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };

        let new_ptr = unsafe { self.alloc(new_layout) };

        if !new_ptr.is_null() {
            let copy_size = layout.size().min(new_size);

            unsafe {
                // Equivalente ao memcpy em C
                ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);

                self.dealloc(ptr, layout);
            }
        }

        new_ptr
    }
}
