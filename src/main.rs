use std::{env, time::{Duration, Instant}, sync};
use std::sync::{Arc, Mutex};

use chip8::Chip8;
use debugger::{DebuggerState, DebugInstructions};
use gui::ChipGUI;
use input::InputDriver;
use rand::{RngCore, thread_rng};

mod chip8;
mod loader;
mod gui;
mod input;
mod hexes;
mod translator;
mod beep;
mod debugger;

const SLEEP_TIME: Duration = Duration::from_millis(2);
const FRAME_DURATION: f32 = 1.0/60.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut file = match args.get(1) {
        Some(str) => loader::get_file_bytes(str)?,
        None => vec![],
    };

    let input_driver = InputDriver::new();
    let driver_keys_clone = input_driver.keys.clone();
    let driver_keys_clone_2 = input_driver.keys.clone();

    let rand_seed = rand::thread_rng().next_u64();
    let chip_rand = rand_pcg::Pcg32::new(rand_seed, 0xa02bdbf7bb3c0a7);

    let chip8arc = Arc::new(Mutex::new(Chip8::new(chip_rand)));
    let chip8clone = chip8arc.clone();
    let chip8_gui_clone = chip8arc.clone();

    let debugger = Arc::new(Mutex::new(DebuggerState::default()));
    let debugger_chip8 = debugger.clone();

    let (debug_send, debug_recv) = sync::mpsc::channel::<DebugInstructions>();

    if !file.is_empty() {
        let mut w = chip8arc.lock().unwrap();
        w.load(&file)?;
    } else {
        let mut debug_writer = debugger.lock().unwrap();
        debug_writer.paused = true;
    }

    std::thread::spawn(move || {
        let last_frame = Instant::now();
        let mut last_checked: i64 = 0;
        let mut last_key_input: u16 = 0;
        let beep = beep::Beep::new().unwrap();

        loop {
            let clock_start = Instant::now();

            let is_paused = {
                let dbg = debugger_chip8.lock().unwrap();
                dbg.paused
            };

            if !is_paused {
                let mut chip8 = chip8clone.lock().unwrap();
                let time_mult = {
                    let spd = debugger_chip8.lock().unwrap().run_speed;
                    (last_frame.elapsed().as_secs_f32() / FRAME_DURATION * spd).floor() as i64
                };

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
                    let key_press: u16 = last_key_input & !*key_input;
                    if key_press > 0 {
                        chip8.keypad_press(key_press);
                    }
                    last_key_input = *key_input;
                }

                if let Err(e) = chip8.tick(last_key_input) {
                    println!("{}", e);
                    return;
                }
            } else {
                match debug_recv.try_recv() {
                    Ok(DebugInstructions::Step) => {
                        let mut chip8 = chip8clone.lock().unwrap();
                        if let Err(e) = chip8.tick(last_key_input) {
                            println!("{}", e);
                            return;
                        }
                    },
                    Ok(DebugInstructions::Frame) => {
                        let mut chip8 = chip8clone.lock().unwrap();
                        chip8.frame();
                        if chip8.sound_playing {
                            beep.play().unwrap();
                        } else {
                            beep.pause().unwrap();
                        }
                    },
                    Ok(DebugInstructions::Reset) => {
                        let mut chip8 = chip8clone.lock().unwrap();
                        *chip8 = Chip8::new(rand_pcg::Pcg32::new(thread_rng().next_u64(), 0xa02bdbf7bb3c0a7));
                        chip8.load(&file).unwrap();
                    },
                    Ok(DebugInstructions::Reload(path)) => {
                        let mut chip8 = chip8clone.lock().unwrap();
                        *chip8 = Chip8::new(rand_pcg::Pcg32::new(thread_rng().next_u64(), 0xa02bdbf7bb3c0a7));
                        file = loader::get_file_bytes(&path).unwrap();
                        chip8.load(&file).unwrap();
                    },
                    Err(sync::mpsc::TryRecvError::Disconnected) => {
                        eprintln!("Error: disconnected");
                        return;
                    },
                    Err(_) => ()
                };
            }

            let clock_total = clock_start.elapsed();
            let sleep_duration = if is_paused {
                SLEEP_TIME.saturating_sub(clock_total)
            } else {
                let spd = debugger_chip8.lock().unwrap().run_speed;
                SLEEP_TIME.div_f32(spd).saturating_sub(clock_total)
            };

            if !sleep_duration.is_zero() {
                spin_sleep::sleep(sleep_duration);
            }
        }
    });

    eframe::run_native("Chip8", eframe::NativeOptions::default(), Box::new(|cc| Box::new(ChipGUI::new(cc, 8.0, driver_keys_clone_2, chip8_gui_clone, debugger, debug_send))));

    Ok(())
}
