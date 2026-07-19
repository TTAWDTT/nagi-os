use core::sync::atomic::{AtomicU64, Ordering};

use crate::{klog, pit, serial, trace, vga};

pub const SYS_WRITE: u64 = 1;
pub const SYS_TIME: u64 = 2;
pub const SYS_TRACE: u64 = 3;
pub const SYS_STATS: u64 = 4;

static CALLS: AtomicU64 = AtomicU64::new(0);
static LAST_NUMBER: AtomicU64 = AtomicU64::new(0);
static LAST_RETURN: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    CALLS.store(0, Ordering::Relaxed);
    LAST_NUMBER.store(0, Ordering::Relaxed);
    LAST_RETURN.store(0, Ordering::Relaxed);
    trace::record(trace::TraceKind::Syscall, 0, "sys-init");
}

pub fn invoke(number: u64, arg0: u64, label: &str) -> u64 {
    CALLS.fetch_add(1, Ordering::Relaxed);
    LAST_NUMBER.store(number, Ordering::Relaxed);
    trace::record(trace::TraceKind::Syscall, number, label);
    klog::record(klog::EventType::Syscall, number, arg0, label);

    let ret = match number {
        SYS_WRITE => {
            serial::write_str("sys_write: ");
            serial::write_str(label);
            serial::write_str("\r\n");
            arg0
        }
        SYS_TIME => pit::ticks(),
        SYS_TRACE => {
            trace::record(trace::TraceKind::Syscall, arg0, "user-trace");
            0
        }
        SYS_STATS => CALLS.load(Ordering::Relaxed),
        _ => u64::MAX,
    };

    LAST_RETURN.store(ret, Ordering::Relaxed);
    ret
}

pub fn calls() -> u64 {
    CALLS.load(Ordering::Relaxed)
}

pub fn last_number() -> u64 {
    LAST_NUMBER.load(Ordering::Relaxed)
}

pub fn last_return() -> u64 {
    LAST_RETURN.load(Ordering::Relaxed)
}

pub fn dump_table_to_vga(start_row: usize, col: usize, max_rows: usize) {
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let rows = [
        (SYS_WRITE, "write", "serial output"),
        (SYS_TIME, "time", "read PIT ticks"),
        (SYS_TRACE, "trace", "emit trace event"),
        (SYS_STATS, "stats", "read syscall stats"),
    ];
    let mut i = 0;
    while i < rows.len() && i < max_rows {
        let (number, name, desc) = rows[i];
        let mut line = [0u8; 80];
        let mut len = copy_bytes(&mut line, 0, b"  ");
        len = append_u64(&mut line, len, number);
        len = copy_bytes(&mut line, len, b" ");
        len = copy_bytes(&mut line, len, name.as_bytes());
        len = copy_bytes(&mut line, len, b" - ");
        len = copy_bytes(&mut line, len, desc.as_bytes());
        vga::write_at(start_row + i, col, as_str(&line[..len]), color);
        i += 1;
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
