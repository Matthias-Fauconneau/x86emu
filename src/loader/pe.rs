use crate::machine_state::MachineState;
use crate::decoder::Decoder;
use crate::cpu::emu_instructions::EmulationCPU;
use crate::utils::convert_i64_to_u8vec;

pub fn execute(file: &str, print_instructions: bool, print_registers: bool) {
    let file = std::fs::read(file).unwrap();
    let pe = (if let goblin::Object::PE(pe) = goblin::Object::parse(&file).unwrap() { Some(pe) } else { None }).unwrap();
    println!("{:#?}", &pe);
    let mut machine_state = MachineState::new();
    for section in pe.sections {
        let start = section.pointer_to_raw_data as usize;
        machine_state.mem_write(/*image_base*/section.virtual_address as u64, &file[start..start+section.size_of_raw_data as usize]);
    }
    machine_state.rip = pe.entry as i64;
    machine_state.rsp = 0x7fffffffe018;
    machine_state.stack_push(&convert_i64_to_u8vec(1));
    machine_state.print_instructions = print_instructions;
    machine_state.print_registers = print_registers;
    let mut cpu = EmulationCPU{};
    let mut decoder = Decoder::new(&mut cpu, &mut machine_state);
    decoder.execute();
}
