use crate::port;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;
const ICW1_INIT: u8 = 0x10;
const ICW1_ICW4: u8 = 0x01;
const ICW4_8086: u8 = 0x01;

pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = 40;

pub fn init() {
    unsafe {
        port::outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        port::io_wait();
        port::outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        port::io_wait();

        port::outb(PIC1_DATA, PIC1_OFFSET);
        port::io_wait();
        port::outb(PIC2_DATA, PIC2_OFFSET);
        port::io_wait();

        port::outb(PIC1_DATA, 4);
        port::io_wait();
        port::outb(PIC2_DATA, 2);
        port::io_wait();

        port::outb(PIC1_DATA, ICW4_8086);
        port::io_wait();
        port::outb(PIC2_DATA, ICW4_8086);
        port::io_wait();

        port::outb(PIC1_DATA, 0xFF);
        port::outb(PIC2_DATA, 0xFF);
    }
}

pub fn enable_timer() {
    unsafe {
        let mask = port::inb(PIC1_DATA) & !0x01;
        port::outb(PIC1_DATA, mask);
    }
}

pub fn end_of_interrupt(vector: u8) {
    unsafe {
        if vector >= PIC2_OFFSET {
            port::outb(PIC2_COMMAND, PIC_EOI);
        }
        port::outb(PIC1_COMMAND, PIC_EOI);
    }
}
