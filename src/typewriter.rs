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

macro_rules! assert_false {
	($falsey_condition:expr, $failure_message:expr) => {
		if $falsey_condition {
			println!($failure_message);
			return;
		}
	};
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
			self.memory.insert(*font.ids_by_key.get(letter).unwrap());
			//Only write a letter, when it visibly fits onto the canvas, else the move cursor redrawing logic will draw it.
			if !self.cursor.is_at_canvas_end() {
				self.write_space(letter_instructions);
				self.write_letter_by_key(font, letter_instructions, letter);
			}
			
			if self.cursor.is_at_canvas_end() {
				//First move the cursor to the start of the line:
				self.cursor.to_line_beginning();
				//Then clear basically the whole screen:
				self.clear_above_cursor(letter_instructions);
				self.clear_from_cursor(letter_instructions);
				//Update the line cache (to know where to put the cursor) and redraw the screen:
				self.update_line_cache();
				self.redraw_before_cursor(letter_instructions);
			} else {
				self.cursor.increment();
			}
			
			//As we added a letter, there is always the demand to redraw things after the cursor,
			// unless the cursor is at the end of the memory or at the end of canvas (stuff is out of bounds then):
			if !self.memory.cursor_at_end() && !self.cursor.is_at_canvas_end() {
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
				//When we remove a character, it has to be replaced with an empty space (whitespace).
				//That has to be manually drawn here, but:
				// - If the cursor is in column 0, a (invisible) newline will be deleted.
				// - If there is text after the cursor, the cursor movement code redraws that bit anyway.
				//The cursor movement logic, should not be bothered/polluted with non-redrawing display changes.
				if self.memory.cursor_at_end() && self.cursor.x != 0 {
					self.cursor.x -= 1;
					self.write_space(letter_instructions);
					self.cursor.x += 1;
				}
				//Now after the action performed, move the cursor back properly:
				self.move_cursor_back(letter_instructions, true);
			}
			Scancode::Left => {
				if self.memory.cursor_at_beginning() {
					//TODO: Bell.
					return;
				}
				self.memory.move_after_cursor();
				self.move_cursor_back(letter_instructions, false);
			}
			Scancode::Right => {
				if self.memory.cursor_at_end() {
					//TODO: Bell.
					return;
				}
				self.memory.move_before_cursor();
				
				let mut scroll_into_next_line = false;
				if self.memory.memory[self.memory.pointer_after_cursor] == NEWLINE {
					//We skipped a newline while going forward, thus we need to go to the next line:
					if self.cursor.is_last_line() {
						//We are in the last line, scroll all content up:
						scroll_into_next_line = true;
					} else {
						//We are somewhere, just move cursor down one row:
						self.cursor.to_line_beginning(); //First go to start of the "next" line.
						self.cursor.to_next_line();
					}
				} else {
					//We did not encounter a newline, means we advance the cursor:
					if self.cursor.is_at_canvas_end() {
						//We are at the end of the canvas, we cannot advance the cursor, scroll up to next line.
						scroll_into_next_line = true;
					} else {
						//Will take care of all other cases:
						self.cursor.increment();
					}
				}
				
				//Code to move all content up one line:
				if scroll_into_next_line {
					//First move the cursor to the start of the line:
					self.cursor.to_line_beginning();
					//Then clear basically the whole screen:
					self.clear_above_cursor(letter_instructions);
					self.clear_from_cursor(letter_instructions);
					//Update the line cache (to know where to put the cursor) and redraw the screen:
					self.update_line_cache();
					self.redraw_before_cursor(letter_instructions);
					
					//In case that there was text after the cursor redraw that as well:
					if !self.memory.cursor_at_end() {
						self.redraw_from_cursor(letter_instructions);
					}
				}
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
					//Clear everything above the cursor
					self.clear_above_cursor(letter_instructions);
					//Draw everything before the cursor:
					self.update_line_cache();
					self.redraw_before_cursor(letter_instructions);
					
					//Clear everything after the cursor (chances are high, this is mandatory - except when cursor was in first column):
					self.clear_from_cursor(letter_instructions);
				} else {
					//Not at the end of display, just use the next line!
					
					//Remove from cursor to the rest of the line, in case that the cursor is not at the end of the line:
					if !self.memory.cursor_at_end() {
						self.clear_from_cursor(letter_instructions);
						self.clear_below_cursor(letter_instructions);
					}
					//Move cursor to the start of the next line:
					self.cursor.to_next_line();
					self.cursor.to_line_beginning();
				}
				
				if !self.memory.cursor_at_end() {
					self.redraw_from_cursor(letter_instructions);
				}
			}
			_ => {}
		}
	}
	
	///Moves cursor back, handling edge cases & redrawing. AFTER the memory-cursor has already been moved.
	fn move_cursor_back(&mut self, letter_instructions: &mut Vec<LetterInstruction>, mut must_update_after_cursor: bool) {
		//We either pressed Backspace or Arrow-Left.
		//The cursor moved from second or first column one slot backwards. Both cases have edge cases.
		
		if self.cursor.x <= 1 {
			let mut go_line_up = false;
			if self.cursor.x == 0 {
				//We are either at the beginning of the document, or we can expect to delete a newline.
				assert_false!(self.memory.memory[self.memory.pointer_before_cursor] != NEWLINE, "VIOLATION/CORRUPTION: Attempted to move cursor back, while in first column, but the character was not a newline!");
				
				//When at the beginning and in first column, the cursor was moved from the second line/row to the first (empty) line/row:
				if self.memory.cursor_at_beginning() {
					self.cursor.y = 0;
				} else {
					//Else we have to set the cursor horizontally to match the previous line:
					self.cursor.x = self.find_line_end();
					go_line_up = true;
				}
			} else {
				//We can either remove the first letter of a line, or remove a letter wrapping to the previous line.
				
				//If we removed a NEWLINE, data is corrupted or the cursor was at the wrong position.
				assert_false!(self.memory.memory[self.memory.pointer_before_cursor] == NEWLINE, "VIOLATION/CORRUPTION: Attempted to move cursor back, while in second column, but the character was a newline!");
				
				//We removed a letter. Next check if we are in a wrapped or empty line:
				if self.memory.cursor_at_beginning() || self.memory.memory[self.memory.pointer_before_cursor - 1] == NEWLINE {
					//(Now) empty line (before cursor), we set and allow first cursor column, as this is a line start:
					self.cursor.x = 0;
				} else {
					//A wrapped line, in this case we have to move the cursor to the very last position in the previous line.
					self.cursor.x = CHAR_WIDTH;
					go_line_up = true;
				}
			}
			//Both above cursor position cases can result into moving the cursor into one line above, both handling are identical, thus take care of them here:
			if go_line_up {
				//Differentiate between first and other line:
				if self.cursor.is_first_line() {
					//In the first line, we need to redraw the canvas:
					self.clear_to_cursor(letter_instructions);
					self.redraw_before_cursor_top_line(letter_instructions);
					//Also redraw anything after the cursor:
					must_update_after_cursor = true;
				} else {
					//Else just move the cursor up to the previous line:
					self.cursor.to_previous_line();
				}
			}
		} else {
			assert_false!(self.memory.cursor_at_beginning(), "VIOLATION: Attempted to move cursor one column back, while not in first column, but memory is empty!");
			assert_false!(self.memory.memory[self.memory.pointer_before_cursor] == NEWLINE, "VIOLATION/CORRUPTION: Attempted to move cursor back, while not in first column, but the character was a newline!");
			
			//We are somewhere within a line and can simply decrement the cursor without fear.
			self.cursor.decrement();
		}
		
		//If we deleted a character or moved the cursor into the previous line, we have to redraw everything after the cursor:
		if must_update_after_cursor && !self.memory.cursor_at_end() {
			self.clear_from_cursor(letter_instructions);
			self.clear_below_cursor(letter_instructions);
			self.redraw_from_cursor(letter_instructions);
		}
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
	
	fn clear_to_cursor(&self, letter_instructions: &mut Vec<LetterInstruction>) {
		for x in 0..=self.cursor.x {
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
		if mem_index == 0 {
			//This should never happen, handle it anyway:
			self.line_cache[0] = 0;
			return;
		}
		let mut line_index = 0;
		let mut counter = 0;
		let mut visible_lines = 0;
		
		// println!("Building line cache:");
		loop {
			let value = self.memory.memory[mem_index];
			// println!(":: {}", value);
			if value != NEWLINE {
				counter += 1;
				if counter > CHAR_WIDTH {
					// println!("- WRAP.");
					counter = 1;
					visible_lines += 1;
				}
			}
			
			if value == NEWLINE || mem_index == 0 {
				self.line_cache[line_index] = counter;
				// println!("- {}: {}", line_index, counter);
				counter = 0;
				line_index += 1;
				visible_lines += 1;
				if visible_lines >= CHAR_HEIGHT {
					break;
				}
			}
			
			if mem_index == 0 {
				break;
			}
			mem_index -= 1;
		}
	}
	
	///Returns the cursor position for the line before the cursor.
	fn find_line_end(&self) -> usize {
		let mem = &self.memory;
		let mut counter = 0;
		let mut index = mem.pointer_before_cursor;
		//TODO: How does this handle a single letter on screen? RIP.
		while index != 0 {
			index -= 1;
			
			let value = mem.memory[index];
			if value == NEWLINE {
				return counter;
			}
			
			counter += 1;
			if counter > CHAR_WIDTH {
				counter = 1;
			}
		}
		counter
	}
	
	//Will redraw only the top line, ignoring all constrains. Assumes a valid state where there is enough to draw without newlines, ignoring correct line offset.
	//Call-Requirement:
	// - Cursor X position must be correct.
	fn redraw_before_cursor_top_line(&mut self, letter_instructions: &mut Vec<LetterInstruction>) {
		// println!("Redrawing top line!");
		self.cursor.backup();
		
		let mut pointer = self.memory.pointer_before_cursor - 1; //Get the stack (value) position
		
		loop {
			if self.cursor.is_at_line_start() {
				break;
			}
			self.cursor.decrement_slot();
			self.write_letter_by_id(letter_instructions, self.memory.memory[pointer]);
			pointer -= 1;
		}
		
		self.cursor.restore();
	}
	
	//Call-Requirement:
	// - Pointer must not be at beginning of memory!
	// - Cursor must not be at canvas start!
	fn redraw_before_cursor(&mut self, letter_instructions: &mut Vec<LetterInstruction>) {
		// println!("Redrawing before cursor!");
		self.cursor.backup();
		
		let mut line_index = 0;
		
		let new_cursor_position = self.line_cache[line_index];
		//The new cursor position needs to be subtracted by 1, to have the first letter drawing position. To prevent overflow, handle 0 first.
		self.cursor.x = (if new_cursor_position == 0 { CHAR_WIDTH } else { new_cursor_position }) - 1;
		
		let mut pointer = self.memory.pointer_before_cursor - 1; //Get the stack (value) position
		
		let mut just_had_line_wrap = false;
		loop {
			let value = self.memory.memory[pointer];
			if value == NEWLINE {
				//Encountered newline.
				// println!("> NL");
				line_index += 1;
				if line_index == CHAR_HEIGHT {
					// println!(">> Last line: {}", line_index);
					break; //Drawn last line already.
				}
				//Move cursor:
				if !just_had_line_wrap {
					if self.cursor.is_first_line() {
						// println!(">> Cursor first line.");
						break; //Already at top line.
					}
					self.cursor.to_previous_line();
				}
				just_had_line_wrap = false;
				self.cursor.x = self.line_cache[line_index] - 1;
			} else {
				// println!("> L: {}", value);
				// self.write_space(letter_instructions);
				self.write_letter_by_id(letter_instructions, value);
				//Move cursor:
				if self.cursor.is_at_canvas_start() {
					// println!(">> Cursor at start.");
					break; //Drawn everything that fits
				}
				
				just_had_line_wrap = self.cursor.x < 1;
				self.cursor.decrement_slot();
			}
			
			if pointer == 0 {
				break; //Done processing memory.
			}
			pointer -= 1;
		}
		
		self.cursor.restore();
		self.cursor.x = new_cursor_position;
	}
	
	//Call-Requirement:
	// - Memory-Cursor must not be at the end of memory.
	fn redraw_from_cursor(&mut self, letter_instructions: &mut Vec<LetterInstruction>) {
		self.cursor.backup();
		
		//By calling requirement, it is guaranteed that the after-caret stack is not empty.
		let mut pointer = self.memory.pointer_after_cursor;
		pointer += 1; //Jump to the first value instead of the free slot.
		
		loop {
			//Get value from after-caret stack:
			let value = self.memory.memory[pointer];
			if value == NEWLINE {
				if self.cursor.is_last_line() {
					break;
				}
				self.cursor.to_next_line();
				self.cursor.to_line_beginning();
			} else {
				//Draw that symbol, but first clear the area:
				self.write_letter_by_id(letter_instructions, value);
				
				if self.cursor.is_last_line() && self.cursor.is_at_line_end() {
					break; //Reached the end of display, no space to draw more.
				}
				//Advance to the next position on the display:
				self.cursor.increment();
			}
			
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
		let (x, y) = self.cursor.get_draw_letter_position();
		letter_instructions.push(LetterInstruction {
			pos_x: x,
			pos_y: y,
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
