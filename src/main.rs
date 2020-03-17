#![feature(abi_efiapi, const_fn)]
#[macro_use] extern crate bitflags;
mod utils;
mod mmu;
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
    let mut cpu = instructions::EmulationCPU{};
    context.rsp = 0x7fffffffe018; // ?
    fn from<T: Sized>(p: &T) -> &[u8] { unsafe{std::slice::from_raw_parts((p as *const T) as *const u8, std::mem::size_of::<T>())} }
    // Setup system table
    let system_table = {use crate::uefi::*;
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
        //context.break_on_access.push((context.rsp as u64, from(&system_table).len()));
        context.rsp // "Allocated on the stack"
    };
    // Emulate call to efi_main(image_handle: Handle, system_table: SystemTable<Boot>) from UEFI (EFI ABI = MS x64 = RCX, RDX, R8, R9)
    //machine_state.rcx = image_handle;
    context.rdx = system_table;
    context.print_instructions = false;
    let mut decoder = decoder::Decoder::new(&mut cpu, &mut context);
    decoder.execute(false);
}
