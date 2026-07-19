use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::{pit, vga};

const TRACE_CAPACITY: usize = 32;

static mut TRACE: [TraceEvent; TRACE_CAPACITY] = [TraceEvent::empty(); TRACE_CAPACITY];
static NEXT: AtomicUsize = AtomicUsize::new(0);
static LEN: AtomicUsize = AtomicUsize::new(0);
static ENABLED: AtomicBool = AtomicBool::new(true);
static SKIPPED: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, PartialEq)]
pub enum TraceKind {
    Boot,
    Timer,
    Keyboard,
    Shell,
    Memory,
    Schedule,
    Syscall,
    File,
    Demo,
}

impl TraceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TraceKind::Boot => "BOOT",
            TraceKind::Timer => "TIMER",
            TraceKind::Keyboard => "KEYBD",
            TraceKind::Shell => "SHELL",
            TraceKind::Memory => "MEM",
            TraceKind::Schedule => "SCHED",
            TraceKind::Syscall => "SYSC",
            TraceKind::File => "FILE",
            TraceKind::Demo => "DEMO",
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
    ENABLED.store(true, Ordering::Relaxed);
    SKIPPED.store(0, Ordering::Relaxed);
}

pub fn record(kind: TraceKind, value: u64, label: &str) {
    if !ENABLED.load(Ordering::Relaxed) {
        SKIPPED.fetch_add(1, Ordering::Relaxed);
        return;
    }

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

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn skipped() -> usize {
    SKIPPED.load(Ordering::Relaxed)
}

pub const fn capacity() -> usize {
    TRACE_CAPACITY
}

pub fn dump_to_vga(start_row: usize, col: usize, max_rows: usize) {
    dump_filtered_to_vga(start_row, col, max_rows, None);
}

pub fn dump_filtered_to_vga(start_row: usize, col: usize, max_rows: usize, filter: Option<&str>) {
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let len = LEN.load(Ordering::Relaxed);
    let next = NEXT.load(Ordering::Relaxed);
    let start = if len == TRACE_CAPACITY {
        next % TRACE_CAPACITY
    } else {
        0
    };

    let mut i = 0;
    let mut written = 0;
    while i < len && written < max_rows {
        let idx = (start + i) % TRACE_CAPACITY;
        let event = unsafe { TRACE[idx] };
        if !matches_filter(event.kind, filter) {
            i += 1;
            continue;
        }
        let mut line = [0u8; 80];
        let mut out = copy_bytes(&mut line, 0, b"  t=");
        out = append_u64(&mut line, out, event.tick);
        out = copy_bytes(&mut line, out, b" ");
        out = copy_bytes(&mut line, out, event.kind.as_str().as_bytes());
        out = copy_bytes(&mut line, out, b" ");
        out = copy_bytes(&mut line, out, label_as_str(&event.label).as_bytes());
        out = copy_bytes(&mut line, out, b" v=");
        out = append_u64(&mut line, out, event.value);
        vga::write_at(start_row + written, col, as_str(&line[..out]), color);
        written += 1;
        i += 1;
    }
}

pub fn dump_timeline_to_vga(start_row: usize, col: usize, max_rows: usize) {
    dump_story_to_vga(start_row, col, max_rows, false);
}

pub fn dump_replay_to_vga(start_row: usize, col: usize, max_rows: usize) {
    dump_story_to_vga(start_row, col, max_rows, true);
}

fn dump_story_to_vga(start_row: usize, col: usize, max_rows: usize, replay: bool) {
    let len = LEN.load(Ordering::Relaxed);
    if len == 0 || max_rows == 0 {
        return;
    }
    let next = NEXT.load(Ordering::Relaxed);
    let ring_start = if len == TRACE_CAPACITY { next % TRACE_CAPACITY } else { 0 };
    let take = core::cmp::min(len, max_rows);
    let first = len - take;
    let mut row = 0;
    while row < take {
        let idx = (ring_start + first + row) % TRACE_CAPACITY;
        let event = unsafe { TRACE[idx] };
        let mut line = [0u8; 80];
        let mut out = 0;
        if replay {
            out = copy_bytes(&mut line, out, b"t+");
            out = append_u64(&mut line, out, event.tick);
            out = copy_bytes(&mut line, out, b"  ");
            out = copy_bytes(&mut line, out, story_verb(event.kind).as_bytes());
            out = copy_bytes(&mut line, out, b" ");
            out = copy_bytes(&mut line, out, label_as_str(&event.label).as_bytes());
        } else {
            out = append_u64(&mut line, out, event.tick);
            out = copy_bytes(&mut line, out, b" | ");
            out = copy_bytes(&mut line, out, event.kind.as_str().as_bytes());
            out = copy_bytes(&mut line, out, b" | ");
            out = copy_bytes(&mut line, out, label_as_str(&event.label).as_bytes());
            out = copy_bytes(&mut line, out, b" | ");
            out = append_u64(&mut line, out, event.value);
        }
        vga::write_at(start_row + row, col, as_str(&line[..out]), kind_color(event.kind));
        row += 1;
    }
}

fn story_verb(kind: TraceKind) -> &'static str {
    match kind {
        TraceKind::Boot => "booted",
        TraceKind::Timer => "timer fired:",
        TraceKind::Keyboard => "input event:",
        TraceKind::Shell => "shell ran:",
        TraceKind::Memory => "page changed:",
        TraceKind::Schedule => "task switched:",
        TraceKind::Syscall => "syscall crossed:",
        TraceKind::File => "file changed:",
        TraceKind::Demo => "demo marked:",
    }
}

fn kind_color(kind: TraceKind) -> u8 {
    let fg = match kind {
        TraceKind::Memory => vga::Color::LightBlue,
        TraceKind::Schedule => vga::Color::LightGreen,
        TraceKind::Syscall => vga::Color::LightCyan,
        TraceKind::File => vga::Color::Yellow,
        TraceKind::Timer | TraceKind::Keyboard => vga::Color::DarkGray,
        _ => vga::Color::LightGray,
    };
    vga::make_color(fg, vga::Color::Black)
}

fn matches_filter(kind: TraceKind, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some("irq") => kind == TraceKind::Timer || kind == TraceKind::Keyboard,
        Some("boot") => kind == TraceKind::Boot,
        Some("timer") => kind == TraceKind::Timer,
        Some("kbd") | Some("key") | Some("keyboard") => kind == TraceKind::Keyboard,
        Some("shell") => kind == TraceKind::Shell,
        Some("mem") | Some("memory") => kind == TraceKind::Memory,
        Some("sched") | Some("schedule") => kind == TraceKind::Schedule,
        Some("syscall") | Some("sys") => kind == TraceKind::Syscall,
        Some("file") | Some("fs") => kind == TraceKind::File,
        Some("demo") => kind == TraceKind::Demo,
        Some(_) => false,
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
