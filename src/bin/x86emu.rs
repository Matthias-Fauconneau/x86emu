fn main() {
    /*let matches = clap::App::new("x86emu")
        .arg(clap::Arg::with_name("file").required(true))
        .arg(clap::Arg::with_name("debug")
            .help("run in debug mode (print all registers after every instruction)")
            .long("debug")
            .short("d"))
        .arg(clap::Arg::with_name("print-instructions")
            .help("print every executed instruction")
            .long("print-instructions")
            .short("p"))
        .get_matches();
    let filename = matches.value_of("file").unwrap();
    let debug = matches.is_present("debug");
    let print_instructions = matches.is_present("print-instructions");
    x86emu::loader::pe::execute(filename, print_instructions, debug);*/
    x86emu::loader::pe::execute(&std::env::args().next().unwrap(), false, false);
}
