#![allow(incomplete_features)]
#![feature(const_generics, abi_efiapi, box_syntax)]
#[macro_use] extern crate bitflags;
mod mmu; use mmu::{from, as_u16, as_u64};
mod instruction_set;
mod machine_state;
mod instructions;
mod decoder;
mod uefi;

fn main() {
   let mut context = machine_state::MachineState::new();
   {
        let file = std::env::args().skip(1).next().unwrap();
        let file = std::fs::read(file).unwrap();
        let pe = (if let goblin::Object::PE(pe) = goblin::Object::parse(&file).unwrap() { Some(pe) } else { None }).unwrap();
        for section in pe.sections {
            let address = pe.image_base as u64+section.virtual_address as u64;
            let size = section.size_of_raw_data as usize;
            let start = section.pointer_to_raw_data as usize;
            context.mem_write(address, &file[start..start+size]);
        }
        context.rip = (pe.image_base as u64 + pe.entry as u64) as i64; // address_of_entry_point
    }
    for i in 0..16 { context.get_page(0x7fffffff0+i); } // Host allocate physical pages for stack (allow to make context.read(&self ...) (not mut))
    context.rsp = 0x7fffffffe018; // ?
    // Setup system table
    let mut traps : fnv::FnvHashMap<u64, Box<dyn Fn(&mut machine_state::MachineState)>> = fnv::FnvHashMap::default();
    let system_table = {use crate::uefi::*;
        let top = context.rsp as u64;
        context.stack_push(from(&default_input()));
        let stdin = unsafe{&*(context.rsp as *const Input)};
        context.stack_push(from(&default_output(&default_output_data())));
        let stdout = unsafe{&*(context.rsp as *const Output)};
        context.stack_push(from(&default_output(&default_output_data())));
        let stderr = unsafe{&*(context.rsp as *const Output)};
        context.stack_push(from(&default_runtime_services()));
        let runtime_services = unsafe{&*(context.rsp as *const RuntimeServices)};
        context.stack_push(from(&default_boot_services()));
        let boot_services = unsafe{&*(context.rsp as *const BootServices)};
        let system_table = default_system_table(&stdin, &stdout, &stderr, &runtime_services, &boot_services);
        context.stack_push(from(&system_table));
        fn address_of<T>(t:&T) -> u64 { return t as *const T as u64; }
        let output_string = as_u64(context.read(address_of(&stdout.output_string)));
        traps.insert(output_string, box |context|{
            let (_self, string) = (context.rcx, context.rdx);
            let end = {let mut ptr = string; while as_u16(context.read(ptr as u64)) != 0 { ptr += 2; } ptr};
            use std::io::Write;
            // Assumes ASCII
            std::io::stdout().write_all(&context.mem_read(string as u64, (end-string) as usize).into_iter().step_by(2).collect::<Vec<u8>>()).unwrap();
            context.rax = 0;
            instructions::EmulationCPU{}.ret(context);
        });
        context.rsp // "Allocated on the stack"
    };
    // Emulate call to efi_main(image_handle: Handle, system_table: SystemTable<Boot>) from UEFI (EFI ABI = MS x64 = RCX, RDX, R8, R9)
    //machine_state.rcx = image_handle;
    context.rdx = system_table;
    context.print_instructions = false;
    let mut cpu = instructions::EmulationCPU{};
    let mut decoder = decoder::Decoder::new(&mut cpu, &mut context);
    decoder.execute(traps, false);
}
