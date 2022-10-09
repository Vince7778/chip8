
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use egui::{Context, Rect, Pos2, Rounding, Color32, Window, Vec2, Sense};
use crate::chip8::{SCREEN_WIDTH, SCREEN_HEIGHT, INSTRUCTION_SIZE, MEMORY_SIZE, Chip8, REGISTER_COUNT};
use crate::debugger::{DebuggerState, DebugInstructions};
use crate::input::InputDriver;
use crate::translator;

const INSTRUCTION_VIEW_RANGE: i32 = 3;

pub struct ChipGUI {
    scale: f32,
    input_mutex: Arc<Mutex<u16>>,
    chip8: Arc<Mutex<Chip8>>,
    debugger_mutex: Arc<Mutex<DebuggerState>>,
    debugger: DebuggerState,
    debug_sender: Sender<DebugInstructions>
}

impl ChipGUI {
    pub fn new(_cc: &eframe::CreationContext<'_>, scale: f32, input_mutex: Arc<Mutex<u16>>, chip8: Arc<Mutex<Chip8>>, debugger_mutex: Arc<Mutex<DebuggerState>>, debug_sender: Sender<DebugInstructions>) -> Self {
        let mutex_clone = {
            let ul = debugger_mutex.lock().unwrap();
            ul.clone()
        };
        ChipGUI {  
            scale,
            input_mutex,
            chip8,
            debugger_mutex,
            debugger: mutex_clone,
            debug_sender
        }
    }
}

impl eframe::App for ChipGUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        {
            let all_input = ctx.input();
            {
                let mut input_lock = self.input_mutex.lock().unwrap();
                *input_lock = InputDriver::convert_keys(&all_input.keys_down);
            }

            // check for scrolling
            if self.debugger.paused {
                if all_input.scroll_delta.y > 0f32 {
                    self.debugger.register_scroll -= 1;
                } else if all_input.scroll_delta.y < 0f32 {
                    self.debugger.register_scroll += 1;
                }
            }
        }

        {
            let mut run_speed_lock = self.debugger_mutex.lock().unwrap();
            *run_speed_lock = self.debugger.clone();
        }

        {
            let mut chip8 = self.chip8.lock().unwrap();
            chip8.quirks_mode = self.debugger.quirks;
        }

        Window::new("instructions")
            .show(ctx, |ui| {
                let chip8 = self.chip8.lock().unwrap();
                self.debugger.register_scroll = self.debugger.register_scroll.clamp(-(chip8.pc as i32) + INSTRUCTION_VIEW_RANGE, chip8.pc as i32 - INSTRUCTION_VIEW_RANGE);
                let range_min = -INSTRUCTION_VIEW_RANGE+self.debugger.register_scroll;//.clamp(0, MEMORY_SIZE as i32-INSTRUCTION_SIZE as i32-2*INSTRUCTION_VIEW_RANGE);
                let range_max = INSTRUCTION_VIEW_RANGE+self.debugger.register_scroll;//.clamp(2*INSTRUCTION_VIEW_RANGE, MEMORY_SIZE as i32-INSTRUCTION_SIZE as i32);
                for i in range_min..=range_max {
                    let new_pc = chip8.pc as i32 + i * INSTRUCTION_SIZE as i32;
                    if new_pc < 0 || new_pc + INSTRUCTION_SIZE as i32 >= MEMORY_SIZE as i32 {
                        continue;
                    }

                    let cur_inst = (chip8.memory[new_pc as usize], chip8.memory[new_pc as usize+1]);
                    let translated = translator::translate(cur_inst.0, cur_inst.1);
                    if i == 0 {
                        ui.code(format!("{:<30}", format!("> {:03x} {}", new_pc, translated)));
                    } else {
                        ui.code(format!("{:<30}", format!("  {:03x} {}", new_pc, translated)));
                    }
                }
            });

        Window::new("controls")
            .show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut self.debugger.run_speed, 0.001..=500.0).logarithmic(true).text("Simulation speed"));
                ui.checkbox(&mut self.debugger.paused, "Paused");
                if self.debugger.paused {
                    if ui.button("Step").clicked() {
                        self.debug_sender.send(DebugInstructions::Step).unwrap();
                        self.debugger.register_scroll = 0;
                    }
                    if ui.button("Frame").clicked() {
                        self.debug_sender.send(DebugInstructions::Frame).unwrap();
                    }
                    if ui.button("Reset").clicked() {
                        self.debug_sender.send(DebugInstructions::Reset).unwrap();
                        self.debugger.register_scroll = 0;
                    }
                    if ui.button("Load game from file").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            let str = path.display().to_string();
                            self.debug_sender.send(DebugInstructions::Reload(str)).unwrap();
                        }
                    }
                } else {
                    self.debugger.register_scroll = 0;
                }
                ui.checkbox(&mut self.debugger.quirks.ldi, "Enable loading index quirk");
                ui.checkbox(&mut self.debugger.quirks.shift, "Enable shift behavior quirk");
            });

        Window::new("registers")
            .show(ctx, |ui| {
                let chip8 = self.chip8.lock().unwrap();
                for i in 0..REGISTER_COUNT {
                    let reg_val = chip8.registers[i];
                    ui.code(format!("V{:x}: {:>3} 0x{:02x}", i, reg_val, reg_val));
                }
                ui.code(format!("I:  0x{:04x}", chip8.ir));
            });

        let game_window_size = Vec2 { x: SCREEN_WIDTH as f32 * self.scale, y: SCREEN_HEIGHT as f32 * self.scale };

        Window::new("game_window")
            .fixed_size(game_window_size)
            .show(ctx, |ui| {
                let chip8 = self.chip8.lock().unwrap();
                let frame_buffer = chip8.frame_buffer;
                let (resp, pt) = ui.allocate_painter(game_window_size, Sense::hover());
                let off = resp.rect.left_top();
                for y in 0..SCREEN_HEIGHT {
                    for x in 0..(SCREEN_WIDTH/8) {
                        let byte = frame_buffer[y * SCREEN_WIDTH/8 + x];
                        for xi in 0..8 {
                            let bit = (byte & (1 << (7-xi))) > 0;
                            let rect = Rect { 
                                min: Pos2 { x: off.x+(x*8+xi) as f32 * self.scale, y: off.y+y as f32 * self.scale },
                                max: Pos2 { x: off.x+(x*8+xi+1) as f32 * self.scale, y: off.y+(y+1) as f32 * self.scale },
                            };
                            pt.rect_filled(rect, Rounding::none(), if bit { Color32::WHITE } else { Color32::BLACK });
                        }
                    }
                }
            });

        ctx.request_repaint();
    }
}
