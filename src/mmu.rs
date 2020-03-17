use std::collections::hash_map::{Entry};
use crate::machine_state::MachineState;

use zero;

const PAGE_SIZE: u64 = 4096;

impl MachineState {
    fn get_page(&mut self, cell: u64) -> &mut Vec<u8> {
        match self.memory.entry(cell) {
            Entry::Occupied(entry) => &mut *entry.into_mut(),
            Entry::Vacant(entry) => {
                let page = vec![0; PAGE_SIZE as usize];
                &mut *entry.insert(page)
            }
        }
    }

    fn translate_virtual_to_physical_address(&mut self, address: u64) -> u64 {
        let cr3 = self.cr3 as u64;
        if cr3 == 0 {
            address
        } else {
            // this code assumes that the operating system is using 2 megabyte pages
            // todo: check some MSR register what page size is actually used
            let page_address = address & 0b0000000000000000000000000000000000000000000111111111111111111111;
            let level3 = (address & 0b0000000000000000000000000000000000111111111000000000000000000000) >> 21;
            let level2 = (address & 0b0000000000000000000000000111111111000000000000000000000000000000) >> 30;
            let level1 = (address & 0b0000000000000000111111111000000000000000000000000000000000000000) >> 39;

            let entry = self.mem_read_phys(cr3 + level1 * 8, 8);
            let entry = *zero::read::<u64>(&entry) >> 12 << 12;

            let entry = self.mem_read_phys(entry + level2 * 8, 8);
            let entry = *zero::read::<u64>(&entry) >> 12 << 12;

            let entry = self.mem_read_phys(entry + level3 * 8, 8);
            let entry = *zero::read::<u64>(&entry) >> 12 << 12;

            entry + page_address
        }
    }

    // FIXME: fast path for fixed length (e.g 1 byte)
    pub fn mem_read(&mut self, virtual_address: u64, length: u64) -> Vec<u8> {
        let physical_address = self.translate_virtual_to_physical_address(virtual_address);
        let data = self.mem_read_phys(physical_address, length);
        for ba in &self.break_on_access {
            if !(virtual_address > ba.1 as u64 || virtual_address+length < ba.0) { println!("{:x}+{:x}: {:x?}", ba.0, virtual_address-ba.0, data); }
        }
        data
    }

    fn mem_read_phys(&mut self, address: u64, length: u64) -> Vec<u8> {
        let mut page_number = address / PAGE_SIZE;
        let mut page_offset = address % PAGE_SIZE;
        let mut data_offset = 0;
        let mut data = Vec::new();
        loop {
            let page = self.get_page(page_number);

            loop {
                if data_offset >= length {
                    return data;
                }
                if page_offset >= PAGE_SIZE {
                    page_number += 1;
                    page_offset = 0;
                    break;
                }

                data.push(page[page_offset as usize]);

                data_offset += 1;
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
