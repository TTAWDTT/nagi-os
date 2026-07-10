const COM1: u16 = 0x3F8;

static mut INITIALIZED: bool = false;

pub fn init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1, 0x03);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
        INITIALIZED = true;
    }
}

pub fn write_str(s: &str) {
    unsafe {
        if !INITIALIZED {
            return;
        }
    }
    for byte in s.bytes() {
        write_byte(byte);
    }
}

pub fn write_byte(byte: u8) {
    while unsafe { inb(COM1 + 5) } & 0x20 == 0 {}
    unsafe {
        outb(COM1, byte);
    }
}

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}
