/*#[macro_use] extern crate bitflags;
#[macro_use] extern crate syscall;
mod utils;
mod debug;
mod machine_state;
mod instructions;
mod decoder;
mod instruction_set;
mod mmu;*/

pub fn execute(file: &str, _print_instructions: bool, _print_registers: bool) {
    let file = std::fs::read(file).unwrap();
    let pe = (if let goblin::Object::PE(pe) = goblin::Object::parse(&file).unwrap() { Some(pe) } else { None }).unwrap();
    println!("{:#?}", &pe);
    /*let mut machine_state = machine_state::MachineState::new();
    for section in pe.sections {
        let start = section.pointer_to_raw_data as usize;
        machine_state.mem_write(/*image_base*/section.virtual_address as u64, &file[start..start+section.size_of_raw_data as usize]);
    }
    machine_state.rip = pe.entry as i64;
    machine_state.rsp = 0x7fffffffe018;
    machine_state.stack_push(&utils::convert_i64_to_u8vec(1));
    machine_state.print_instructions = print_instructions;
    machine_state.print_registers = print_registers;
    let mut cpu = instructions::EmulationCPU{};
    let mut decoder = decoder::Decoder::new(&mut cpu, &mut machine_state);
    decoder.execute();*/
}
