use std::{env, time::Duration, io};

use chip8::Chip8;

mod chip8;
mod loader;

const SLEEP_TIME: Duration = Duration::from_millis(2);

fn print_to_console(arr: &[u8], width: usize) {
    print!("{}[2J", 27 as char); // cls
    for (i, x) in arr.iter().enumerate() {
        let val_mapped = format!("{:08b}", x).replace("0", ".").replace("1", "@");
        print!("{}", val_mapped);
        if ((i+1) * 8) % width == 0 {
            print!("\n");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file = match args.get(1) {
        Some(str) => loader::get_file_bytes(str),
        None => Err(io::Error::new(io::ErrorKind::InvalidInput, "File for rom not given")),
    }?;

    let mut chip8 = Chip8::new(rand::thread_rng());
    chip8.load(&file)?;
    
    loop {
        chip8.tick(0).or_else(|e| { println!("{}", e); Err(e) })?;
        if chip8.display_changed {
            print_to_console(&chip8.frame_buffer, 64);
        }
        std::thread::sleep(SLEEP_TIME);
    }
}
