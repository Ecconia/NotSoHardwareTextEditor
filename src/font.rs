use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

//File name to parse:
const FILE_NAME: &str = "FontBytes.txt";
//Symbol keys of the entries in that file:
const KEYS: &str = concat!(
	"ABCDEFGHIJKLMNOPQRSTUVWXYZ",
	"abcdefghijklmnopqrstuvwxyz",
	"?!.,:'\"",
	"0123456789",
	"+-*/=()→←↓↑",
);

#[derive(Copy, Clone)]
pub struct Instruction {
	pub clear: bool,
	pub x_start: u32,
	pub x_fill: bool,
	pub x_end: u32,
	pub y_start: u32,
	pub y_fill: bool,
	pub y_end: u32,
}

pub type SymbolDescription = Vec<Instruction>;

pub struct Font {
	pub space: SymbolDescription,
	pub symbols_by_key: HashMap<char, SymbolDescription>,
	pub symbols_by_id: HashMap<u8, SymbolDescription>,
	pub keys_by_id: HashMap<u8, char>,
	pub ids_by_key: HashMap<char, u8>,
	pub highest_id: u8,
}

pub fn load_font() -> Font {
	let symbol_instructions = load_instructions();
	if symbol_instructions.len() != KEYS.chars().count() {
		panic!("Loaded wrong amount of symbols from file. Expected {} got {}.", KEYS.len(), symbol_instructions.len());
	}
	
	let space = vec!(Instruction {
		clear: true,
		x_start: 0,
		x_fill: true,
		x_end: 4,
		y_start: 0,
		y_fill: true,
		y_end: 10,
	});
	
	let mut symbols_by_key = HashMap::new();
	let mut symbols_by_id = HashMap::new();
	let mut ids_by_key = HashMap::new();
	let mut keys_by_id = HashMap::new();
	
	symbols_by_key.insert(' ', space.clone());
	symbols_by_id.insert(0, space.clone());
	keys_by_id.insert(0, ' ');
	ids_by_key.insert(' ', 0);
	
	let mut counter = 0;
	for (key, symbol) in KEYS.chars().zip(symbol_instructions.into_iter()) {
		counter += 1;
		symbols_by_key.insert(key, symbol.clone());
		symbols_by_id.insert(counter, symbol);
		keys_by_id.insert(counter, key);
		ids_by_key.insert(key, counter);
	}
	
	return Font {
		space,
		symbols_by_key,
		symbols_by_id,
		ids_by_key,
		keys_by_id,
		highest_id: counter,
	};
}

fn load_instructions() -> Vec<SymbolDescription> {
	let text = read_to_string(Path::new(FILE_NAME)).expect("Failed to read file.");
	
	let mut all_symbols = Vec::new();
	let mut current_symbol = None;
	for line in text.lines() {
		if line.len() != 18 {
			panic!("Font file has invalid line: '{}'", line);
		}
		let line = &line.chars().rev().collect::<String>()[..];
		
		let chars = line.chars().collect::<Vec<char>>();
		let mut is_empty = true;
		for c in chars.iter() {
			if *c != '0' && *c != '1' {
				panic!("Font file line contains invalid character: '{}'", line);
			}
			if *c == '1' {
				is_empty = false;
			}
		}
		if is_empty {
			continue;
		}
		
		//Split/Parse line:
		
		let is_enable = chars[0] == '1';
		let x_start = u32::from_str_radix(&line[1..4], 2).expect("Wat?");
		let x_end = u32::from_str_radix(&line[4..7], 2).expect("Wat?");
		let x_fill = chars[7] == '1';
		let mut y_start = i32::from_str_radix(&line[8..12], 2).expect("Wat?");
		if y_start >= 0b1000 {
			y_start |= 0xFFFFFFF0u32 as i32; //Sign extension
		}
		let mut y_end = i32::from_str_radix(&line[12..16], 2).expect("Wat?");
		if y_end >= 0b1000 {
			y_end |= 0xFFFFFFF0u32 as i32; //Sign extension
		}
		let y_fill = chars[16] == '1';
		let is_start = chars[17] == '1';
		
		//Check bounds:
		if x_start > 4 {
			panic!("X value is out of bounds: {}", x_start);
		}
		if x_end > 4 {
			panic!("X value is out of bounds: {}", x_end);
		}
		if y_start > 7 || y_start < -3 {
			panic!("Y value is out of bounds: {}", y_start);
		}
		if y_end > 7 || y_end < -3 {
			panic!("Y value is out of bounds: {}", y_end);
		}
		if !is_enable {
			panic!("Inverted text is not supported. Encountered enabled==false");
		}
		if x_fill && x_start == x_end {
			panic!("For X_Fill, the X start and end values must differ.");
		}
		if y_fill && y_start == y_end {
			panic!("For Y_Fill, the X start and end values must differ.");
		}
		
		//Move offset to be positive:
		let y_start = (y_start + 3) as u32;
		let y_end = (y_end + 3) as u32;
		
		if is_start {
			// println!();
			if current_symbol.is_some() {
				all_symbols.push(current_symbol.unwrap());
			}
			current_symbol = Some(Vec::new());
		}
		if current_symbol.is_none() {
			panic!("Got draw instruction without ever getting a start bit.");
		}
		// println!("{} {} {} {} {} {}", x_start, x_end, x_fill, y_start, y_end, y_fill);
		current_symbol.as_mut().unwrap().push(Instruction {
			clear: false,
			x_start,
			x_fill,
			x_end,
			y_start,
			y_fill,
			y_end,
		});
	}
	all_symbols.push(current_symbol.unwrap());
	return all_symbols;
}
