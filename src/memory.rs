pub fn /*to_*/raw<T>(value: &T) -> &[u8] { unsafe{std::slice::from_raw_parts((value as *const T) as *const u8, std::mem::size_of::<T>())} }
pub fn /*to_*/raw_mut<T>(value: &mut std::mem::MaybeUninit<T>) -> &mut [u8] {
    unsafe{std::slice::from_raw_parts_mut(value.as_mut_ptr() as *mut u8, std::mem::size_of::<T>())}
}
pub fn from_raw<T>(raw : &[u8]) -> T {
    let mut value = std::mem::MaybeUninit::uninit();
    raw_mut(&mut value).copy_from_slice(raw);
    unsafe{value.assume_init()}
}

pub const PAGE_SIZE: u64 = 0x1000;
fn is_aligned(virtual_address: u64, size: usize) -> bool { size.is_power_of_two() && virtual_address%(size as u64)==0 && size<=PAGE_SIZE as usize }

#[derive(Default)]
pub struct Memory {
    pub physical_to_host: fnv::FnvHashMap<u64, Vec<u8>>
    //pub cr3: i64,
}

impl Memory {
    pub fn translate(&self, address: u64) -> u64 {
        address
        /*let cr3 = self.cr3 as u64;
        if cr3 == 0 {
            address
        } else {
            unimplemented!();
            /*// this code assumes that the guest operating system is using 2 megabyte pages
            // todo: check some MSR register what page size is actually used
            let page_address = address & 0b0000000000000000000000000000000000000000000111111111111111111111;
            let level3 = (address & 0b0000000000000000000000000000000000111111111000000000000000000000) >> 21;
            let level2 = (address & 0b0000000000000000000000000111111111000000000000000000000000000000) >> 30;
            let level1 = (address & 0b0000000000000000111111111000000000000000000000000000000000000000) >> 39;

            let entry = as_u64(self.read(cr3 + level1 * 8)) >> 12 << 12;
            let entry = as_u64(self.read(cr3 + level2 * 8)) >> 12 << 12;
            let entry = as_u64(self.read(cr3 + level3 * 8)) >> 12 << 12;
            entry + page_address*/
        }*/
    }
}

impl Memory {
    pub fn host_allocate_physical(&mut self, physical_address: u64, size: usize) {
        for page_index in physical_address/PAGE_SIZE..(physical_address+(size as u64)+PAGE_SIZE-1)/PAGE_SIZE {
            self.physical_to_host.insert(page_index, vec![0; PAGE_SIZE as usize]);
        }
    }

    fn try_read_aligned_physical(&self, physical_address: u64, size: usize) -> Option<&[u8]> {
        assert!(is_aligned(physical_address, size), "unaligned read {:x} {}", physical_address, size);
        let page = self.physical_to_host.get(&(physical_address/PAGE_SIZE))?;
        let offset = (physical_address%PAGE_SIZE) as usize;
        Some(&page[offset..offset+size])
    }

    pub fn try_read_aligned(&self, virtual_address: u64, size: usize) -> Option<&[u8]> {
        self.try_read_aligned_physical(self.translate(virtual_address), size)
    }
    fn read_aligned(&self, virtual_address: u64, size: usize) -> &[u8] {
        self.try_read_aligned(virtual_address, size).unwrap_or_else(|| panic!("read {:x} {}", virtual_address, size))
    }

    pub fn write_aligned(&mut self, virtual_address: u64, value: &[u8]) {
        assert!(is_aligned(virtual_address, value.len()), "unaligned write {:x} {}", virtual_address, value.len());
        let physical_address = self.translate(virtual_address);
        let page = self.physical_to_host.get_mut(&(physical_address/PAGE_SIZE)).unwrap_or_else(|| panic!("write {:x} {}",physical_address, value.len()));
        let offset = (physical_address%PAGE_SIZE) as usize;
        page[offset..offset+value.len()].copy_from_slice(value);
    }

    pub fn read_byte(&self, virtual_address: u64) -> u8 { self.read_aligned(virtual_address, 1)[0] }
    pub fn write_byte(&mut self, virtual_address: u64, value: u8) { self.write_aligned(virtual_address, raw(&value)) }

    pub fn mem_read_byte(&self, virtual_address: u64) -> u8 { self.read_byte(virtual_address) }

    pub fn read<T>(&self, virtual_address: u64) -> T { from_raw(self.read_aligned(virtual_address, std::mem::size_of::<T>())) }

