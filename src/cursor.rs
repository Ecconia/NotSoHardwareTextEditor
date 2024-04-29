use crate::config::{CHAR_HEIGHT_UPPER_BOUND, CHAR_WIDTH_UPPER_BOUND, CHAR_WIDTH};

#[derive(Default)]
pub struct CursorPointer {
	x_backup: usize,
	y_backup: usize,
	pub x: usize,
	pub y: usize,
}

impl CursorPointer {
	pub fn increment(&mut self) {
		self.x += 1;
		if self.x > CHAR_WIDTH {
			self.x = 1; //Skip 0, as we just drew a letter.
			self.y += 1;
			if self.y > CHAR_HEIGHT_UPPER_BOUND {
				self.y = 0;
			}
		}
	}
	
	pub fn decrement(&mut self) {
		if self.x <= 1 {
			self.x = CHAR_WIDTH;
			if self.y == 0 {
				self.y = CHAR_HEIGHT_UPPER_BOUND;
			} else {
				self.y -= 1;
			}
		} else {
			self.x -= 1;
		}
	}
	
	/// Go to the previous drawing slot, not the previous cursor position.
	/// # Remarks
	/// Has to be handled differently, due to the WIDTH having an additional cursor position.
	pub fn decrement_slot(&mut self) {
		if self.x < 1 {
			self.x = CHAR_WIDTH_UPPER_BOUND;
			if self.y == 0 {
				self.y = CHAR_HEIGHT_UPPER_BOUND;
			} else {
				self.y -= 1;
			}
		} else {
			self.x -= 1;
		}
	}
	
	pub fn backup(&mut self) {
		self.x_backup = self.x;
		self.y_backup = self.y;
	}
	
	pub fn restore(&mut self) {
		self.x = self.x_backup;
		self.y = self.y_backup;
	}
	
	pub fn is_at_canvas_start(&self) -> bool {
		self.x == 0 && self.y == 0
	}
	
	pub fn is_at_canvas_end(&self) -> bool {
		self.x == CHAR_WIDTH && self.y == CHAR_HEIGHT_UPPER_BOUND
	}
	
	pub fn is_at_line_end(&self) -> bool {
		self.x == CHAR_WIDTH_UPPER_BOUND
	}
	
	pub fn is_at_line_start(&self) -> bool {
		self.x == 0
	}
	
	pub fn is_last_line(&self) -> bool {
		self.y == CHAR_HEIGHT_UPPER_BOUND
	}
	
	pub fn is_first_line(&self) -> bool {
		self.y == 0
	}
	
	pub fn to_line_beginning(&mut self) {
		self.x = 0;
	}
	
	pub fn to_next_line(&mut self) {
		self.y += 1;
	}
	
	pub fn to_previous_line(&mut self) {
		self.y -= 1;
	}
	
	//Position getters. The visible cursor is different from the next placement position, when one line is full.
	
	pub fn get_draw_cursor_position(&self) -> (usize, usize) {
		(self.x, self.y)
	}
	
	pub fn get_draw_letter_position(&self) -> (usize, usize) {
		if self.x == CHAR_WIDTH {
			//Assume that there always is a next line. Because else special handling should be performed and this function never called directly.
			if self.y == CHAR_HEIGHT_UPPER_BOUND {
				println!("VIOLATION: Before drawing a letter, make sure that there is a next line accessible when the cursor is at the end of a line!");
				return (0, self.y);
			}
			return (0, self.y + 1);
		}
		(self.x, self.y)
	}
}
