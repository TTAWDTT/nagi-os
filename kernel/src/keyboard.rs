use core::sync::atomic::{AtomicU64, Ordering};

use crate::{port, serial, shell, trace, ui};

const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;
const INPUT_CAPACITY: usize = 62;
const HISTORY_CAPACITY: usize = 8;

static mut INPUT: [u8; INPUT_CAPACITY] = [0; INPUT_CAPACITY];
static mut INPUT_LEN: usize = 0;
static mut INPUT_CURSOR: usize = 0;
static mut HISTORY: [[u8; INPUT_CAPACITY]; HISTORY_CAPACITY] = [[0; INPUT_CAPACITY]; HISTORY_CAPACITY];
static mut HISTORY_LEN: [usize; HISTORY_CAPACITY] = [0; HISTORY_CAPACITY];
static mut HISTORY_COUNT: usize = 0;
static mut HISTORY_NEXT: usize = 0;
static mut HISTORY_NAV: usize = 0;
static mut DRAFT: [u8; INPUT_CAPACITY] = [0; INPUT_CAPACITY];
static mut DRAFT_LEN: usize = 0;
static mut CANDIDATE_INDEX: usize = 0;
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
    HistoryPrev,
    HistoryNext,
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
                begin_edit();
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
                begin_edit();
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
                begin_edit();
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
                CANDIDATE_INDEX = 0;
                let mut i = 0;
                while i < INPUT_CAPACITY {
                    INPUT[i] = 0;
                    i += 1;
                }
            }
            Key::Recall => {
                history_latest();
                trace::record(trace::TraceKind::Keyboard, INPUT_LEN as u64, "recall");
            }
            Key::HistoryPrev => {
                if can_select_candidate() {
                    select_candidate(false);
                } else {
                    history_previous();
                }
                trace::record(trace::TraceKind::Keyboard, INPUT_LEN as u64, "up");
            }
            Key::HistoryNext => {
                if can_select_candidate() {
                    select_candidate(true);
                } else {
                    history_next();
                }
                trace::record(trace::TraceKind::Keyboard, INPUT_LEN as u64, "down");
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
                if INPUT_LEN == 0 && shell::presentation_active() {
                    shell::presentation_navigate(false);
                } else if INPUT_CURSOR > 0 {
                    INPUT_CURSOR -= 1;
                    trace::record(trace::TraceKind::Keyboard, INPUT_CURSOR as u64, "left");
                }
            }
            Key::MoveRight => {
                if INPUT_LEN == 0 && shell::presentation_active() {
                    shell::presentation_navigate(true);
                } else if INPUT_CURSOR < INPUT_LEN {
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
        let suggestion = if INPUT_CURSOR == INPUT_LEN { shell::complete_at(input, CANDIDATE_INDEX) } else { None };
        let mut matches = [&""[..]; 4];
        let match_count = shell::sidebar_matches(input, &mut matches);
        if CANDIDATE_INDEX >= match_count && match_count > 0 {
            CANDIDATE_INDEX = match_count - 1;
        }
        ui::draw_sidebar(shell::current_page(), input, &matches[..match_count], CANDIDATE_INDEX);
        ui::draw_prompt(input, suggestion, INPUT_CURSOR, INPUT_LEN >= INPUT_CAPACITY);
    }
}

