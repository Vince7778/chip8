use std::{sync::{Mutex, Arc}, collections::HashSet};

use egui::Key;

pub struct InputDriver {
    pub keys: Arc<Mutex<u16>>,
}

impl InputDriver {
    pub fn new() -> Self {
        InputDriver {
            keys: Arc::new(Mutex::new(0u16)),
        }
    }

    pub fn convert_keys(hs: &HashSet<Key>) -> u16 {
        let mut out = 0u16;
        for key in hs.iter() {
            out += match key {
                Key::Num1 => 1u16 << 0x1,
                Key::Num2 => 1u16 << 0x2,
                Key::Num3 => 1u16 << 0x3,
                Key::Num4 => 1u16 << 0xC,
                Key::Q    => 1u16 << 0x4,
                Key::W    => 1u16 << 0x5,
                Key::E    => 1u16 << 0x6,
                Key::R    => 1u16 << 0xD,
                Key::A    => 1u16 << 0x7,
                Key::S    => 1u16 << 0x8,
                Key::D    => 1u16 << 0x9,
                Key::F    => 1u16 << 0xE,
                Key::Z    => 1u16 << 0xA,
                Key::X    => 1u16 << 0x0,
                Key::C    => 1u16 << 0xB,
                Key::V    => 1u16 << 0xF,
                _         => 0
            };
        }
        out
    }
}