    pub fn read_unaligned<T>(&self, virtual_address: u64) -> T {
        let size = std::mem::size_of::<T>();
        let line_size = size.next_power_of_two();
        let offset = (virtual_address%line_size as u64) as usize;
        let split = line_size-offset;
        let line = self.read_aligned((virtual_address                         )/line_size as u64*line_size as u64, line_size);
        let mut value = std::mem::MaybeUninit::uninit();
        if split > size { raw_mut(&mut value).copy_from_slice(&line[offset..][..size]); }
        else {
            raw_mut(&mut value)[..split].copy_from_slice(&line[offset..]);
            let line = self.read_aligned((virtual_address+size as u64)/line_size as u64*line_size as u64, line_size);
            raw_mut(&mut value)[split..].copy_from_slice(&line[..size-split]);
        }
        unsafe{value.assume_init()}
    }

    /*fn read_unaligned(&self, virtual_address: u64, target: &mut [u8]) {
        assert_eq!(virtual_address%8==0 && target.len() <= 8); // Unaligned access might stride guest pages
        let physical_address = self.translate_virtual_to_physical_address(virtual_address);
        let page = self.physical_to_host.get(&(physical_address/PAGE_SIZE)).expect(&format!("{:x} {}",physical_address, target.len()));
        let offset = (physical_address%PAGE_SIZE) as usize;
        target.copy_from_slice(page[offset..offset+target.len()]);
    }*/

    /*pub fn mem_read(&self, virtual_address: u64, size: usize) -> Vec<u8> {
        let mut target = std::vec::from_elem(0, size);
        self.read_into(physical_address, &mut target);
        for ba in &self.break_on_access {
            if !(virtual_address > ba.1 as u64 || virtual_address+(size as u64) < ba.0) { println!("{:x}+{:x}: {:x?}", ba.0, virtual_address-ba.0, target); }
        }
        target
    }*/
}

pub struct Bytes<'t> { memory : &'t Memory, virtual_address : u64, size : usize }
impl Iterator for Bytes<'_> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 { return None; }
        let byte = self.memory.read_byte(self.virtual_address);
        self.size -= 1;
        self.virtual_address += 1;
        Some(byte)
    }
}
impl Memory {
    pub fn read_bytes(&self, virtual_address: u64, size: usize) -> Bytes { Bytes{memory: &self, virtual_address, size} }

    pub fn mem_read(&self, virtual_address: u64, size: usize) -> Vec<u8> {
        self.read_aligned(self.translate(virtual_address), size).to_vec()
    }

    /*pub fn read<const N: usize>(&self, virtual_address: u64) -> [u8; N] {
        let mut buffer = {
            let mut buffer : [std::mem::MaybeUninit<u8>; N] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            //for i in 0..N { array[i] = std::mem::MaybeUninit::new(f(i)) }
            let ptr = &mut buffer as *mut _ as *mut [u8; N];
            let buffer_as_initialized = unsafe { ptr.read() };
            core::mem::forget(buffer);
            buffer_as_initialized
        };
        //self.read_into(virtual_address, &mut buffer);
        buffer.copy_from_slice(self.read_aligned(virtual_address, N));
        buffer
    }*/

    pub fn write<T>(&mut self, virtual_address: u64, value: &T) { self.write_aligned(virtual_address, raw(value)) }
    //pub fn write<T>(&mut self, virtual_address: u64, value: T) { self.write_aligned(virtual_address, raw(&value)) }

    //pub fn write_bytes(&mut self, virtual_address: u64, data: &[u8]) { for offset in 0..data.len() { self.write_aligned(virtual_address+offset as u64, &[data[offset]]) } }
    pub fn write_bytes<Bytes:IntoIterator<Item=u8>>(&mut self, virtual_address: u64, bytes: Bytes) {
        for (offset, byte) in bytes.into_iter().enumerate() { self.write_byte(virtual_address+offset as u64, byte); }
    }

    //pub fn mem_write(&mut self, virtual_address: u64, data: &[u8]) { self.write_bytes(virtual_address, data); }

    fn get<T>(&self, base: i64, offset: i64) -> T { self.read_unaligned((base + offset) as u64) }
    pub fn get_i64(&self, base: i64, offset: i64) -> i64 { self.get(base, offset) }
    pub fn get_i32(&self, base: i64, offset: i64) -> i32 { self.get(base, offset) }
    pub fn get_i16(&self, base: i64, offset: i64) -> i16 { self.get(base, offset) }
    pub fn get_i8  (&self, base: i64, offset: i64) -> i8   { self.get(base, offset) }
    pub fn get_u8 (&self, base: i64, offset: i64) -> u8 { self.get(base, offset) }
}
