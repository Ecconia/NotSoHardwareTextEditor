use sdl2::keyboard::{Mod, Scancode};
use crate::config::{CHAR_HEIGHT, CHAR_WIDTH, NEWLINE};
use crate::cursor::CursorPointer;
use crate::font::Font;
use crate::memory::Memory;

pub struct LetterInstruction {
	pub pos_x: usize,
	pub pos_y: usize,
	pub id: u8,
}

pub struct Typewriter {
	//Cursor pointer:
	pub cursor: CursorPointer,
	pub memory: Memory,
	pub line_cache: [usize; CHAR_HEIGHT],
}

impl Default for Typewriter {
	fn default() -> Self {
		Self {
			cursor: CursorPointer::default(),
			memory: Memory::default(),
			line_cache: [0; CHAR_HEIGHT],
		}
	}
}

impl Typewriter {
	pub fn handle_input(
		&mut self,
		scancode: &Scancode,
		keymod: &Mod,
		letter_instructions: &mut Vec<LetterInstruction>,
		font: &Font,
	) {
		// println!("{} - {}", self.memory.pointer_before_cursor, self.memory.pointer_after_cursor);
		let typed_letter = self.resolve_symbol(scancode, keymod);
		if typed_letter.is_some() {
			if self.memory.memory_full() {
				//TODO: Bell.
				return;
			}
			
			let letter = &typed_letter.unwrap();
			//Add the letter to memory:
			self.memory.insert(*font.ids_by_key.get(letter).unwrap());
			//Now draw the letter:
			
			if self.cursor.is_at_end() {
				println!(">>> AT END");
				//Cursor is at the end of the display.
				//While one could write to this position, afterwards a line-wrap would occur, overwriting the previous data.
				
				//Set the cursor to the beginning of the line, as it would end up there.
				self.cursor.to_line_beginning();
				//Then clear the now empty new line:
				self.clear_from_cursor(letter_instructions);
				// //Finally redraw the memory which is before the cursor:
				self.update_line_cache(); //First create a line cache, to know where the last lines ends.
				// // The last line ofc ends at the end of the display. But treat it like everything else and just redraw:
				self.clear_above_cursor(letter_instructions);
				self.redraw_before_cursor(letter_instructions);
			} else {
				//Cursor is at a valid position on the display, where we intend to write to.
				
				//Write to valid position, clear the position first:
				self.write_space(letter_instructions);
				self.write_letter_by_key(font, letter_instructions, letter);
				//Cursor must now advance to the next position:
				self.cursor.increment();
			}
			
			if !self.memory.cursor_at_end() && !self.cursor.is_at_end() {
				self.clear_from_cursor(letter_instructions);
				self.clear_below_cursor(letter_instructions);
				self.redraw_from_cursor(letter_instructions);
			}
			return;
		}
		
		match scancode {
			Scancode::Backspace => {
				if self.memory.cursor_at_beginning() {
					//TODO: Bell.
					return;
				}
				self.memory.delete_backwards();
				let deleted_value = self.memory.memory[self.memory.pointer_before_cursor];
				if deleted_value == NEWLINE {
					//Clear the current line, as we are at the beginning of it and move the whole content up.
					// It either redraws or whatever.
					for x in 0..CHAR_WIDTH {
						letter_instructions.push(LetterInstruction {
							pos_x: x,
							pos_y: self.cursor.y,
							id: 0,
						});
					}
					
					let x_pos = self.find_line_end();
					self.cursor.x = x_pos;
					self.cursor.to_previous_line();
				} else {
					self.cursor.decrement();
				}
				
				if !self.memory.cursor_at_end() {
					self.redraw_from_cursor(letter_instructions);
				} else {
					self.write_space(letter_instructions);
				}
			}
			Scancode::Left => {
				if self.memory.cursor_at_beginning() {
					//TODO: Bell.
					return;
				}
				self.cursor.decrement();
				self.memory.move_after_cursor();
			}
			Scancode::Right => {
				if self.memory.cursor_at_end() {
					//TODO: Bell.
					return;
				}
				self.cursor.increment();
				self.memory.move_before_cursor();
			}
			Scancode::Return => {
				if self.memory.memory_full() {
					//TODO: Bell
					return; //Memory full, cannot insert
				}
				
				//Inject the newline into memory:
				self.memory.insert(NEWLINE);
				
				if self.cursor.is_last_line() {
					//At the end of display, stuff has to be shifted!
					
					//Move cursor to the beginning of the line, as a new line starts now:
					self.cursor.to_line_beginning();
					
					self.update_line_cache();
					self.clear_above_cursor(letter_instructions);
					self.redraw_before_cursor(letter_instructions);
				} else {
					//Not at the end of display, just use the next line!
					
					//Remove from cursor to the rest of the line, in case that the cursor is not at the end of the line:
					self.clear_from_cursor(letter_instructions);
					//Move cursor to the beginning of the line, as a new line starts now:
					self.cursor.to_line_beginning();
					
					//Go to next line:
					self.cursor.to_next_line();
				}
				
				//Clear everything after the cursor:
				self.clear_from_cursor(letter_instructions);
				self.clear_below_cursor(letter_instructions);
				
				if !self.memory.cursor_at_end() {
					self.redraw_from_cursor(letter_instructions);
				}
			}
			_ => {}
		}
	}
	
