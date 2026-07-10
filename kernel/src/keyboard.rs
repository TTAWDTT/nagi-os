use crate::{port, serial, vga};

const KEYBOARD_DATA_PORT: u16 = 0x60;
const INPUT_ROW: usize = 13;
const INPUT_CAPACITY: usize = 62;

static mut INPUT: [u8; INPUT_CAPACITY] = [0; INPUT_CAPACITY];
static mut INPUT_LEN: usize = 0;
static mut SHIFT: bool = false;
static mut SEEN_KEY: bool = false;

pub fn init_screen() {
    redraw();
}

pub fn handle_interrupt() {
    let scancode = unsafe { port::inb(KEYBOARD_DATA_PORT) };

    match scancode {
        0x2A | 0x36 => unsafe {
            SHIFT = true;
        },
        0xAA | 0xB6 => unsafe {
            SHIFT = false;
        },
        code if code & 0x80 != 0 => {}
        code => {
            if let Some(key) = decode_scancode(code) {
                handle_key(key);
            }
        }
    }
}

enum Key {
    Char(u8),
    Backspace,
    Enter,
}

fn handle_key(key: Key) {
    unsafe {
        if !SEEN_KEY {
            serial::write_str("keyboard IRQ1 online\r\n");
            SEEN_KEY = true;
        }

        match key {
            Key::Char(byte) => {
                if INPUT_LEN < INPUT_CAPACITY {
                    INPUT[INPUT_LEN] = byte;
                    INPUT_LEN += 1;
                    serial::write_byte(byte);
                }
            }
            Key::Backspace => {
                if INPUT_LEN > 0 {
                    INPUT_LEN -= 1;
                    INPUT[INPUT_LEN] = 0;
                    serial::write_str("\x08 \x08");
                }
            }
            Key::Enter => {
                serial::write_str("\r\n");
                INPUT_LEN = 0;
                let mut i = 0;
                while i < INPUT_CAPACITY {
                    INPUT[i] = 0;
                    i += 1;
                }
            }
        }
    }

    redraw();
}

fn redraw() {
    let color = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let mut line = [b' '; 80];
    let mut idx = copy_bytes(&mut line, 0, b"keyboard> ");

    unsafe {
        let mut i = 0;
        while i < INPUT_LEN && idx < line.len() {
            line[idx] = INPUT[i];
            idx += 1;
            i += 1;
        }
    }

    vga::write_line(INPUT_ROW, as_str(&line), color);
}

fn decode_scancode(scancode: u8) -> Option<Key> {
    match scancode {
        0x0E => Some(Key::Backspace),
        0x1C => Some(Key::Enter),
        0x39 => Some(Key::Char(b' ')),
        code => decode_ascii(code).map(Key::Char),
    }
}

fn decode_ascii(scancode: u8) -> Option<u8> {
    let shifted = unsafe { SHIFT };
    let normal = match scancode {
        0x02 => b'1',
        0x03 => b'2',
        0x04 => b'3',
        0x05 => b'4',
        0x06 => b'5',
        0x07 => b'6',
        0x08 => b'7',
        0x09 => b'8',
        0x0A => b'9',
        0x0B => b'0',
        0x0C => b'-',
        0x0D => b'=',
        0x10 => b'q',
        0x11 => b'w',
        0x12 => b'e',
        0x13 => b'r',
        0x14 => b't',
        0x15 => b'y',
        0x16 => b'u',
        0x17 => b'i',
        0x18 => b'o',
        0x19 => b'p',
        0x1A => b'[',
        0x1B => b']',
        0x1E => b'a',
        0x1F => b's',
        0x20 => b'd',
        0x21 => b'f',
        0x22 => b'g',
        0x23 => b'h',
        0x24 => b'j',
        0x25 => b'k',
        0x26 => b'l',
        0x27 => b';',
        0x28 => b'\'',
        0x29 => b'`',
        0x2B => b'\\',
        0x2C => b'z',
        0x2D => b'x',
        0x2E => b'c',
        0x2F => b'v',
        0x30 => b'b',
        0x31 => b'n',
        0x32 => b'm',
        0x33 => b',',
        0x34 => b'.',
        0x35 => b'/',
        _ => return None,
    };

    Some(if shifted {
        shifted_ascii(normal)
    } else {
        normal
    })
}

fn shifted_ascii(byte: u8) -> u8 {
    match byte {
        b'a'..=b'z' => byte - 32,
        b'1' => b'!',
        b'2' => b'@',
        b'3' => b'#',
        b'4' => b'$',
        b'5' => b'%',
        b'6' => b'^',
        b'7' => b'&',
        b'8' => b'*',
        b'9' => b'(',
        b'0' => b')',
        b'-' => b'_',
        b'=' => b'+',
        b'[' => b'{',
        b']' => b'}',
        b';' => b':',
        b'\'' => b'"',
        b'`' => b'~',
        b'\\' => b'|',
        b',' => b'<',
        b'.' => b'>',
        b'/' => b'?',
        _ => byte,
    }
}

fn copy_bytes(dst: &mut [u8], mut idx: usize, src: &[u8]) -> usize {
    for byte in src {
        if idx >= dst.len() {
            break;
        }
        dst[idx] = *byte;
        idx += 1;
    }
    idx
}

fn as_str(bytes: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(bytes) }
}
