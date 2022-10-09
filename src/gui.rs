
use std::sync::{Arc, Mutex};

use egui::{Context, Rect, Pos2, Rounding, Color32, Window, Vec2, Sense};
use crate::chip8::{SCREEN_WIDTH, SCREEN_HEIGHT, Chip8};
use crate::input::InputDriver;
use crate::translator;

pub struct ChipGUI {
    scale: f32,
    input_mutex: Arc<Mutex<u16>>,
    chip8: Arc<Mutex<Chip8>>
}

impl ChipGUI {
    pub fn new(_cc: &eframe::CreationContext<'_>, scale: f32, input_mutex: Arc<Mutex<u16>>, chip8: Arc<Mutex<Chip8>>) -> Self {
        ChipGUI {  
            scale,
            input_mutex,
            chip8
        }
    }
}

impl eframe::App for ChipGUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        {
            let mut input_lock = self.input_mutex.lock().unwrap();
            *input_lock = InputDriver::convert_keys(&ctx.input().keys_down);
        }

        Window::new("inst_panel").show(ctx, |ui| {
            let chip8 = self.chip8.lock().unwrap();
            let cur_inst = (chip8.memory[chip8.pc as usize], chip8.memory[chip8.pc as usize+1]);
            let translated = translator::translate(cur_inst.0, cur_inst.1);
            ui.code(format!("{}", translated));
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
