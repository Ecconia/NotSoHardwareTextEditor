use crate::config::{CHAR_HEIGHT_UPPER_BOUND, CHAR_WIDTH_UPPER_BOUND};

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
		if self.x > CHAR_WIDTH_UPPER_BOUND {
			self.x = 0;
			self.y += 1;
			if self.y > CHAR_HEIGHT_UPPER_BOUND {
				self.y = 0;
			}
		}
	}
	
	pub fn decrement(&mut self) {
		if self.x == 0 {
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
	
	pub fn is_at_start(&self) -> bool {
		self.x == 0 && self.y == 0
	}
	
	pub fn is_at_end(&self) -> bool {
		self.x == CHAR_WIDTH_UPPER_BOUND && self.y == CHAR_HEIGHT_UPPER_BOUND
	}
	
	pub fn is_at_line_end(&self) -> bool {
		self.x == CHAR_WIDTH_UPPER_BOUND
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
}
