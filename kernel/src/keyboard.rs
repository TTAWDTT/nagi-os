use core::sync::atomic::{AtomicU64, Ordering};

use crate::{port, serial, shell, trace, ui};

const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;
const INPUT_CAPACITY: usize = 62;

static mut INPUT: [u8; INPUT_CAPACITY] = [0; INPUT_CAPACITY];
static mut INPUT_LEN: usize = 0;
static mut INPUT_CURSOR: usize = 0;
static mut LAST_INPUT: [u8; INPUT_CAPACITY] = [0; INPUT_CAPACITY];
static mut LAST_INPUT_LEN: usize = 0;
static mut SHIFT: bool = false;
static mut EXTENDED: bool = false;
static mut SEEN_KEY: bool = false;
static IRQ_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn init_screen() {
    drain_output_buffer();
    redraw();
}

pub fn handle_interrupt() {
    IRQ_COUNT.fetch_add(1, Ordering::Relaxed);
    let scancode = unsafe { port::inb(KEYBOARD_DATA_PORT) };

    if scancode == 0xE0 {
        unsafe {
            EXTENDED = true;
        }
        return;
    }

    match scancode {
        0x2A | 0x36 => unsafe {
            SHIFT = true;
        },
        0xAA | 0xB6 => unsafe {
            SHIFT = false;
        },
        code if code & 0x80 != 0 => unsafe {
            EXTENDED = false;
        },
        code => {
            let extended = unsafe {
                let was_extended = EXTENDED;
                EXTENDED = false;
                was_extended
            };
            if let Some(key) = decode_scancode(code, extended) {
                handle_key(key);
            }
        }
    }
}

pub fn irq_count() -> u64 {
    IRQ_COUNT.load(Ordering::Relaxed)
}

fn drain_output_buffer() {
    let mut attempts = 0;
    while attempts < 16 {
        let status = unsafe { port::inb(KEYBOARD_STATUS_PORT) };
        if status & 0x01 == 0 {
            break;
        }
        let _ = unsafe { port::inb(KEYBOARD_DATA_PORT) };
        attempts += 1;
    }
}

enum Key {
    Char(u8),
    Backspace,
    Delete,
    Enter,
    Recall,
    Complete,
    Clear,
    MoveLeft,
    MoveRight,
    Home,
    End,
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
                    let mut i = INPUT_LEN;
                    while i > INPUT_CURSOR {
                        INPUT[i] = INPUT[i - 1];
                        i -= 1;
                    }
                    INPUT[INPUT_CURSOR] = byte;
                    INPUT_LEN += 1;
                    INPUT_CURSOR += 1;
                    serial::write_byte(byte);
                    trace::record(trace::TraceKind::Keyboard, byte as u64, "key");
                }
            }
            Key::Backspace => {
                if INPUT_CURSOR > 0 {
                    let mut i = INPUT_CURSOR - 1;
                    while i + 1 < INPUT_LEN {
                        INPUT[i] = INPUT[i + 1];
                        i += 1;
                    }
                    INPUT_LEN -= 1;
                    INPUT_CURSOR -= 1;
                    INPUT[INPUT_LEN] = 0;
                    serial::write_str("\x08 \x08");
                    trace::record(trace::TraceKind::Keyboard, 8, "backspace");
                }
            }
            Key::Delete => {
                if INPUT_CURSOR < INPUT_LEN {
                    let mut i = INPUT_CURSOR;
                    while i + 1 < INPUT_LEN {
                        INPUT[i] = INPUT[i + 1];
                        i += 1;
                    }
                    INPUT_LEN -= 1;
                    INPUT[INPUT_LEN] = 0;
                    trace::record(trace::TraceKind::Keyboard, 127, "delete");
                }
            }
            Key::Enter => {
                serial::write_str("\r\n");
                trace::record(trace::TraceKind::Keyboard, INPUT_LEN as u64, "enter");
                remember_input();
                shell::run(as_str(&INPUT[..INPUT_LEN]));
                INPUT_LEN = 0;
                INPUT_CURSOR = 0;
                let mut i = 0;
                while i < INPUT_CAPACITY {
                    INPUT[i] = 0;
                    i += 1;
                }
            }
            Key::Recall => {
                recall_last_input();
                trace::record(trace::TraceKind::Keyboard, LAST_INPUT_LEN as u64, "recall");
            }
            Key::Complete => {
                complete_input();
                trace::record(trace::TraceKind::Keyboard, INPUT_LEN as u64, "complete");
            }
            Key::Clear => {
                clear_input();
                trace::record(trace::TraceKind::Keyboard, 0, "clear-input");
            }
            Key::MoveLeft => {
                if INPUT_CURSOR > 0 {
                    INPUT_CURSOR -= 1;
                    trace::record(trace::TraceKind::Keyboard, INPUT_CURSOR as u64, "left");
                }
            }
            Key::MoveRight => {
                if INPUT_CURSOR < INPUT_LEN {
                    INPUT_CURSOR += 1;
                    trace::record(trace::TraceKind::Keyboard, INPUT_CURSOR as u64, "right");
                } else {
                    complete_input();
                    trace::record(trace::TraceKind::Keyboard, INPUT_LEN as u64, "complete");
                }
            }
            Key::Home => {
                INPUT_CURSOR = 0;
                trace::record(trace::TraceKind::Keyboard, 0, "home");
            }
            Key::End => {
                INPUT_CURSOR = INPUT_LEN;
                trace::record(trace::TraceKind::Keyboard, INPUT_CURSOR as u64, "end");
            }
        }
    }

    redraw();
}

