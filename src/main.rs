#![feature(abi_efiapi, box_syntax,or_patterns)]
#[macro_use] extern crate bitflags;
fn address_of<T>(t:&T) -> u64 { t as *const T as u64 }
mod memory; use memory::PAGE_SIZE;
mod state; use state::State;
mod instruction;
mod decoder;
mod interpreter;
mod dispatch;
mod guest; use guest::cast_slice;
mod uefi;

fn main() -> Result<(),String> {
    let file = std::env::args().skip(1).next().unwrap();
    let file = std::fs::read(file).unwrap();
    let object = addr2line::object::File::parse(&file).unwrap();
    let addr2line = addr2line::Context::new(&object).unwrap();

    let mut state = State::new(box move |address| {
        if let Ok(Some(location)) = addr2line.find_location(address) { format!("{}:{}", location.file.unwrap_or("").rsplit('/').next().unwrap(), location.line.unwrap_or(0)) }
        else { Default::default() }
    });
    state.rsp = STACK_BASE as i64;

    //static LOADER_BASE : u64 = 0x1_0000_0000;
    //static LOADER_SIZE : usize = 0x1_0000_0000;

    static HEAP_BASE : u64 = 0x2_0000_0000;
    static HEAP_SIZE : usize = 0x0_0010_0000;
    state.memory.host_allocate_physical(HEAP_BASE, HEAP_SIZE);

    static BOOK_BASE : u64 = 0x8_0000_0000/PAGE_SIZE;
    //static BOOK_SIZE : usize = 0x7_0000_0000/PAGE_SIZE;

    static STACK_BASE : u64 = 0x8000_0000_0000;
    static STACK_SIZE : usize = 0x0000_0010_0000;
    state.memory.host_allocate_physical(STACK_BASE-(STACK_SIZE as u64), STACK_SIZE); // 64KB stack

    state.rip = {
        let pe = (if let goblin::Object::PE(pe) = goblin::Object::parse(&file).unwrap() { Some(pe) } else { None }).unwrap();
        for section in pe.sections {
            let image_base = state.memory.translate(pe.image_base as u64+section.virtual_address as u64)/PAGE_SIZE;
            for (page_index, page) in file[section.pointer_to_raw_data as usize..][..section.size_of_raw_data as usize].chunks(PAGE_SIZE as usize).enumerate() {
                let mut page = page.to_vec();
                page.resize(PAGE_SIZE as usize, 0);
                state.memory.physical_to_host.insert(image_base+page_index as u64, page);
            }
        }
        pe.image_base as u64 + pe.entry as u64 // address_of_entry_point
    } as i64;

    #[derive(Default)]
    struct Guest {
        heap_next: usize,
        book_next: usize,
    }
    let mut traps : fnv::FnvHashMap<u64, Box<dyn Fn(&mut state::State, &mut Guest)->u64>> = fnv::FnvHashMap::default();

    use crate::uefi::*;
    unsafe { // Guest stack frame : Typed local variables invalid to be accessed but through state.read|write(address_of(struct.field), ..)
        //let stdin = state.push( new_input() );
        //let output_data = state.push( new_output_data() );
        let (stdin, output_data) = state.push( (new_input(), new_output_data()) );
        let stdout = state.push( new_output(&output_data ));
        let stderr = state.push( new_output(&output_data) );
        let runtime_services = state.push( new_runtime_services() );
        let boot_services = state.push( new_boot_services() );
        let system_table = state.push( new_system_table(&stdin, &stdout, &stderr, &runtime_services, &boot_services) );
        let load_options = "fs0:\\efiloader.efi kernel=kernel.elf image.simple_fb=simple_fb.elf fb.width=1920 fb.height=1080".encode_utf16().collect::<Vec<u16>>();
        let load_options = state.push_slice( &load_options );
        let loaded_image = state.push( new_loaded_image(load_options) );
        let file_impl = state.push( new_file_impl() );
        let simple_file_system = state.push( new_simple_file_system() );

    traps.insert(state.memory.read(address_of(&stdout.output_string)), box |state,_|{
        let (_self, string) = (state.rcx, state.rdx); //EFI ABI = MS x64 = RCX, RDX, R8, R9
        let end = {let mut ptr = string; while state.memory.read_unaligned::<u16>(ptr as u64) != 0 { ptr += 2; } ptr};
        let bytes = state.memory.read_bytes(string as u64, (end-string) as usize).collect::<Vec<u8>>();
        use std::io::Write;
        std::io::stdout().write_all(String::from_utf16(&cast_slice(&bytes)).unwrap().as_bytes()).unwrap();
        0
    });

    traps.insert(state.memory.read(address_of(&file_impl.get_info)), box |state,host|{
        let (self_, _information_type, _buffer_size, buffer) = (state.rcx, state.rdx, state.r8, state.r9); // &Guid, &mut usize, *mut u8
        state.memory.write_bytes(buffer as u64, cast_slice(&"BOOT".encode_utf16().collect::<Vec<_>>()).iter().copied());
        0
        //panic!("{}", host.find_location(state.memory.read(state.rsp as u64)));
    });
    traps.insert(state.memory.read(address_of(&file_impl.close)), box |state,host|{ 0 });

    traps.insert(state.memory.read(address_of(&simple_file_system.open_volume)), box move |state,_|{
        let (_handle, out_root) = (state.rcx, state.rdx);
        state.memory.write(out_root as u64, &file_impl);
        0
    });

    traps.insert(state.memory.read(address_of(&boot_services.handle_protocol)), box move |state,_|{
        let (handle, _protocol_guid, out_protocol) = (state.rcx, state.rdx, state.r8);
        state.memory.write(out_protocol as u64, &handle); // Singletons
        0
    });

    traps.insert(state.memory.read(address_of(&boot_services.allocate_pool)), box |state, host|{
        let (_pool_type, size, out_buffer) = (state.rcx, state.rdx, state.r8); // MemoryType, usize, &mut *mut u8
        state.memory.write(out_buffer as u64, &(HEAP_BASE+host.heap_next as u64));
        host.heap_next += size as usize;
        0
    });

    traps.insert(state.memory.read(address_of(&boot_services.free_pool)), box |state,heap_next|{
        let _buffer = state.rcx; // *mut u8
        0
    });

    traps.insert(state.memory.read(address_of(&boot_services.locate_handle)), box move |state,_|{
        let (_type, _guid, _key, out_buffer_size, buffer) = (state.rcx, state.rdx, state.r8, state.r9, state.memory.read::<u64>(state.rsp as u64+0x28)); // return, shadow, align?
        let size : u64 = state.memory.read(out_buffer_size as u64);
        state.memory.write(out_buffer_size as u64, &std::mem::size_of::<::uefi::Handle>());
        if size == 0 { // Only return number of handles
            //assert!(buffer == 0);
        } else { // Assumes SimpleFileSystem
            assert!(buffer != 0);
            state.memory.write(buffer, &address_of(simple_file_system));
        }
        0
    });

    traps.insert(state.memory.read(address_of(&boot_services.allocate_pages)), box |state,host|{
        let (_alloc_type, _memory_type, count, out_address) = (state.rcx, state.rdx, state.r8, state.r9); // u32, MemoryType, usize, &mut u64
        let count = count as usize;
        assert!(state.memory.read::<u64>(out_address as u64) == 0); // AnyAddress
        for page_index in 0..count { state.memory.physical_to_host.insert(BOOK_BASE+(host.book_next+page_index) as u64, vec![0; PAGE_SIZE as usize]); }
        state.memory.write(out_address as u64, &((BOOK_BASE+host.book_next as u64) * PAGE_SIZE));
        host.book_next += count;
        0
    });

    traps.insert(state.memory.read(address_of(&boot_services.set_mem)), box |state,_| {
        let (buffer, len, value) = (state.rcx, state.rdx, state.r8); // buffer, usize, u8
        for i in 0..len { state.memory.write_byte((buffer+i) as u64, value as u8); }
        0
    });

    state.rcx = address_of(loaded_image) as i64;
    state.rdx = address_of(system_table) as i64;
    }
    //state.print_instructions = true;
    state.execute(&traps, &mut Default::default())
}
