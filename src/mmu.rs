pub fn from<T>(p: &T) -> &[u8] { unsafe{std::slice::from_raw_parts((p as *const T) as *const u8, std::mem::size_of::<T>())} }
pub fn as_u16(v:[u8;2]) -> u16 { unsafe{std::mem::transmute(v)} }
pub fn as_u32(v:[u8;4]) -> u32 { unsafe{std::mem::transmute(v)} }
pub fn as_u64(v:[u8;8]) -> u64 { unsafe{std::mem::transmute(v)} }

use std::collections::hash_map::Entry;
use crate::machine_state::MachineState;

const PAGE_SIZE: u64 = 4096;

impl MachineState {
    // Get host page from physical_address / PAGE_SIZE
    pub fn get_page(&mut self, cell: u64) -> &mut Vec<u8> {
        match self.memory.entry(cell) {
            Entry::Occupied(entry) => &mut *entry.into_mut(),
            Entry::Vacant(entry) => {
                let page = vec![0; PAGE_SIZE as usize];
                &mut *entry.insert(page)
            }
        }
    }

    fn translate_virtual_to_physical_address(&self, address: u64) -> u64 {
        let cr3 = self.cr3 as u64;
        if cr3 == 0 {
            address
        } else {
            // this code assumes that the guest operating system is using 2 megabyte pages
            // todo: check some MSR register what page size is actually used
            let page_address = address & 0b0000000000000000000000000000000000000000000111111111111111111111;
            let level3 = (address & 0b0000000000000000000000000000000000111111111000000000000000000000) >> 21;
            let level2 = (address & 0b0000000000000000000000000111111111000000000000000000000000000000) >> 30;
            let level1 = (address & 0b0000000000000000111111111000000000000000000000000000000000000000) >> 39;

            let entry = as_u64(self.read(cr3 + level1 * 8)) >> 12 << 12;
            let entry = as_u64(self.read(cr3 + level2 * 8)) >> 12 << 12;
            let entry = as_u64(self.read(cr3 + level3 * 8)) >> 12 << 12;
            entry + page_address
        }
    }

    pub fn mem_read(&self, virtual_address: u64, length: usize) -> Vec<u8> {
        let physical_address = self.translate_virtual_to_physical_address(virtual_address);
        let mut target = std::vec::from_elem(0, length);
        self.read_into(physical_address, &mut target);
        for ba in &self.break_on_access {
            if !(virtual_address > ba.1 as u64 || virtual_address+(length as u64) < ba.0) { println!("{:x}+{:x}: {:x?}", ba.0, virtual_address-ba.0, target); }
        }
        target
    }

    pub fn read<const N: usize>(&self, virtual_address: u64) -> [u8; N] {
        let mut buffer = {
            let mut buffer : [std::mem::MaybeUninit<u8>; N] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            //for i in 0..N { array[i] = std::mem::MaybeUninit::new(f(i)) }
            let ptr = &mut buffer as *mut _ as *mut [u8; N];
            let buffer_as_initialized = unsafe { ptr.read() };
            core::mem::forget(buffer);
            buffer_as_initialized
        };
        self.read_into(virtual_address, &mut buffer);
        buffer
    }

    fn read_into(&self, address: u64, target: &mut [u8]) {
        let mut page_number = address / PAGE_SIZE;
        let mut page_offset = address % PAGE_SIZE;
        let mut target_offset = 0;
        loop {
            let page = self.memory.get(&page_number).expect(&format!("{:x} {}",address, target.len()));

            loop {
                if target_offset >= target.len() { return; }
                if page_offset >= PAGE_SIZE {
                    page_number += 1;
                    page_offset = 0;
                    break;
                }

                target[target_offset] = page[page_offset as usize];

                target_offset += 1;
                page_offset += 1;
            }
        }
    }

    pub fn mem_read_byte(&mut self, address: u64) -> u8 { self.mem_read(address, 1)[0] }

    pub fn mem_write(&mut self, address: u64, data: &[u8]) {
        let address = self.translate_virtual_to_physical_address(address);
        self.mem_write_phys(address, data)
    }

    fn mem_write_phys(&mut self, address: u64, data: &[u8]) {
        /*const MEMORY_OFFSET: u64 = 0xB8000;
        if address >= MEMORY_OFFSET && address <= (MEMORY_OFFSET + 80 * 25 * 2) && address % 2 == 0{
            println!("VIDEO: {}", data[0] as char);
        }*/

        let mut page_number = address / PAGE_SIZE;
        let mut page_offset = address % PAGE_SIZE;
        let mut data_offset = 0;
        loop {
            let page = self.get_page(page_number);

            loop {
                if data_offset >= data.len() {
                    return;
                }
                if page_offset >= PAGE_SIZE {
                    page_number += 1;
                    page_offset = 0;
                    break;
                }

                page[page_offset as usize] = data[data_offset];

                data_offset += 1;
                page_offset += 1;
            }
        }
    }
}