fn redraw() {
    unsafe {
        let input = as_str(&INPUT[..INPUT_LEN]);
        let suggestion = if INPUT_CURSOR == INPUT_LEN {
            shell::complete(input)
        } else {
            None
        };
        ui::draw_prompt(input, suggestion, INPUT_CURSOR, INPUT_LEN >= INPUT_CAPACITY);
    }
}

fn decode_scancode(scancode: u8, extended: bool) -> Option<Key> {
    if extended {
        return match scancode {
            0x48 => Some(Key::Recall),
            0x47 => Some(Key::Home),
            0x4B => Some(Key::MoveLeft),
            0x4D => Some(Key::MoveRight),
            0x4F => Some(Key::End),
            0x53 => Some(Key::Delete),
            _ => None,
        };
    }

    match scancode {
        0x01 => Some(Key::Clear),
        0x0E => Some(Key::Backspace),
        0x0F => Some(Key::Complete),
        0x1C => Some(Key::Enter),
        0x3B => Some(Key::Recall),
        0x39 => Some(Key::Char(b' ')),
        code => decode_ascii(code).map(Key::Char),
    }
}

fn remember_input() {
    unsafe {
        if INPUT_LEN == 0 {
            return;
        }
        LAST_INPUT_LEN = INPUT_LEN;
        let mut i = 0;
        while i < INPUT_CAPACITY {
            LAST_INPUT[i] = if i < INPUT_LEN { INPUT[i] } else { 0 };
            i += 1;
        }
    }
}

fn recall_last_input() {
    unsafe {
        INPUT_LEN = LAST_INPUT_LEN;
        INPUT_CURSOR = INPUT_LEN;
        let mut i = 0;
        while i < INPUT_CAPACITY {
            INPUT[i] = LAST_INPUT[i];
            i += 1;
        }
        serial::write_str("[recall]\r\n");
    }
}

fn complete_input() {
    unsafe {
        if INPUT_CURSOR != INPUT_LEN {
            return;
        }
        let input = as_str(&INPUT[..INPUT_LEN]);
        if let Some(candidate) = shell::complete(input) {
            let bytes = candidate.as_bytes();
            INPUT_LEN = 0;
            let mut i = 0;
            while i < bytes.len() && i < INPUT_CAPACITY {
                INPUT[i] = bytes[i];
                INPUT_LEN += 1;
                i += 1;
            }
            INPUT_CURSOR = INPUT_LEN;
            while i < INPUT_CAPACITY {
                INPUT[i] = 0;
                i += 1;
            }
            serial::write_str("[complete] ");
            serial::write_str(candidate);
            serial::write_str("\r\n");
        }
    }
}

fn clear_input() {
    unsafe {
        INPUT_LEN = 0;
        INPUT_CURSOR = 0;
        let mut i = 0;
        while i < INPUT_CAPACITY {
            INPUT[i] = 0;
            i += 1;
        }
        serial::write_str("[clear]\r\n");
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

fn as_str(bytes: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(bytes) }
}
