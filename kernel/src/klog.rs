use crate::{console, vga};

const LOG_CAPACITY: usize = 32;

static mut KLOG: KernelLog = KernelLog {
    events: [Event::empty(); LOG_CAPACITY],
    next: 0,
    len: 0,
};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum EventType {
    Boot,
    Trace,
    Syscall,
    Schedule,
    Memory,
    File,
}

impl EventType {
    fn as_str(self) -> &'static str {
        match self {
            EventType::Boot => "BOOT",
            EventType::Trace => "TRACE",
            EventType::Syscall => "SYSCALL",
            EventType::Schedule => "SCHED",
            EventType::Memory => "MEM",
            EventType::File => "FILE",
        }
    }
}

#[derive(Clone, Copy)]
struct Event {
    tick: u64,
    pid: u32,
    kind: EventType,
    arg0: u64,
    arg1: u64,
    name: [u8; 16],
}

impl Event {
    const fn empty() -> Self {
        Self {
            tick: 0,
            pid: 0,
            kind: EventType::Boot,
            arg0: 0,
            arg1: 0,
            name: [0; 16],
        }
    }
}

struct KernelLog {
    events: [Event; LOG_CAPACITY],
    next: usize,
    len: usize,
}

pub fn init() {
    unsafe {
        KLOG.next = 0;
        KLOG.len = 0;
    }
}

pub fn record(kind: EventType, arg0: u64, arg1: u64, name: &str) {
    unsafe {
        let idx = KLOG.next;
        let mut event = Event::empty();
        event.tick = KLOG.next as u64;
        event.pid = 0;
        event.kind = kind;
        event.arg0 = arg0;
        event.arg1 = arg1;
        copy_name(&mut event.name, name);
        KLOG.events[idx] = event;
        KLOG.next = (KLOG.next + 1) % LOG_CAPACITY;
        if KLOG.len < LOG_CAPACITY {
            KLOG.len += 1;
        }
    }
}

pub fn len() -> usize {
    unsafe { KLOG.len }
}

pub const fn capacity() -> usize {
    LOG_CAPACITY
}

#[allow(dead_code)]
pub fn dump_to_console() {
    unsafe {
        let start = if KLOG.len == LOG_CAPACITY { KLOG.next } else { 0 };
        for i in 0..KLOG.len {
            let idx = (start + i) % LOG_CAPACITY;
            let event = KLOG.events[idx];
            console::print("  tick=");
            console::print_u64(event.tick);
            console::print(" pid=");
            console::print_u64(event.pid as u64);
            console::print(" type=");
            console::print(event.kind.as_str());
            console::print(" a0=");
            console::print_u64(event.arg0);
            console::print(" a1=");
            console::print_u64(event.arg1);
            console::print(" name=");
            console::println(name_as_str(&event.name));
        }
    }
}

pub fn dump_to_vga(start_row: usize, col: usize, max_rows: usize) {
    unsafe {
        let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
        let start = if KLOG.len == LOG_CAPACITY { KLOG.next } else { 0 };
        let mut i = 0;
        while i < KLOG.len && i < max_rows {
            let idx = (start + i) % LOG_CAPACITY;
            let event = KLOG.events[idx];
            let mut line = [0u8; 80];
            let mut len = copy_bytes_slice(&mut line, 0, b"  ");
            len = append_u64(&mut line, len, event.tick);
            len = copy_bytes_slice(&mut line, len, b" ");
            len = copy_bytes_slice(&mut line, len, event.kind.as_str().as_bytes());
            len = copy_bytes_slice(&mut line, len, b" ");
            len = copy_bytes_slice(&mut line, len, name_as_str(&event.name).as_bytes());
            len = copy_bytes_slice(&mut line, len, b" a0=");
            len = append_u64(&mut line, len, event.arg0);
            len = copy_bytes_slice(&mut line, len, b" a1=");
            len = append_u64(&mut line, len, event.arg1);
            vga::write_at(start_row + i, col, as_str(&line[..len]), color);
            i += 1;
        }
    }
}

fn copy_name(dst: &mut [u8; 16], name: &str) {
    for (i, byte) in name.bytes().take(dst.len() - 1).enumerate() {
        dst[i] = byte;
    }
}

fn copy_bytes_slice(dst: &mut [u8], mut idx: usize, src: &[u8]) -> usize {
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
        return copy_bytes_slice(buf, idx, b"0");
    }

    let mut digits = [0u8; 20];
    let mut digit_idx = digits.len();
    while value > 0 {
        digit_idx -= 1;
        digits[digit_idx] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    copy_bytes_slice(buf, idx, &digits[digit_idx..])
}

fn name_as_str(bytes: &[u8; 16]) -> &str {
    let mut len = 0;
    while len < bytes.len() && bytes[len] != 0 {
        len += 1;
    }
    core::str::from_utf8(&bytes[..len]).unwrap_or("?")
}

fn as_str(bytes: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(bytes) }
}