fn decode_scancode(scancode: u8, extended: bool) -> Option<Key> {
    if extended {
        return match scancode {
            0x48 => Some(Key::HistoryPrev),
            0x50 => Some(Key::HistoryNext),
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
        let slot = HISTORY_NEXT;
        HISTORY_LEN[slot] = INPUT_LEN;
        let mut i = 0;
        while i < INPUT_CAPACITY {
            HISTORY[slot][i] = if i < INPUT_LEN { INPUT[i] } else { 0 };
            i += 1;
        }
        HISTORY_NEXT = (HISTORY_NEXT + 1) % HISTORY_CAPACITY;
        if HISTORY_COUNT < HISTORY_CAPACITY {
            HISTORY_COUNT += 1;
        }
        HISTORY_NAV = HISTORY_COUNT;
        DRAFT_LEN = 0;
    }
}

fn history_latest() {
    unsafe {
        HISTORY_NAV = HISTORY_COUNT;
    }
    history_previous();
}

fn history_previous() {
    unsafe {
        if HISTORY_COUNT == 0 {
            return;
        }
        if HISTORY_NAV > HISTORY_COUNT {
            HISTORY_NAV = HISTORY_COUNT;
        }
        if HISTORY_NAV == HISTORY_COUNT {
            save_draft();
        }
        if HISTORY_NAV > 0 {
            HISTORY_NAV -= 1;
        }
        load_history(HISTORY_NAV);
        CANDIDATE_INDEX = 0;
    }
}

fn history_next() {
    unsafe {
        if HISTORY_NAV >= HISTORY_COUNT {
            return;
        }
        HISTORY_NAV += 1;
        if HISTORY_NAV == HISTORY_COUNT {
            load_draft();
        } else {
            load_history(HISTORY_NAV);
        }
        CANDIDATE_INDEX = 0;
    }
}

fn history_slot(logical: usize) -> usize {
    unsafe {
        let start = if HISTORY_COUNT < HISTORY_CAPACITY { 0 } else { HISTORY_NEXT };
        (start + logical) % HISTORY_CAPACITY
    }
}

fn load_history(logical: usize) {
    unsafe {
        let slot = history_slot(logical);
        INPUT_LEN = HISTORY_LEN[slot];
        let mut i = 0;
        while i < INPUT_CAPACITY {
            INPUT[i] = HISTORY[slot][i];
            i += 1;
        }
        INPUT_CURSOR = INPUT_LEN;
    }
}

fn save_draft() {
    unsafe {
        DRAFT_LEN = INPUT_LEN;
        let mut i = 0;
        while i < INPUT_CAPACITY {
            DRAFT[i] = INPUT[i];
            i += 1;
        }
    }
}

fn load_draft() {
    unsafe {
        INPUT_LEN = DRAFT_LEN;
        let mut i = 0;
        while i < INPUT_CAPACITY {
            INPUT[i] = DRAFT[i];
            i += 1;
        }
        INPUT_CURSOR = INPUT_LEN;
    }
}

fn begin_edit() {
    unsafe {
        HISTORY_NAV = HISTORY_COUNT;
        CANDIDATE_INDEX = 0;
    }
}

fn can_select_candidate() -> bool {
    unsafe {
        if INPUT_LEN == 0 || HISTORY_NAV != HISTORY_COUNT {
            return false;
        }
        let mut matches = [&""[..]; 4];
        shell::sidebar_matches(as_str(&INPUT[..INPUT_LEN]), &mut matches) > 0
    }
}

fn select_candidate(forward: bool) {
    unsafe {
        let mut matches = [&""[..]; 4];
        let count = shell::sidebar_matches(as_str(&INPUT[..INPUT_LEN]), &mut matches);
        if count == 0 {
            return;
        }
        CANDIDATE_INDEX = if forward {
            (CANDIDATE_INDEX + 1) % count
        } else if CANDIDATE_INDEX == 0 {
            count - 1
        } else {
            CANDIDATE_INDEX - 1
        };
    }
}

fn complete_input() {
    unsafe {
        if INPUT_CURSOR != INPUT_LEN {
            return;
        }
        let input = as_str(&INPUT[..INPUT_LEN]);
        if let Some(candidate) = shell::complete_at(input, CANDIDATE_INDEX) {
            let bytes = candidate.as_bytes();
            INPUT_LEN = 0;
            let mut i = 0;
            while i < bytes.len() && i < INPUT_CAPACITY {
                INPUT[i] = bytes[i];
                INPUT_LEN += 1;
                i += 1;
            }
            INPUT_CURSOR = INPUT_LEN;
            CANDIDATE_INDEX = 0;
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
        HISTORY_NAV = HISTORY_COUNT;
        CANDIDATE_INDEX = 0;
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
