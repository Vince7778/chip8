
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use egui::{Context, Rect, Pos2, Rounding, Color32, CentralPanel};
use crate::chip8::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::input::InputDriver;

pub struct ChipGUI {
    frame_buffer: Vec<u8>,
    scale: f32,
    frame_rx: Receiver<Vec<u8>>,
    input_mutex: Arc<Mutex<u16>>
}

impl ChipGUI {
    pub fn new(scale: f32, frame_rx: Receiver<Vec<u8>>, input_mutex: Arc<Mutex<u16>>) -> Self {
        ChipGUI {  
            scale,
            frame_buffer: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT / 8],
            frame_rx,
            input_mutex: input_mutex.clone()
        }
    }
}

impl eframe::App for ChipGUI {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        while let Ok(recv_res) = self.frame_rx.recv_timeout(Duration::from_millis(1)) {
            self.frame_buffer = recv_res;
        }

        let mut input_lock = self.input_mutex.lock().unwrap();
        *input_lock = InputDriver::convert_keys(&ctx.input().keys_down);

        CentralPanel::default().show(ctx, |ui| {
            let pt = ui.painter();
            for y in 0..SCREEN_HEIGHT {
                for x in 0..(SCREEN_WIDTH/8) {
                    let byte = self.frame_buffer.get(y * SCREEN_WIDTH/8 + x).unwrap();
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
