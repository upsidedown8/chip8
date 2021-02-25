mod chip8;
use chip8::Chip8;

use std::io::{self, Read, Write};
use std::thread;
use std::time;
use std::fs;

use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn write_to_file(filename: &str, bytes: &[u8]) {
    let mut f = fs::File::create(&filename)
        .expect("Unable to create .ch8 file");
    f.write_all(bytes)
        .expect("Unable to write to file");
}

fn load_file(filename: &str) -> Vec<u8> {
    let mut f = fs::File::open(&filename)
        .expect("Unable to open .ch8 file");
    let metadata = fs::metadata(&filename)
        .expect("Unable to read .ch8 file metadata");
    let mut buffer = vec![0; metadata.len() as usize];

    f.read_exact(&mut buffer)
        .expect("buffer overflow");

    buffer
}
fn load_program(chip: &mut Chip8, filename: &str) {
    let buffer = load_file(filename);
    
    let mut addr = 0x0200;

    for &instruction in &buffer[0..buffer.len()-(buffer.len()%2)] {
        chip.write(addr, instruction);
        addr += 1;
    }
}

fn run_interpreter(filename: &str) {
    // setup input
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut stdin = termion::async_stdin().keys();

    // setup chip8
    let mut chip = Chip8::new();
    load_program(&mut chip, filename);

    // initial display
    write!(&mut stdout, "{}", chip.display_to_string()).unwrap();
    stdout.flush().unwrap();

    // main loop
    loop {
        chip.cycle();

        if chip.display_updated() {
            write!(&mut stdout, "{}", termion::clear::All).unwrap();
            write!(&mut stdout, "{}{}", termion::cursor::Goto(1, 1), chip.display_to_string()).unwrap();
            stdout.flush().unwrap();
        }

        // delay so program can be visualised
        thread::sleep(time::Duration::from_millis(2));

        // keyboard input
        (0..16).for_each(|x| chip.write_keypad(x, false));
        let input = stdin.next();
        if let Some(Ok(key)) = input {
            match key {
                // Exit if Ctrl+c is pressed
                termion::event::Key::Ctrl('c') => break,

                // Set keypad
                termion::event::Key::Char('1') => chip.write_keypad(0x0, true),
                termion::event::Key::Char('2') => chip.write_keypad(0x1, true),
                termion::event::Key::Char('3') => chip.write_keypad(0x2, true),
                termion::event::Key::Char('4') => chip.write_keypad(0x3, true),

                termion::event::Key::Char('q') => chip.write_keypad(0x4, true),
                termion::event::Key::Char('w') => chip.write_keypad(0x5, true),
                termion::event::Key::Char('e') => chip.write_keypad(0x6, true),
                termion::event::Key::Char('r') => chip.write_keypad(0x7, true),

                termion::event::Key::Char('a') => chip.write_keypad(0x8, true),
                termion::event::Key::Char('s') => chip.write_keypad(0x9, true),
                termion::event::Key::Char('d') => chip.write_keypad(0xa, true),
                termion::event::Key::Char('f') => chip.write_keypad(0xb, true),

                termion::event::Key::Char('z') => chip.write_keypad(0xc, true),
                termion::event::Key::Char('x') => chip.write_keypad(0xd, true),
                termion::event::Key::Char('c') => chip.write_keypad(0xe, true),
                termion::event::Key::Char('v') => chip.write_keypad(0xf, true),

                _ => {}
            };
        }
    }
}

fn main() {
    /*
        let program = vec![
            0xF0, 0x0A, // Wait for keypad input and store in V0
            0x00, 0xE0, // Clear display
            0xF0, 0x29, // sets I to the font character for V0
            0xD0, 0x04, // Draw four lines of the sprite stored in V0
            0xF0, 0x29, // sets I to the font character for V0
            0x12, 0x00, // Jump to location 200 in memory
        ];
    */

    run_interpreter("examples/snake.ch8");
}
