use std::{env, time::{Duration, Instant}, io};
use std::sync::{Arc, Mutex};

use chip8::Chip8;
use gui::ChipGUI;
use input::InputDriver;
use rand::RngCore;

mod chip8;
mod loader;
mod gui;
mod input;
mod hexes;
mod translator;
mod beep;

const _SLEEP_TIME: Duration = Duration::from_millis(2);
const FRAME_DURATION: f32 = 1.0/60.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file = match args.get(1) {
        Some(str) => loader::get_file_bytes(str),
        None => Err(io::Error::new(io::ErrorKind::InvalidInput, "File for rom not given")),
    }?;

    let input_driver = InputDriver::new();
    let driver_keys_clone = input_driver.keys.clone();
    let driver_keys_clone_2 = input_driver.keys.clone();

    let rand_seed = rand::thread_rng().next_u64();
    let chip_rand = rand_pcg::Pcg32::new(rand_seed, 0xa02bdbf7bb3c0a7);

    let chip8arc = Arc::new(Mutex::new(Chip8::new(chip_rand)));
    let chip8clone = chip8arc.clone();
    let chip8_gui_clone = chip8arc.clone();
    {
        let mut w = chip8arc.lock().unwrap();
        w.load(&file)?;
    }


    std::thread::spawn(move || {
        let last_frame = Instant::now();
        let mut last_checked: i64 = 0;
        let mut last_key_input: u16 = 0;
        let beep = beep::Beep::new().unwrap();

        loop {
            let clock_start = Instant::now();

            {
                let mut chip8 = chip8clone.lock().unwrap();
                let time_mult = (last_frame.elapsed().as_secs_f32() / FRAME_DURATION).floor() as i64;
                if time_mult != last_checked {
                    last_checked = time_mult;
                    chip8.frame();
                    if chip8.sound_playing {
                        beep.play().unwrap();
                    } else {
                        beep.pause().unwrap();
                    }
                }

                {
                    let key_input = driver_keys_clone.lock().unwrap();
                    let key_press: u16 = !last_key_input & *key_input;
                    if key_press > 0 {
                        chip8.keypad_press(key_press);
                    }
                    last_key_input = *key_input;
                }

                if let Err(e) = chip8.tick(last_key_input) {
                    println!("{}", e);
                    return;
                }
            }
            

            let clock_total = clock_start.elapsed();
            spin_sleep::sleep(_SLEEP_TIME - clock_total);
        }
    });

    eframe::run_native("Chip8", eframe::NativeOptions::default(), Box::new(|cc| Box::new(ChipGUI::new(cc, 8.0, driver_keys_clone_2, chip8_gui_clone))));

    Ok(())
}
