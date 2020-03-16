fn main() {
    x86emu::execute(&std::env::args().next().unwrap(), false, false);
}
