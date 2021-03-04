pub fn raw<T>(value: &T) -> &[u8] { unsafe{std::slice::from_raw_parts((value as *const T) as *const u8, std::mem::size_of::<T>())} }
pub fn raw_mut<T>(value: &mut std::mem::MaybeUninit<T>) -> &mut [u8] {
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
}

impl Memory {
    pub fn translate(&self, address: u64) -> u64 { address }
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

    pub fn write_aligned_bytes(&mut self, virtual_address: u64, bytes: &[u8]) {
        assert!(is_aligned(virtual_address, bytes.len()), "unaligned write {:x} {}", virtual_address, bytes.len());
        let physical_address = self.translate(virtual_address);
        let page = self.physical_to_host.get_mut(&(physical_address/PAGE_SIZE)).unwrap_or_else(|| panic!("write {:x} {}",physical_address, bytes.len()));
        let offset = (physical_address%PAGE_SIZE) as usize;
        page[offset..offset+bytes.len()].copy_from_slice(bytes);
    }

    pub fn read_byte(&self, virtual_address: u64) -> u8 { self.read_aligned(virtual_address, 1)[0] }
    pub fn write_byte(&mut self, virtual_address: u64, value: u8) { self.write_aligned_bytes(virtual_address, &[value]) }

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
    pub fn write<T>(&mut self, virtual_address: u64, value: &T) { self.write_aligned_bytes(virtual_address, raw(value)) }

    pub fn write_unaligned_bytes(&mut self, virtual_address: u64, bytes: &[u8]) {
        for (offset, &byte) in bytes.iter().enumerate() { self.write_byte(virtual_address+offset as u64, byte); }
    }
    pub fn write_unaligned<T>(&mut self, virtual_address: u64, value: &T) { self.write_unaligned_bytes(virtual_address, raw(value)) }

    fn get<T>(&self, base: i64, offset: i64) -> T { self.read_unaligned((base + offset) as u64) }
    pub fn get_i64(&self, base: i64, offset: i64) -> i64 { self.get(base, offset) }
    pub fn get_i32(&self, base: i64, offset: i64) -> i32 { self.get(base, offset) }
    pub fn get_i16(&self, base: i64, offset: i64) -> i16 { self.get(base, offset) }
    pub fn get_i8  (&self, base: i64, offset: i64) -> i8   { self.get(base, offset) }
    pub fn get_u8 (&self, base: i64, offset: i64) -> u8 { self.get(base, offset) }
}