	pub fn dump_debug(&self) {
		println!("Status:");
		println!(" Mem: {} - {}", self.memory.pointer_before_cursor, self.memory.pointer_after_cursor);
		println!(" Cur: {} {}", self.cursor.x, self.cursor.y);
	}
	
	fn clear_from_cursor(&self, letter_instructions: &mut Vec<LetterInstruction>) {
		for x in self.cursor.x..CHAR_WIDTH {
			letter_instructions.push(LetterInstruction {
				pos_x: x,
				pos_y: self.cursor.y,
				id: 0,
			});
		}
	}
	
	fn clear_below_cursor(&self, letter_instructions: &mut Vec<LetterInstruction>) {
		for y in (self.cursor.y + 1)..CHAR_HEIGHT {
			for x in 0..CHAR_WIDTH {
				letter_instructions.push(LetterInstruction {
					pos_x: x,
					pos_y: y,
					id: 0,
				});
			}
		}
	}
	
	fn clear_above_cursor(&self, letter_instructions: &mut Vec<LetterInstruction>) {
		for y in 0..self.cursor.y {
			for x in 0..CHAR_WIDTH {
				letter_instructions.push(LetterInstruction {
					pos_x: x,
					pos_y: y,
					id: 0,
				});
			}
		}
	}
	
	fn update_line_cache(&mut self) {
		let mut mem_index = self.memory.pointer_before_cursor - 1;
		let mut line_index = 0;
		let mut counter = 0;
		let mut visible_lines = 0;
		
		println!("Building line cache:");
		loop {
			let value = self.memory.memory[mem_index];
			// println!(":: {}", value);
			if value == NEWLINE || mem_index == 0 {
				self.line_cache[line_index] = counter;
				println!("- {}: {}", line_index, counter);
				counter = 0;
				line_index += 1;
				visible_lines += 1;
				if visible_lines >= CHAR_HEIGHT {
					break;
				}
			} else {
				counter += 1;
				if counter == CHAR_WIDTH {
					counter = 0;
					visible_lines += 1;
				}
			}
			
			if mem_index == 0 {
				break;
			}
			mem_index -= 1;
		}
	}
	
	fn find_line_end(&self) -> usize {
		let mem = &self.memory;
		let mut counter = 0;
		let mut index = mem.pointer_before_cursor;
		while index != 0 {
			index -= 1;
			
			let value = mem.memory[index];
			if value == NEWLINE {
				return counter;
			}
			
			counter += 1;
			if counter == CHAR_WIDTH {
				counter = 0;
			}
		}
		counter
	}
	
	//Call-Requirement:
	// Pointer must not be at beginning of memory!
	// Cursor should be at beginning of the line? Or not?
	// Cursor must not be in first line.
	fn redraw_before_cursor(&mut self, letter_instructions: &mut Vec<LetterInstruction>) {
		println!("Readrawing!");
		self.cursor.backup();
		// self.cursor.to_previous_line();
		
		let mut line_index = 0;
		
		let cache_number = self.line_cache[line_index];
		let cache_number_off = (if cache_number == 0 { CHAR_WIDTH } else { cache_number }) - 1; //Correct the X position - required or not.
		if cache_number != self.cursor.x {
			println!("Ehm, by design line cache should end at cursor position, it was off though. Cursor {}, cache {}, off {}.", self.cursor.x, cache_number, cache_number_off);
			self.cursor.x = cache_number_off;
		}
		
		// Set the cursor position to where ???
		
		if self.line_cache[line_index] == 0 {
			// Need to go back 1 horizontal line (if possible):
			if self.cursor.is_first_line() {
				self.cursor.restore();
				return;
			}
			self.cursor.to_previous_line();
		}
		println!("> Cur-Pos: {}", self.cursor.x);
		
		let mut pointer = self.memory.pointer_before_cursor; //Get the stack position
		pointer -= 1; //But decrement it once, as the value is relevant
		
		loop {
			let value = self.memory.memory[pointer];
			if value == NEWLINE {
				//Encountered newline.
				println!("> NL");
				line_index += 1;
				if line_index == CHAR_HEIGHT {
					println!(">> Last line.");
					break; //Drawn last line already.
				}
				//Move cursor:
				if !self.cursor.is_at_line_end() {
					if self.cursor.is_first_line() {
						println!(">> Cursor first line.");
						break; //Already at top line.
					}
					self.cursor.to_previous_line();
				}
				self.cursor.x = self.line_cache[line_index] - 1;
			} else {
				println!("> L: {}", value);
				// self.write_space(letter_instructions);
				self.write_letter_by_id(letter_instructions, value);
				//Move cursor:
				if self.cursor.is_at_start() {
					println!(">> Cursor at start.");
					break; //Drawn everything that fits
				}
				self.cursor.decrement();
			}
			
			if pointer == 0 {
				break; //Done processing memory.
			}
			pointer -= 1;
		}
		
		self.cursor.restore();
	}
	
