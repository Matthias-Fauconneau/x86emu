#![feature(destructuring_assignment, type_ascription)]
mod memory; pub use memory::PAGE_SIZE;
mod state; pub use state::State;
mod instruction; use instruction::{Opcode, Operands};
mod decoder; use decoder::decode;
mod interpreter;
mod dispatch; use dispatch::dispatch;

impl State {
	pub fn execute(&mut self) {
		let mut instruction_cache = fnv::FnvHashMap::<u64,(Opcode, Operands, usize)>::default();
		while self.rip != !0 {
			let instruction_start = self.rip as u64;
			let instruction = match instruction_cache.entry(instruction_start) {
				std::collections::hash_map::Entry::Occupied(entry) => {
					let instruction = entry.into_mut();
					self.rip += instruction.2 as i64;
					instruction
				},
				std::collections::hash_map::Entry::Vacant(slot) => {
					let instruction = decode(&mut self.rip, &self.memory);
					slot.insert((instruction.0, instruction.1, ((self.rip as u64) - instruction_start) as usize))
				}
			};
			dispatch(self, instruction);
		}
	}
}

pub fn allocate_stack(state: &mut State) {
	const STACK_BASE : u64 = 0x8000_0000_0000;
	const STACK_SIZE : usize = 0x0000_0010_0000;
	state.memory.host_allocate_physical(STACK_BASE-(STACK_SIZE as u64), STACK_SIZE); // 64KB stack
	state.rsp = STACK_BASE as i64;
}

pub fn load(state: &mut State, function: &[u8]) {
	state.rip = {
		const LOADER_BASE : u64 = 0x00_0000;
		let (address, entry) = (0, 0);
		let image_base = state.memory.translate(LOADER_BASE+address)/PAGE_SIZE;
		for (page_index, page) in function.chunks(PAGE_SIZE as usize).enumerate() {
			let mut page = page.to_vec();
			page.resize(PAGE_SIZE as usize, 0); // Last piece
			state.memory.physical_to_host.insert(image_base+page_index as u64, page);
		}
		LOADER_BASE + entry
	} as i64;
}

pub struct Heap {
	next: u64
}
impl Heap {
	const HEAP_BASE : u64 = 0x2_0000_0000;
	const HEAP_SIZE : usize = 0x0_0010_0000;
	pub fn new(state: &mut State) -> Self {
		state.memory.host_allocate_physical(Self::HEAP_BASE, Self::HEAP_SIZE);
		Self{next: 0}
	}
	pub fn push_slice<T>(&mut self, state: &mut State, slice: &[T]) -> u64 {
		fn as_bytes<T>(slice: &[T]) -> &[u8] { unsafe{std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len() * std::mem::size_of::<T>())} }
		let offset = Self::HEAP_BASE+self.next;
		let slice = as_bytes(slice);
		state.memory.write_unaligned_bytes(offset, slice);
		self.next += slice.len() as u64;
		offset
	}
}

pub fn stack_push_bytes(state: &mut State, bytes: &[u8]) {
	state.rsp -= bytes.len() as i64;
	state.memory.write_aligned_bytes(state.rsp as u64, bytes);
}
pub fn stack_push<T>(state: &mut State, value: &T) {
	assert_eq!(std::mem::size_of::<T>()%8, 0);
	stack_push_bytes(state, memory::raw(value));
}

pub fn call(state: &mut State, args: &[i64], fargs: &[f32]) {
	(state.rdi, state.rsi/*, state.r8*//*, state.r9*/) = (args[0], args[1]/*, args[2]*/); //, args.get(3).unwrap_or_default()];
	(state.xmm[0], ) = (fargs[0].to_bits() as u128, );
	stack_push(state, &!0u64);
}
