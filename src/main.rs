extern crate sdl2;

use std::time::Instant;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use typewriter::config::{HEIGHT, PIXEL_SIDE, WIDTH};
use typewriter::font;
use typewriter::font::Instruction;
use typewriter::typewriter::{LetterInstruction, Typewriter};

const COLOR_BACKGROUND: Color = Color::RGB(10, 15, 10);
const COLOR_FOREGROUND: Color = Color::RGB(50, 255, 50);

pub fn main() -> Result<(), String> {
	let font = font::load_font();
	
	sdl2::hint::set("SDL_HINT_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");
	
	let sdl_context = sdl2::init()?;
	let video_subsystem = sdl_context.video()?;
	
	let window = video_subsystem
		.window(
			"Ecconia Maze Test",
			WIDTH * PIXEL_SIDE,
			HEIGHT * PIXEL_SIDE,
		)
		.position_centered()
		.build()
		.map_err(|e| e.to_string())?;
	
	let mut canvas = window
		.into_canvas()
		.target_texture()
		.present_vsync()
		.build()
		.map_err(|e| e.to_string())?;
	
	canvas.set_scale(PIXEL_SIDE as f32, PIXEL_SIDE as f32).expect("Scale failed.");
	
	canvas.set_draw_color(COLOR_BACKGROUND);
	canvas.clear();
	canvas.present();
	
	let texture_creator = canvas.texture_creator();
	let mut buffer_texture = texture_creator.create_texture_target(None, WIDTH, HEIGHT).map_err(|e| e.to_string())?;
	canvas.with_texture_canvas(&mut buffer_texture, |buffer_canvas| {
		buffer_canvas.set_draw_color(COLOR_BACKGROUND);
		buffer_canvas.clear();
	}).map_err(|e| e.to_string())?;
	
	let mut typewriter = Typewriter::default();
	
	let mut letter_instructions : Vec<LetterInstruction> = Vec::new();
	let cursor_time = Instant::now();
	let mut event_pump = sdl_context.event_pump()?;
	'running: loop {
		for event in event_pump.poll_iter() {
			match event {
				//Termination condition of the program:
				Event::Quit { .. }
				| Event::KeyDown {
					keycode: Some(Keycode::Escape),
					..
				} => break 'running,
				//Custom keydown events to be considered by the typewriter:
				Event::KeyDown {
					scancode: Some(scancode),
					repeat: false,
					keymod,
					..
				} => {
					typewriter.handle_input(&scancode, &keymod, &mut letter_instructions, &font);
					typewriter.dump_debug();
				}
				_ => {}
			}
		}
		
		//Clear the canvas buffer:
		canvas.set_draw_color(COLOR_BACKGROUND);
		canvas.clear();
		
		if !letter_instructions.is_empty() {
			//Draw the new letters:
			canvas.with_texture_canvas(&mut buffer_texture, |texture_canvas| {
				for instruction in &letter_instructions {
					let symbol = font.symbols_by_id.get(&instruction.id);
					if symbol.is_none() {
						panic!("Tried to draw letter that does not exist with ID: {}", instruction.id);
					}
					draw_letter(texture_canvas, symbol.unwrap(), instruction.pos_x, instruction.pos_y);
				}
			}).expect("Failed to edit buffer texture.");
			//Clear the instructions
			letter_instructions.clear();
		}
		//Always update the canvas with the buffer:
		canvas.copy(&buffer_texture, None, Rect::new(0,0,WIDTH,HEIGHT)).map_err(|e| e.to_string())?;
		
		//Draw cursor:
		if cursor_time.elapsed().as_millis() % 1000 >= 500 {
			draw_cursor(&mut canvas, typewriter.cursor.x, typewriter.cursor.y);
		}
		
		//Apply:
		canvas.present();
	}
	
	Ok(())
}

