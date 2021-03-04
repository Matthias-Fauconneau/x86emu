#![feature(destructuring_assignment)]
//use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
mod memory; pub use memory::PAGE_SIZE;
mod state; pub use state::State;
mod instruction; use instruction::{Opcode, Operands};
mod decoder; use decoder::decode;
mod interpreter;
mod dispatch; use dispatch::dispatch;

impl State {
	pub fn execute/*<'data,'file,Guest>*/(&mut self, //traps: &fnv::FnvHashMap<u64, Box<dyn Fn(&mut State, &mut Guest)->u64>>, guest: &mut Guest,
																	/*addr2line: addr2line::Context<impl addr2line::gimli::Reader>, sigint: Arc<AtomicBool>, object : &'file impl object::Object<'data,'file>,*/ /*image_base : u64*/)
																			{//-> Result<(), String> {
			let mut instruction_cache = fnv::FnvHashMap::<u64,(Opcode, Operands, usize)>::default();
			//let mut last_location = addr2line::Location{file: None, line: None, column:None};
			//while !sigint.load(Ordering::Relaxed) {
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

					/*if let Some(closure) = traps.get(&(self.rip as u64)) {
							self.rax = closure(self, guest) as i64;
							interpreter::ret(self);
					}*/
					/*else if trace { if let Some(location) = (self.find_location)(instruction_start) {
							if location != last_location {
									if let (Some(trim), Some(line)) = (location.file.find("pebble"), location.line) { println!("{}:{}", &location.file[trim..], line); }
									last_location = location;
							}
					} }*/
					/*else if let Ok(Some(location)) = addr2line.find_location(instruction_start) {
							if location.line != last_location.line {
									let file = location.file.unwrap();
									if let (Some(trim), Some(line)) = (file.find("pebble"), location.line) { println!("{}:{}", &file[trim..], line); }
									if let Some(56) = location.line { break; } // Breakpoint
									last_location = location;
							}
					}*/
					if self.rip as u64 == instruction_start { break; } //return Err(format!("{}", (self.find_location)(self.rip as u64))); }
					//self.memory.try_read_aligned(self.rip as u64, 1).or_else(|| panic!("{:x} {}", instruction_start, (self.find_location)(instruction_start)) );
			}
			/*{
					use gimli::{BaseAddresses, EhFrame, EndianSlice, NativeEndian, UninitializedUnwindContext,UnwindSection};
					let eh_frame = EhFrame::new(object.section_data_by_name(EhFrame::section_name())(), NativeEndian);
					let ctx = UninitializedUnwindContext::new();
					let bases = BaseAddresses::default()
							//.set_text(address_of_text_section_in_memory)
							//.set_got(address_of_got_section_in_memory)
					;
					let ip = state.rip as u64;
					loop {
							let unwind_info = eh_frame.unwind_info_for_address(&bases, &mut ctx, ip, EhFrame::cie_from_offset).unwrap();
							let ip = self.memory.read(rsp as u64);
							rsp = self.memory.read((rsp-8) as u64); // rbp
							println!("rsp {:x} ", rsp);
							print!("{:x}: ", ip);
							if let Ok(Some(location)) = addr2line.find_location(ip) { println!("{}:{}", &location.file.unwrap_or(""), location.line.unwrap_or(0)); }
					}
			}*/
			//use gimli::{read::Section, BaseAddresses, EhFrame, EndianSlice, NativeEndian, UninitializedUnwindContext,UnwindSection};
			/*use {object::ObjectSection, gimli::{EhFrame, NativeEndian, BaseAddresses}, unwind::dwarf::{Object, FallibleIterator, Unwinder, FrameIterator, Registers, X86_64}};
			let mut objects = Vec::new();
			objects.push(Object{ eh_frame: EhFrame::new(object.section_by_name(".eh_fram").unwrap().data().unwrap(), NativeEndian),
					bases: BaseAddresses::default()
													.set_eh_frame(image_base+object.section_by_name(".eh_fram").unwrap().address()) // What for ?
													.set_text(image_base+object.section_by_name(".text").unwrap().address())
			});
			println!("{:x} {:x}", image_base, object.section_by_name(".text").unwrap().address());
			let mut unwinder = Unwinder::new( objects );
			let mut registers : Registers = Default::default();
			registers[X86_64::RA] = Some(self.rip as u64);
			let mut frames = FrameIterator::new(&mut unwinder, registers);
			while let Some(frame) = frames.next().unwrap() {
					print!("{:x}", frames.registers[X86_64::RA].unwrap());
					if let Ok(Some(location)) = addr2line.find_location(frames.registers[X86_64::RA].unwrap()) {
							print!("{}:{}", &location.file.unwrap_or(""), location.line.unwrap_or(0));
					}
					println!("");
			}*/
			//Ok(())
	}
	pub fn call(&mut self, function: &[u8], args: &[i64], fargs: &[f32]) {
		self.rip = {
			static LOADER_BASE : u64 = 0x00_0000;
			let (address, entry) = (0, 0);
			let image_base = self.memory.translate(LOADER_BASE+address)/PAGE_SIZE;
			for (page_index, page) in function.chunks(PAGE_SIZE as usize).enumerate() {
				let mut page = page.to_vec();
				page.resize(PAGE_SIZE as usize, 0); // Last piece
				self.memory.physical_to_host.insert(image_base+page_index as u64, page);
			}
			LOADER_BASE + entry
		} as i64;
		static STACK_BASE : u64 = 0x8000_0000_0000;
    static STACK_SIZE : usize = 0x0000_0010_0000;
    self.memory.host_allocate_physical(STACK_BASE-(STACK_SIZE as u64), STACK_SIZE); // 64KB stack
    self.rsp = STACK_BASE as i64;
		(self.rcx, self.rdx/*, self.r8*//*, self.r9*/) = (args[0], args[1]/*, args[2]*/); //, args.get(3).unwrap_or_default()];
		(self.xmm[0], ) = (fargs[0], );
		self.execute(/*&[], &mut std::default::default(), &loader*/)
	}
}

// framework::core::memory|ffi
//use crate::memory::raw;
//unsafe fn cast_pointer_to_reference_to_same_type_as_value<T>(ptr : i64, _: T) -> &'static T { &*(ptr as *const T) }
//pub fn cast_slice<T,F>(from: &[F]) -> &[T] {  unsafe{std::slice::from_raw_parts(from.as_ptr() as *const T, from.len() * std::mem::size_of::<F>() / std::mem::size_of::<T>())} }
//unsafe fn cast_pointer_to_slice_of_same_type_and_len_as_slice<T:'static>(ptr : i64, slice: &[T]) -> &'static [T] { std::slice::from_raw_parts(ptr as *const T, slice.len()) }

/*impl State {
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
}*/
