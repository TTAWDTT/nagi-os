use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{pit, vga};

const TRACE_CAPACITY: usize = 32;

static mut TRACE: [TraceEvent; TRACE_CAPACITY] = [TraceEvent::empty(); TRACE_CAPACITY];
static NEXT: AtomicUsize = AtomicUsize::new(0);
static LEN: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
pub enum TraceKind {
    Boot,
    Timer,
    Keyboard,
    Shell,
}

impl TraceKind {
    fn as_str(self) -> &'static str {
        match self {
            TraceKind::Boot => "BOOT",
            TraceKind::Timer => "TIMER",
            TraceKind::Keyboard => "KEYBD",
            TraceKind::Shell => "SHELL",
        }
    }
}

#[derive(Clone, Copy)]
struct TraceEvent {
    tick: u64,
    kind: TraceKind,
    value: u64,
    label: [u8; 16],
}

impl TraceEvent {
    const fn empty() -> Self {
        Self {
            tick: 0,
            kind: TraceKind::Boot,
            value: 0,
            label: [0; 16],
        }
    }
}

pub fn init() {
    NEXT.store(0, Ordering::Relaxed);
    LEN.store(0, Ordering::Relaxed);
}

pub fn record(kind: TraceKind, value: u64, label: &str) {
    let idx = NEXT.fetch_add(1, Ordering::Relaxed) % TRACE_CAPACITY;
    let mut event = TraceEvent::empty();
    event.tick = pit::ticks();
    event.kind = kind;
    event.value = value;
    copy_label(&mut event.label, label);

    unsafe {
        TRACE[idx] = event;
    }

    let len = LEN.load(Ordering::Relaxed);
    if len < TRACE_CAPACITY {
        LEN.store(len + 1, Ordering::Relaxed);
    }
}

pub fn len() -> usize {
    LEN.load(Ordering::Relaxed)
}

pub const fn capacity() -> usize {
    TRACE_CAPACITY
}

pub fn dump_to_vga(start_row: usize, max_rows: usize) {
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let len = LEN.load(Ordering::Relaxed);
    let next = NEXT.load(Ordering::Relaxed);
    let start = if len == TRACE_CAPACITY {
        next % TRACE_CAPACITY
    } else {
        0
    };

    let mut i = 0;
    while i < len && i < max_rows {
        let idx = (start + i) % TRACE_CAPACITY;
        let event = unsafe { TRACE[idx] };
        let mut line = [0u8; 80];
        let mut out = copy_bytes(&mut line, 0, b"  t=");
        out = append_u64(&mut line, out, event.tick);
        out = copy_bytes(&mut line, out, b" ");
        out = copy_bytes(&mut line, out, event.kind.as_str().as_bytes());
        out = copy_bytes(&mut line, out, b" ");
        out = copy_bytes(&mut line, out, label_as_str(&event.label).as_bytes());
        out = copy_bytes(&mut line, out, b" v=");
        out = append_u64(&mut line, out, event.value);
        vga::write_line(start_row + i, as_str(&line[..out]), color);
        i += 1;
    }
}

fn copy_label(dst: &mut [u8; 16], label: &str) {
    for (i, byte) in label.bytes().take(dst.len() - 1).enumerate() {
        dst[i] = byte;
    }
}

fn label_as_str(bytes: &[u8; 16]) -> &str {
    let mut len = 0;
    while len < bytes.len() && bytes[len] != 0 {
        len += 1;
    }
    core::str::from_utf8(&bytes[..len]).unwrap_or("?")
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

fn append_u64(buf: &mut [u8], idx: usize, mut value: u64) -> usize {
    if value == 0 {
        return copy_bytes(buf, idx, b"0");
    }

    let mut digits = [0u8; 20];
    let mut digit_idx = digits.len();
    while value > 0 {
        digit_idx -= 1;
        digits[digit_idx] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    copy_bytes(buf, idx, &digits[digit_idx..])
}

fn as_str(bytes: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(bytes) }
}
