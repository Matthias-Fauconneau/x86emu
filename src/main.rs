#![feature(abi_efiapi, box_syntax,or_patterns)]
#[macro_use] extern crate bitflags;
fn address_of<T>(t:&T) -> u64 { t as *const T as u64 }
mod memory; use memory::{Memory, PAGE_SIZE, raw};
mod state; use state::State;
mod instruction; use instruction::{Opcode, Operand, Operands};
mod decoder; use decoder::decode;
mod interpreter;
mod dispatch; use dispatch::dispatch;

pub fn execute<R: addr2line::gimli::read::Reader, Host>
(state : &mut State, addr2line : &addr2line::Context<R>, traps: &fnv::FnvHashMap<u64, Box<dyn Fn(&mut State, &Host)->u64>>, host: &Host) {
    let mut instruction_cache = fnv::FnvHashMap::<u64,(Opcode, Operands, usize)>::default();
    loop {
        let instruction_start = state.rip as u64;

        let find_location = |address| {
            if let Ok(Some(location)) = addr2line.find_location(address) { format!("{}:{}", location.file.unwrap_or("").rsplit('/').next().unwrap(), location.line.unwrap_or(0)) }
            else { Default::default() }
        };

        let instruction = match instruction_cache.entry(instruction_start) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let instruction = entry.into_mut(); // Outlives entry unlike get
                state.rip += instruction.2 as i64;
                instruction
            },
            std::collections::hash_map::Entry::Vacant(slot) => {
                let instruction = decode(&mut state.rip, &state.memory); //.unwrap_or_else(|| panic!("{}", find_location(instruction_start).unwrap()));
                slot.insert((instruction.0, instruction.1, ((state.rip as u64) - instruction_start) as usize))
            }
        };

        let print_before = |state : &State, opcode, op : &Operands| {
            if let Some(op0 @ (Operand::Register{..} | Operand::EffectiveAddress{..})) = &op.operands.0 { println!("{}=0x{:x}", op0, state.get_value(&op0, op.size())); }
            if let Opcode::Mov = opcode {} // overwritten
            else if let Some(op1) = &op.operands.1 { println!("{}=0x{:x}", op1, state.get_value(&op1, op.size())); }
        };
        if state.print_instructions { print_before(&state, instruction.0, &instruction.1); }
        if state.print_instructions {
            print!("{} ", find_location(instruction_start));
            print!("{:x}: ", instruction_start);
            print!("{:x?}: ", &state.memory.read_bytes(instruction_start, instruction.2).collect::<Vec<_>>());
        }
        dispatch(state, instruction);
        let print_after = |state : &State, opcode, op : &Operands| {
            if let Opcode::Mov | Opcode::Movsx = opcode {} // unchanged
            else if let Some(op0 @ (Operand::Register{..} | Operand::EffectiveAddress{..})) = &op.operands.0 { println!("{}=0x{:x}", op0, state.get_value(&op0, op.size())); }
            if let Some(op1) = &op.operands.1 { println!("{}=0x{:x}", op1, state.get_value(&op1, op.size())); }
        };
        if state.print_instructions { print_after(&state, instruction.0, &instruction.1); }

        if let Some(closure) = traps.get(&(state.rip as u64)) {
            state.rax = closure(state, host) as i64;
            interpreter::ret(state);
        }
        assert!(state.rip as u64 != instruction_start, "{}", find_location(state.rip as u64));
        state.memory.try_read_aligned(state.rip as u64, 1).or_else(|| panic!("{:x} {}", instruction_start, find_location(instruction_start)) );
    }
}

pub fn stack_push_bytes(state: &mut State, bytes: &[u8]) {
    state.rsp -= ((bytes.len()+7)/8*8) as i64;
    state.memory.write_bytes(state.rsp as u64, bytes.iter().copied());
}
pub fn stack_push<T>(state: &mut State, value: &T) {
    assert!(raw(value).len()%8 == 0);
    stack_push_bytes(state, raw(value)); // todo opti: 64bit aligned
}
fn cast_slice<T,F>(from: &[F]) -> &[T] { unsafe{std::slice::from_raw_parts(from.as_ptr() as *const T, from.len() * std::mem::size_of::<F>() / std::mem::size_of::<T>())} }
pub fn stack_push_slice<T>(state: &mut State, value: &[T]) {
    stack_push_bytes(state, cast_slice(value));
}

fn cast_pointer_to_reference_to_same_type_as_value<'t, T>(ptr : i64, _: T) -> &'t T { unsafe{&*(ptr as *const T)} }

// stack push value
// \return stack reference
// \note macro to avoid state borrow
macro_rules! push { ($state:expr, $value:expr) => ({
    stack_push(&mut $state, &$value);
    $crate::cast_pointer_to_reference_to_same_type_as_value($state.rsp, $value)
})}

fn cast_pointer_to_slice_of_same_type_and_len_as_slice<'t, T>(ptr : i64, slice: &[T]) -> &[T] {
    unsafe{std::slice::from_raw_parts(ptr as *const T, slice.len())}
}