fn draw_cursor(canvas: &mut WindowCanvas, x: usize, y: usize) {
	canvas.set_draw_color(COLOR_FOREGROUND);
	let x_offset = (x * 6) as i32;
	let y_offset = (y * 12 + 11) as i32;
	canvas.draw_line(
		Point::new(x_offset, y_offset - 3),
		Point::new(x_offset, y_offset - 10),
	).expect("Failed to draw line.");
}

fn draw_letter(canvas: &mut WindowCanvas, symbol: &Vec<Instruction>, x: usize, y: usize) {
	let x_offset = (x * 6 + 1) as i32;
	let y_offset = (y * 12 + 11) as i32;
	draw_letter_at(canvas, symbol, x_offset, y_offset);
}

fn draw_letter_at(canvas: &mut WindowCanvas, symbol: &Vec<Instruction>, x_offset: i32, y_offset: i32) {
	for operation in symbol.iter() {
		let x_start = operation.x_start as i32;
		let x_end = operation.x_end as i32;
		let y_start = operation.y_start as i32;
		let y_end = operation.y_end as i32;
		
		canvas.set_draw_color(if operation.clear { COLOR_BACKGROUND } else { COLOR_FOREGROUND });
		
		if operation.x_fill {
			if operation.y_fill {
				//Fill in both directions - fill a rect:
				canvas.fill_rect(
					Rect::new(
						x_offset + x_start,
						y_offset - y_start - 10,
						(x_end - x_start + 1) as u32,
						(y_end - y_start + 1) as u32,
					)
				).expect("Failed to fill rect.");
			} else {
				//Just X-Fill:
				if y_start == y_end {
					//One horizontal line:
					canvas.draw_line(
						Point::new(x_offset + x_start, y_offset - y_start),
						Point::new(x_offset + x_end, y_offset - y_end),
					).expect("Failed to draw line.");
				} else {
					//Two horizontal lines:
					canvas.draw_line(
						Point::new(x_offset + x_start, y_offset - y_start),
						Point::new(x_offset + x_end, y_offset - y_start),
					).expect("Failed to draw line.");
					canvas.draw_line(
						Point::new(x_offset + x_start, y_offset - y_end),
						Point::new(x_offset + x_end, y_offset - y_end),
					).expect("Failed to draw line.");
				}
			}
		} else {
			if operation.y_fill {
				//Just Y-Fill:
				if x_start == x_end {
					//One vertical line:
					canvas.draw_line(
						Point::new(x_offset + x_start, y_offset - y_start),
						Point::new(x_offset + x_end, y_offset - y_end),
					).expect("Failed to draw line.");
				} else {
					//Two vertical lines:
					canvas.draw_line(
						Point::new(x_offset + x_start, y_offset - y_start),
						Point::new(x_offset + x_start, y_offset - y_end),
					).expect("Failed to draw line.");
					canvas.draw_line(
						Point::new(x_offset + x_end, y_offset - y_start),
						Point::new(x_offset + x_end, y_offset - y_end),
					).expect("Failed to draw line.");
				}
			} else {
				//No fill at all, just draw the points:
				if x_start == x_end && y_start == y_end {
					canvas.draw_point(
						Point::new(x_offset + x_start, y_offset - y_start),
					).expect("Failed to draw point.");
				} else if x_start == x_end || y_start == y_end {
					canvas.draw_point(
						Point::new(x_offset + x_start, y_offset - y_start),
					).expect("Failed to draw point.");
					canvas.draw_point(
						Point::new(x_offset + x_end, y_offset - y_end),
					).expect("Failed to draw point.");
				} else {
					canvas.draw_point(
						Point::new(x_offset + x_start, y_offset - y_start),
					).expect("Failed to draw point.");
					canvas.draw_point(
						Point::new(x_offset + x_end, y_offset - y_end),
					).expect("Failed to draw point.");
					canvas.draw_point(
						Point::new(x_offset + x_start, y_offset - y_end),
					).expect("Failed to draw point.");
					canvas.draw_point(
						Point::new(x_offset + x_end, y_offset - y_start),
					).expect("Failed to draw point.");
				}
			}
		}
	}
}
