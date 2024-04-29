//Stuff to change, enjoy:
pub const TARGET_DISPLAY : u32 = 1;
pub const PIXEL_SIDE: u32 = 6; //8
pub const CHAR_WIDTH: usize = 42; //42
pub const CHAR_HEIGHT: usize = 10; //10
pub const MEMORY_SIZE : usize = 0x1000;

//Adjust if needed...
pub const NEWLINE : u8 = -1i8 as u8;

//Calculated:
pub const WIDTH: u32 = (1 + CHAR_WIDTH * 6) as u32;
pub const HEIGHT: u32 = (1 + CHAR_HEIGHT * 12) as u32;
pub const CHAR_WIDTH_UPPER_BOUND: usize = CHAR_WIDTH - 1;
pub const CHAR_HEIGHT_UPPER_BOUND: usize = CHAR_HEIGHT - 1;
