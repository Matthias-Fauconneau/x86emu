use crate::{state::State, instruction::{Opcode, Operands}, decoder::decode, dispatch::dispatch, interpreter};

impl State {
    pub fn execute<Guest>(&mut self, traps: &fnv::FnvHashMap<u64, Box<dyn Fn(&mut State, &mut Guest)->u64>>, guest: &mut Guest, trace: bool) -> Result<(), String> {
        let mut instruction_cache = fnv::FnvHashMap::<u64,(Opcode, Operands, usize)>::default();
        let mut last_location = Default::default();
        loop {
            let instruction_start = self.rip as u64;
            let instruction = match instruction_cache.entry(instruction_start) {
                std::collections::hash_map::Entry::Occupied(entry) => {
                    let instruction = entry.into_mut(); // Outlives entry unlike get
                    self.rip += instruction.2 as i64;
                    instruction
                },
                std::collections::hash_map::Entry::Vacant(slot) => {
                    let instruction = decode(&mut self.rip, &self.memory); //.unwrap_or_else(|| panic!("{}", find_location(instruction_start).unwrap()));
                    slot.insert((instruction.0, instruction.1, ((self.rip as u64) - instruction_start) as usize))
                }
            };

            /*let print_before = |self : &State, opcode, op : &Operands| {
                if let Some(op0 @ (Operand::Register{..} | Operand::EffectiveAddress{..})) = &op.operands.0 { println!("{}=0x{:x}", op0, self.get_value(&op0, op.size())); }
                if let Opcode::Mov = opcode {} // overwritten
                else if let Some(op1) = &op.operands.1 { println!("{}=0x{:x}", op1, self.get_value(&op1, op.size())); }
            };
            if self.print_instructions { print_before(&self, instruction.0, &instruction.1); }
            if self.print_instructions {
                print!("{} ", find_location(instruction_start));
                print!("{:x}: ", instruction_start);
                print!("{:x?}: ", &self.memory.read_bytes(instruction_start, instruction.2).collect::<Vec<_>>());
            }*/
            dispatch(self, instruction);
            /*let print_after = |opcode, op : &Operands| {
                if let Opcode::Mov | Opcode::Movsx = opcode {} // unchanged
                else if let Some(op0 @ (Operand::Register{..} | Operand::EffectiveAddress{..})) = &op.operands.0 { println!("{}=0x{:x}", op0, self.get_value(&op0, op.size())); }
                if let Some(op1) = &op.operands.1 { println!("{}=0x{:x}", op1, self.get_value(&op1, op.size())); }
            };*/
            //if self.print_instructions { print_after(&self, instruction.0, &instruction.1); }

            if let Some(closure) = traps.get(&(self.rip as u64)) {
                self.rax = closure(self, guest) as i64;
                interpreter::ret(self);
            } else if trace {
                let location = (self.find_location)(instruction_start);
                if location != last_location {
                    println!("{} ", location);
                    last_location = location;
                }
            }
            if self.rip as u64 == instruction_start { return Err(format!("{}", (self.find_location)(self.rip as u64))); }
            self.memory.try_read_aligned(self.rip as u64, 1).or_else(|| panic!("{:x} {}", instruction_start, (self.find_location)(instruction_start)) );
        }
        //Ok(())
    }
}

// framework::core::memory|ffi
use crate::memory::raw;
unsafe fn cast_pointer_to_reference_to_same_type_as_value<T>(ptr : i64, _: T) -> &'static T { &*(ptr as *const T) }
pub fn cast_slice<T,F>(from: &[F]) -> &[T] {  unsafe{std::slice::from_raw_parts(from.as_ptr() as *const T, from.len() * std::mem::size_of::<F>() / std::mem::size_of::<T>())} }
unsafe fn cast_pointer_to_slice_of_same_type_and_len_as_slice<T:'static>(ptr : i64, slice: &[T]) -> &'static [T] { std::slice::from_raw_parts(ptr as *const T, slice.len()) }

impl State {
    pub fn stack_push_bytes(&mut self, bytes: &[u8]) {
        self.rsp -= ((bytes.len()+7)/8*8) as i64;
        self.memory.write_bytes(self.rsp as u64, bytes.iter().copied());
    }

    pub fn stack_push<T>(&mut self, value: &T) {
        //assert_eq!(std::mem::size_of::<T>(), 8);
        assert_eq!(std::mem::size_of::<T>()%8, 0);
        self.stack_push_bytes(raw(value)); // todo opti: 64bit aligned
    }

    pub unsafe fn push<T:'static>(&mut self, value: T) -> &'static T {
        self.stack_push(&value);
        cast_pointer_to_reference_to_same_type_as_value(self.rsp, value)
    }

    pub fn stack_push_slice<T>(&mut self, value: &[T]) {
        self.stack_push_bytes(cast_slice(value));
    }

    pub unsafe fn push_slice<T:'static>(&mut self, slice: &[T]) -> &'static [T] {
        self.stack_push_slice(slice);
        cast_pointer_to_slice_of_same_type_and_len_as_slice(self.rsp, slice)
    }
}
