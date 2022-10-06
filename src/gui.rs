
use std::sync::{Arc, Mutex};

use egui::{Context, Rect, Pos2, Rounding, Color32, CentralPanel};
use crate::chip8::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::input::InputDriver;

pub struct ChipGUI {
    scale: f32,
    frame_mutex: Arc<Mutex<[u8; SCREEN_WIDTH * SCREEN_HEIGHT / 8]>>,
    input_mutex: Arc<Mutex<u16>>
}

impl ChipGUI {
    pub fn new(scale: f32, frame_mutex: Arc<Mutex<[u8; SCREEN_WIDTH * SCREEN_HEIGHT / 8]>>, input_mutex: Arc<Mutex<u16>>) -> Self {
        ChipGUI {  
            scale,
            frame_mutex,
            input_mutex
        }
    }
}

impl eframe::App for ChipGUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut input_lock = self.input_mutex.lock().unwrap();
        *input_lock = InputDriver::convert_keys(&ctx.input().keys_down);

        CentralPanel::default().show(ctx, |ui| {
            let frame_buffer = self.frame_mutex.lock().unwrap();
            let pt = ui.painter();
            for y in 0..SCREEN_HEIGHT {
                for x in 0..(SCREEN_WIDTH/8) {
                    let byte = frame_buffer[y * SCREEN_WIDTH/8 + x];
                    for xi in 0..8 {
                        let bit = (byte & (1 << (7-xi))) > 0;
                        let rect = Rect { 
                            min: Pos2 { x: (x*8+xi) as f32 * self.scale, y: y as f32 * self.scale },
                            max: Pos2 { x: (x*8+xi+1) as f32 * self.scale, y: (y+1) as f32 * self.scale },
                        };
                        pt.rect_filled(rect, Rounding::none(), if bit { Color32::WHITE } else { Color32::BLACK });
                    }
                }
            }
        });

        ctx.request_repaint();
    }
}
