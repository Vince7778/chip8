
use crate::chip8::get_nnn;

pub fn translate(b1: u8, b2: u8) -> String {
    let bits = (b1 >> 4, b1 & 0x0F, b2 >> 4, b2 & 0x0F);
    match bits.0 {
        0x0 => match get_nnn(b1, b2) {
            0x0E0 => String::from("CLS"),
            0x0EE => String::from("RET"),
            _ => format!("XXXX {:02x}{:02x}", b1, b2)
        }
        0x1 => format!("JP   0x{:03x}", get_nnn(b1, b2)),
        0x2 => format!("CALL 0x{:03x}", get_nnn(b1, b2)),
        0x3 => format!("SE   V{:x},  0x{:02x}", bits.1, b2),
        0x4 => format!("SNE  V{:x},  0x{:02x}", bits.1, b2),
        0x5 => match bits.3 {
            0x0 => format!("SE   V{:x},  V{:x}", bits.1, bits.2),
            _ => format!("XXXX {:02x}{:02x}", b1, b2),
        },
        0x6 => format!("LD   V{:x},  0x{:02x}", bits.1, b2),
        0x7 => format!("ADD  V{:x},  0x{:02x}", bits.1, b2),
        0x8 => match bits.3 {
            0x0 => format!("LD   V{:x},  V{:x}", bits.1, bits.2),
            0x1 => format!("OR   V{:x},  V{:x}", bits.1, bits.2),
            0x2 => format!("AND  V{:x},  V{:x}", bits.1, bits.2),
            0x3 => format!("XOR  V{:x},  V{:x}", bits.1, bits.2),
            0x4 => format!("ADD  V{:x},  V{:x}", bits.1, bits.2),
            0x5 => format!("SUB  V{:x},  V{:x}", bits.1, bits.2),
            0x6 => format!("SHR  V{:x},  <V{:x}>", bits.1, bits.2),
            0x7 => format!("SUBN V{:x},  V{:x}", bits.1, bits.2),
            0xE => format!("SHL  V{:x},  <V{:x}>", bits.1, bits.2),
            _ => format!("XXXX {:02x}{:02x}", b1, b2)
        },
        0x9 => match bits.3 {
            0x0 => format!("SNE  V{:x},  V{:x}", bits.1, bits.2),
            _ => format!("XXXX {:02x}{:02x}", b1, b2)
        },
        0xA => format!("LD   I,   0x{:03x}", get_nnn(b1, b2)),
        0xB => format!("JP   V0,  0x{:03x}", get_nnn(b1, b2)),
        0xC => format!("RND  V{:x},  0x{:02x}", bits.1, b2),
        0xD => format!("DRW  V{:x},  V{:x},  0x{:x}", bits.1, bits.2, bits.3),
        0xE => match b2 {
            0x9E => format!("SKP  V{:x}", bits.1),
            0xA1 => format!("SKNP V{:x}", bits.1),
            _ => format!("XXXX {:02x}{:02x}", b1, b2),
        },
        0xF => match b2 {
            0x07 => format!("LD   V{:x},  DT", bits.1),
            0x0A => format!("LD   V{:x},  K", bits.1),
            0x15 => format!("LD   DT,  V{:x}", bits.1),
            0x18 => format!("LD   ST,  V{:x}", bits.1),
            0x1E => format!("ADD  I,   V{:x}", bits.1),
            0x29 => format!("LD   F,   V{:x}", bits.1),
            0x33 => format!("LD   B,   V{:x}", bits.1),
            0x55 => format!("LD   [I], V{:x}", bits.1),
            0x65 => format!("LD   V{:x},  [I]", bits.1),
            _ => format!("XXXX {:02x}{:02x}", b1, b2)
        }
        _ => format!("XXXX {:02x}{:02x}", b1, b2)
    }
}