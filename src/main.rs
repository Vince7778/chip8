use std::{env, time::{Duration, Instant}, io, sync::mpsc::{Sender, Receiver, self}};

use chip8::Chip8;
use gui::ChipGUI;
use input::InputDriver;

mod chip8;
mod loader;
mod gui;
mod input;
mod hexes;

const _SLEEP_TIME: Duration = Duration::from_millis(2);
const FRAME_DURATION: f32 = 1.0/60.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file = match args.get(1) {
        Some(str) => loader::get_file_bytes(str),
        None => Err(io::Error::new(io::ErrorKind::InvalidInput, "File for rom not given")),
    }?;

    let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
    let tx_thread = tx.clone();

    let input_driver = InputDriver::new();
    let driver_keys_clone = input_driver.keys.clone();

    std::thread::spawn(move || {

        let mut chip8 = Chip8::new(rand::thread_rng());
        if let Err(e) = chip8.load(&file) {
            println!("{}", e);
            return;
        }

        let last_frame = Instant::now();
        let mut last_checked: i64 = 0;
        let mut last_key_input: u16 = 0;

        loop {
            let clock_start = Instant::now();

            let time_mult = (last_frame.elapsed().as_secs_f32() / FRAME_DURATION).floor() as i64;
            if time_mult != last_checked {
                if chip8.display_changed {
                    if let Err(e) = tx_thread.send(chip8.frame_buffer.to_vec()) {
                        println!("{}", e);
                        return;
                    }
                }

                last_checked = time_mult;
                chip8.frame();
            }

            let key_input = driver_keys_clone.lock().unwrap();
            let key_press: u16 = !last_key_input & *key_input;
            if key_press > 0 {
                chip8.keypad_press(key_press);
            }
            last_key_input = *key_input;

            if let Err(e) = chip8.tick(*key_input) {
                println!("{}", e);
                return;
            }

            let clock_total = clock_start.elapsed();
            spin_sleep::sleep(_SLEEP_TIME - clock_total);
        }
    });

    eframe::run_native("Chip8", eframe::NativeOptions::default(), Box::new(|_| Box::new(ChipGUI::new(8.0, rx, input_driver.keys))));

    Ok(())
}
