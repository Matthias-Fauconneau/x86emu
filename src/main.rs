#![allow(incomplete_features)]
#![feature(const_generics, abi_efiapi, box_syntax)]
#[macro_use] extern crate bitflags;
fn address_of<T>(t:&T) -> u64 { return t as *const T as u64; }
mod memory; use memory::raw;
mod state; use state::State;
mod instruction; use instruction::{Opcode, Instruction};
mod decoder; use decoder::decode;
mod interpreter;
mod dispatch; use dispatch::dispatch;

pub fn execute(state : &mut State, traps: &fnv::FnvHashMap<u64, Box<dyn Fn(&mut State)>>) {
    let mut instruction_cache = fnv::FnvHashMap::<u64,(Opcode, Instruction,usize)>::default();
    loop {
        let instruction_start = state.rip as u64;
        if let Some(closure) = traps.get(&instruction_start) {
            closure(state);
            continue;
        }
        //if state.read_bytes(state.rip as u64, 16) == [0; 16] { panic!("{:x} {:x?}", state.rip, traps.keys()); }
        let instruction = match instruction_cache.entry(instruction_start) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let instruction = entry.into_mut(); // Outlives entry unlike get
                state.rip += instruction.2 as i64;
                instruction
            },
            std::collections::hash_map::Entry::Vacant(slot) => {
                let instruction = decode(&mut state.rip, &state.memory);
                slot.insert((instruction.0, instruction.1, ((state.rip as u64) - instruction_start) as usize))
            }
        };

        if state.print_instructions { print!("{:x}: ", instruction_start); }
        dispatch(state, instruction);
    }
}

pub fn stack_push_unaligned<T>(state: &mut State, value: &T) {
    state.rsp -= std::mem::size_of::<T>() as i64;
    state.memory.write_bytes(state.rsp as u64, raw(value).iter().copied());
}
fn cast_to_same_type_as_value<'t, T>(ptr : i64, _: T) -> &'t T { unsafe{&*(ptr as *const T)} }
// stack push value
// \return stack reference
// \note macro to avoid state borrow
macro_rules! push { ($state:expr, $value:expr) => ({
    stack_push_unaligned(&mut $state, &$value);
    $crate::cast_to_same_type_as_value($state.rsp, $value)
})}

mod uefi;

fn main() {
    let mut state = State::default();
    state.print_instructions = false;
    let mut traps : fnv::FnvHashMap<u64, Box<dyn Fn(&mut state::State)>> = fnv::FnvHashMap::default();
    {
        let file = std::env::args().skip(1).next().unwrap();
        let file = std::fs::read(file).unwrap();
        let pe = (if let goblin::Object::PE(pe) = goblin::Object::parse(&file).unwrap() { Some(pe) } else { None }).unwrap();
        for section in pe.sections {
            let address = pe.image_base as u64+section.virtual_address as u64;
            let size = section.size_of_raw_data as usize;
            let start = section.pointer_to_raw_data as usize;
            state.memory.host_allocate_physical(address, size);
            state.memory.write_bytes(address, file[start..start+size].iter().copied());
        }
        state.rip = (pe.image_base as u64 + pe.entry as u64) as i64; // address_of_entry_point
    }
    let stack_end = 0x8000_0000_0000;
    let stack_size : usize = 0x10000;
    state.memory.host_allocate_physical(stack_end-(stack_size as u64), stack_size); // 64KB stack
    state.rsp =  stack_end as i64;
    // Emulate call to efi_main(image_handle: Handle, system_table: SystemTable<Boot>) from UEFI
    { // image handle
        let image_handle = 0;
        state.rcx = image_handle;
    }
    { // system table
        use crate::uefi::*;
        let stdin = push!(state, new_input());
        let output_data = push!(state, new_output_data());
        let stdout = push!(state, new_output(&output_data));
        let stderr = push!(state, new_output(&output_data));
        let runtime_services = push!(state, new_runtime_services());
        let boot_services = push!(state, new_boot_services());
        let system_table = push!(state, new_system_table(&stdin, &stdout, &stderr, &runtime_services, &boot_services));

        traps.insert(state.memory.read(address_of(&stdout.output_string)), box |state|{
            let (_self, string) = (state.rcx, state.rdx); //EFI ABI = MS x64 = RCX, RDX, R8, R9
            let end = {let mut ptr = string; while state.memory.read_unaligned::<u16>(ptr as u64) != 0 { ptr += 2; } ptr};
            use std::io::Write;
            // Assumes ASCII
            std::io::stdout().write_all(&state.memory.read_bytes(string as u64, (end-string) as usize).step_by(2).collect::<Vec<u8>>()).unwrap();
            state.rax = 0;
            interpreter::ret(state);
        });

        let loaded_image = push!(state, new_loaded_image());
        traps.insert(state.memory.read(address_of(&boot_services.handle_protocol)), box move |state|{
            let (_self, protocol_guid, out_procotol) = (state.rcx, state.rdx, state.r8);
            state.memory.write(out_procotol as u64, loaded_image);
            state.rax = 0;
            interpreter::ret(state);
        });

        state.rdx = address_of(system_table) as i64;
    }
    execute(&mut state, &traps);
}