// stack push slice
// \return stack reference
// \note macro to avoid state borrow
macro_rules! push_slice { ($state:expr, $slice:expr) => ({
    stack_push_slice(&mut $state, $slice);
    cast_pointer_to_slice_of_same_type_and_len_as_slice($state.rsp, $slice)
})}

mod uefi;

fn main() {
    let mut memory = Memory::default();
    let file = std::env::args().skip(1).next().unwrap();
    let file = std::fs::read(file).unwrap();
    let object = addr2line::object::File::parse(&file).unwrap();
    let addr2line = addr2line::Context::new(&object).unwrap();
    let rip = { // 140001000~140300000
        let pe = (if let goblin::Object::PE(pe) = goblin::Object::parse(&file).unwrap() { Some(pe) } else { None }).unwrap();
        for section in pe.sections {
            let page_base = memory.translate(pe.image_base as u64+section.virtual_address as u64)/PAGE_SIZE;
            for (page_index, page) in file[section.pointer_to_raw_data as usize..][..section.size_of_raw_data as usize].chunks(PAGE_SIZE as usize).enumerate() {
                let mut page = page.to_vec();
                page.resize(PAGE_SIZE as usize, 0);
                memory.physical_to_host.insert(page_base+page_index as u64, page);
            }
        }
        pe.image_base as u64 + pe.entry as u64 // address_of_entry_point
    };

    let stack_base = 0x8000_0000_0000;
    let stack_size : usize = 0x100000;
    memory.host_allocate_physical(stack_base-(stack_size as u64), stack_size); // 64KB stack

    let heap_base = 0x8000_0000_0000;
    let heap_next = AtomicUsize::new(0);
    let heap_size = 0x100000;
    memory.host_allocate_physical(heap_base, heap_size);

    let mut state = State{memory, rsp: stack_base as i64, rip: rip as i64, print_instructions: false, ..Default::default()};

    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
    type Host = AtomicUsize;
    let mut traps : fnv::FnvHashMap<u64, Box<dyn Fn(&mut state::State, &Host)->u64>> = fnv::FnvHashMap::default();

    // UEFI
    use crate::uefi::*;
    let stdin = push!(state, new_input());
    let output_data = push!(state, new_output_data());
    let stdout = push!(state, new_output(&output_data));
    let stderr = push!(state, new_output(&output_data));
    let runtime_services = push!(state, new_runtime_services());
    let boot_services = push!(state, new_boot_services());
    let system_table = push!(state, new_system_table(&stdin, &stdout, &stderr, &runtime_services, &boot_services));

    traps.insert(state.memory.read(address_of(&stdout.output_string)), box |state,_|{
        let (_self, string) = (state.rcx, state.rdx); //EFI ABI = MS x64 = RCX, RDX, R8, R9
        let end = {let mut ptr = string; while state.memory.read_unaligned::<u16>(ptr as u64) != 0 { ptr += 2; } ptr};
        let bytes = state.memory.read_bytes(string as u64, (end-string) as usize).collect::<Vec<u8>>();
        use std::io::Write;
        std::io::stdout().write_all(String::from_utf16(&cast_slice(&bytes)).unwrap().as_bytes()).unwrap();
        0
    });

    let load_options = "\0".encode_utf16().collect::<Vec<u16>>();
    let load_options = push_slice!(state, &load_options);
    let loaded_image = push!(state, new_loaded_image(load_options));

    let simple_file_system = push!(state, new_simple_file_system());
    traps.insert(state.memory.read(address_of(&boot_services.handle_protocol)), box move |state,_|{
        let (handle, _protocol_guid, out_protocol) = (state.rcx, state.rdx, state.r8);
        /*match handle {
            image_handle => &address_of(loaded_image)),
            simple_file_system => &address_of(simple_file_system)
            _ => unimplemented!()
        }*/
        state.memory.write(out_protocol as u64, &handle); // Singletons
        0
    });

    traps.insert(state.memory.read(address_of(&boot_services.allocate_pool)), box |state,heap_next|{
        let (_pool_type, size, out_buffer) = (state.rcx, state.rdx, state.r8); // MemoryType, usize, &mut *mut u8
        state.memory.write(out_buffer as u64, &heap_next.fetch_add(size as usize, Relaxed));
        0
    });

    traps.insert(state.memory.read(address_of(&simple_file_system.open_volume)), box move |state,_|{
        let (_this, _root) = (state.rcx, state.rdx);
        unimplemented!();
    });

    traps.insert(state.memory.read(address_of(&boot_services.locate_handle)), box move |state,_|{
        let (_type, _guid, _key, out_buffer_size, buffer) = (state.rcx, state.rdx, state.r8, state.r9, state.memory.read::<u64>(state.rsp as u64+8));
        let size : u64 = state.memory.read(out_buffer_size as u64);
        state.memory.write(out_buffer_size as u64, &1);
        if size == 0 { // Only return number of handles
            assert!(buffer == 0);
        } else { // Assumes SimpleFileSystem
            assert!(buffer != 0);
            state.memory.write(buffer, simple_file_system);
        }
        0
    });
    state.rcx = address_of(loaded_image) as i64;
    state.rdx = address_of(system_table) as i64;
    execute(&mut state, &addr2line, &traps, &heap_next);
}
