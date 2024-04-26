use crate::config::MEMORY_SIZE;

pub struct Memory {
	pub memory: [u8; MEMORY_SIZE],
	pub pointer_before_cursor: usize,
	pub pointer_after_cursor: usize,
}

impl Default for Memory {
	fn default() -> Self {
		let memory = [0; MEMORY_SIZE];
		Self {
			memory,
			pointer_before_cursor: 0,
			pointer_after_cursor: memory.len() - 1,
		}
	}
}

impl Memory {
	pub fn insert(&mut self, id: u8) {
		self.memory[self.pointer_before_cursor] = id;
		self.pointer_before_cursor += 1;
	}
	
	pub fn delete_backwards(&mut self) {
		self.pointer_before_cursor -= 1;
	}
	
	pub fn cursor_at_beginning(&self) -> bool {
		self.pointer_before_cursor == 0
	}
	
	pub fn memory_full(&self) -> bool {
		self.pointer_before_cursor == self.pointer_after_cursor + 1
	}
	
	pub fn cursor_at_end(&self) -> bool {
		self.pointer_after_cursor == self.memory.len() - 1
	}
	
	//Cursor left:
	pub fn move_after_cursor(&mut self) {
		self.pointer_before_cursor -= 1;
		self.memory[self.pointer_after_cursor] = self.memory[self.pointer_before_cursor];
		self.pointer_after_cursor -= 1;
	}
	
	//Cursor right:
	pub fn move_before_cursor(&mut self) {
		self.pointer_after_cursor += 1;
		self.memory[self.pointer_before_cursor] = self.memory[self.pointer_after_cursor];
		self.pointer_before_cursor += 1;
	}
}