	fn redraw_from_cursor(&mut self, letter_instructions: &mut Vec<LetterInstruction>) {
		self.cursor.backup();
		
		//By calling requirement, it is guaranteed that the after-caret stack is not empty.
		let mut pointer = self.memory.pointer_after_cursor;
		pointer += 1; //Jump to the first value instead of the free slot.
		
		//First clear up the first slot, as that is done a cycle before the actual drawing:
		// self.write_space(letter_instructions);
		
		loop {
			//Get value from after-caret stack:
			let value = self.memory.memory[pointer];
			//Draw that symbol, but first clear the area:
			self.write_letter_by_id(letter_instructions, value);
			
			if self.cursor.is_at_end() {
				break; //Reached the end of display, no space to draw more.
			}
			//Advance to the next position on the display:
			self.cursor.increment();
			//We got more things to write, clear up the next (or last) slot:
			// self.write_space(letter_instructions);
			
			if pointer == (self.memory.memory.len() - 1) {
				break; //Reached end of memory, nothing more to draw.
			}
			//Advance to the next position on the stack:
			pointer += 1;
		}
		
		self.cursor.restore();
	}
	
	fn write_space(&self, letter_instructions: &mut Vec<LetterInstruction>) {
		self.write_letter_by_id(letter_instructions, 0);
	}
	
	fn write_letter_by_key(&self, font: &Font, letter_instructions: &mut Vec<LetterInstruction>, letter_key: &char) {
		self.write_letter_by_id(letter_instructions, *font.ids_by_key.get(letter_key).unwrap());
	}
	
	fn write_letter_by_id(&self, letter_instructions: &mut Vec<LetterInstruction>, letter_id: u8) {
		letter_instructions.push(LetterInstruction {
			pos_x: self.cursor.x,
			pos_y: self.cursor.y,
			id: letter_id,
		});
	}
	
	fn resolve_symbol(&mut self, keycode: &Scancode, keymod: &Mod) -> Option<char> {
		let raw_keycode = *keycode as u32;
		let is_shift = (*keymod & Mod::LSHIFTMOD) == Mod::LSHIFTMOD || (*keymod & Mod::RSHIFTMOD) == Mod::RSHIFTMOD;
		
		if is_shift {
			if raw_keycode >= Scancode::A as u32 && raw_keycode <= Scancode::Z as u32 {
				let mut offset = raw_keycode - Scancode::A as u32;
				//Hacky way of applying the German YZ swap code:
				if offset == ('z' as u32 - 'a' as u32) {
					offset = 'y' as u32 - 'a' as u32;
				} else if offset == ('y' as u32 - 'a' as u32) {
					offset = 'z' as u32 - 'a' as u32;
				}
				return Some(char::from_u32(offset + 'A' as u32).expect("This should not fail"));
			}
			match keycode {
				Scancode::Space => Some(' '),
				
				Scancode::Num1 => Some('!'),
				Scancode::Num2 => Some('"'),
				Scancode::Num7 => Some('/'),
				Scancode::Num8 => Some('('),
				Scancode::Num9 => Some(')'),
				Scancode::Num0 => Some('='),
				
				Scancode::Period => Some(':'),
				Scancode::Backslash => Some('\''),
				Scancode::RightBracket => Some('*'),
				
				Scancode::Minus => Some('?'),
				_ => {
					// println!("Uff: {:x?}", keycode);
					None
				}
			}
		} else {
			if raw_keycode >= Scancode::A as u32 && raw_keycode <= Scancode::Z as u32 {
				let mut offset = raw_keycode - Scancode::A as u32;
				//Hacky way of applying the German YZ swap code:
				if offset == ('z' as u32 - 'a' as u32) {
					offset = 'y' as u32 - 'a' as u32;
				} else if offset == ('y' as u32 - 'a' as u32) {
					offset = 'z' as u32 - 'a' as u32;
				}
				return Some(char::from_u32(offset + 'a' as u32).expect("This should not fail"));
			} else if raw_keycode >= Scancode::Num1 as u32 && raw_keycode <= Scancode::Num9 as u32 {
				let offset = raw_keycode - Scancode::Num1 as u32;
				return Some(char::from_u32(offset + '1' as u32).expect("This should not fail"));
			}
			match keycode {
				Scancode::Space => Some(' '),
				
				Scancode::Num0 => Some('0'),
				
				Scancode::Comma => Some(','),
				Scancode::Period => Some('.'),
				Scancode::Slash => Some('-'),
				
				Scancode::RightBracket => Some('+'),
				_ => {
					// println!("Uff: {:?}", keycode);
					None
				}
			}
		}
	}
}
