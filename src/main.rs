#[macro_use] extern crate bitflags;
mod utils;
mod mmu;
mod instruction_set;
mod machine_state;
mod instructions;
mod decoder;

fn main() {
   let mut machine_state = machine_state::MachineState::new();
   machine_state.print_instructions = true;
   //machine_state.print_registers = true;
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
            machine_state.mem_write(address, &file[start..start+size]);
        }
        println!("{:x} {:x}", pe.image_base, pe.entry);
        machine_state.rip = (pe.image_base as u64 + pe.entry as u64) as i64; // address_of_entry_point
    }
    machine_state.rsp = 0x7fffffffe018; // ?
    //machine_state.stack_push(&utils::convert_i64_to_u8vec(1));
    let mut cpu = instructions::EmulationCPU{};
    let mut decoder = decoder::Decoder::new(&mut cpu, &mut machine_state);
    decoder.execute();
    println!("OK");
}
