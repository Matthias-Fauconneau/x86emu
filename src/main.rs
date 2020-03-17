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
        println!("{:#?}", &pe);
        for section in pe.sections {
            let address = pe.image_base as u64+section.virtual_address as u64;
            let size = section.size_of_raw_data as usize;
            println!("{:#?} {:?} {:x} {:x}", std::str::from_utf8(section.name.split(|&c|c==0).next().unwrap()).unwrap(), section.real_name, address, address+size as u64);
            let start = section.pointer_to_raw_data as usize;
            context.mem_write(address, &file[start..start+size]);
        }
        println!("{:x} {:x}", pe.image_base, pe.entry);
        context.rip = (pe.image_base as u64 + pe.entry as u64) as i64; // address_of_entry_point
    }
    //machine_state.stack_push(&utils::convert_i64_to_u8vec(1));
    let mut cpu = instructions::EmulationCPU{};
    context.rsp = 0x7fffffffe018; // ?
    // Emulate call to efi_main(image_handle: Handle, system_table: SystemTable<Boot>) from UEFI (EFI ABI = MS x64 = RCX, RDX, R8, R9)
    //machine_state.rcx = image_handle;
    let system_table = crate::uefi::default_system_table();
    fn from<T: Sized>(p: &T) -> &[u8] { unsafe{std::slice::from_raw_parts((p as *const T) as *const u8, std::mem::size_of::<T>())} }
    context.stack_push(from(&system_table));
    context.break_on_access.push((context.rsp as u64, from(&system_table).len()));
    let system_table = context.rsp; // "Allocated on the stack"
    context.rdx = system_table;
    let mut decoder = decoder::Decoder::new(&mut cpu, &mut context);
    decoder.execute(true);
    println!("OK");
}
