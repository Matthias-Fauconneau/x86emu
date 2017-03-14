use std::fs::File;
use std::io::Read;
use std::env;

extern crate zero;
use zero::read_str;

extern crate xmas_elf;
use xmas_elf::{ElfFile, program, sections};
use xmas_elf::symbol_table::Entry;

extern crate x86emu;
use x86emu::cpu::emu_instructions::EmulationCPU;
use x86emu::machine_state::MachineState;
use x86emu::decoder::Decoder;

fn main() {
    let filename = match env::args().nth(1) {
        Some(filename) => filename,
        None => {
            println!("Usage: cargo run --bin elf <program>");
            return;
        }
    };

    let symbol_name = match env::args().nth(2) {
        Some(symbol_name) => symbol_name,
        None => "main".to_string(),
    };

    let mut file = File::open(filename).expect("Cannot open file");
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).expect("Failed to read file.");

    let elf_file = ElfFile::new(&buffer);

    // get address where to load the text section
    let load_address = get_load_address(&elf_file).expect("can not get load address");

    // get the binary code from the .text section
    let text_section = elf_file.find_section_by_name(".text").expect("text section not found.");
    let code = text_section.raw_data(&elf_file);
    let code_offset = text_section.offset();

    // get the virtual address of the main function
    let (main_symbol_address, main_size) = get_main_symbol_address(&elf_file, &symbol_name);
    // get the offset of the main function
    let offset = main_symbol_address - code_offset - load_address;

    let mut end = (offset + main_size) as usize;
    if end >= code.len() {
        end = code.len() - 1
    }
    let main_code = &code[offset as usize..end];

    let mut cpu = EmulationCPU{};
    let mut machine_state = MachineState::new(main_code.to_vec());
    let mut decoder = Decoder::new(&mut cpu, &mut machine_state);
    decoder.execute();
}

fn get_load_address(elf_file: &ElfFile) -> Option<u64> {
    for sect in elf_file.program_iter() {
        let t = sect.get_type().unwrap();
        match t {
            program::Type::Load if sect.flags() & program::FLAG_X == program::FLAG_X => {
                return Some(sect.virtual_addr());
            }
            _ => {}
        }
    }
    return None;
}

fn get_main_symbol_address(elf_file: &ElfFile, symbol_name: &str) -> (u64, u64) {
    let symbol_string_table = elf_file.find_section_by_name(".strtab")
        .expect("strtab (String table) section not found, is this a stripped binary?");
    let symbol_string_table = symbol_string_table.raw_data(&elf_file);

    let symbol_table = elf_file.find_section_by_name(".symtab")
        .expect("symtab (Symbol table) section not found");
    if let sections::SectionData::SymbolTable64(data) = symbol_table.get_data(&elf_file).unwrap() {
        let symbol =
            data.iter()
                .find(|&symbol| {
                    read_str(&symbol_string_table[symbol.name() as usize..]) == symbol_name
                })
                .expect("symbol not found");
        if symbol.size() == 0 {
            return (symbol.value(), u64::max_value());
        } else {
            return (symbol.value(), symbol.size());
        }
    } else {
        unreachable!();
    };
}
