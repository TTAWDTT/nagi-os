use core::sync::atomic::{AtomicU64, Ordering};

use crate::port;

const PIT_CHANNEL0: u16 = 0x40;
const PIT_COMMAND: u16 = 0x43;
const PIT_BASE_FREQUENCY: u32 = 1_193_182;

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init(frequency_hz: u32) {
    let divisor = PIT_BASE_FREQUENCY / frequency_hz;
    unsafe {
        port::outb(PIT_COMMAND, 0x36);
        port::outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
        port::outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);
    }
}

pub fn tick() -> u64 {
    TICKS.fetch_add(1, Ordering::Relaxed) + 1
}

#[allow(dead_code)]
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
