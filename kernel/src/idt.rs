use core::arch::{asm, global_asm};
use core::cell::UnsafeCell;
use core::mem::size_of;

use crate::{keyboard, pic, pit, serial, trace, vga};

const IDT_LEN: usize = 256;
const KERNEL_CODE_SELECTOR: u16 = 0x18;
const INTERRUPT_GATE: u8 = 0x8E;

global_asm!(
    r#"
.section .text
.global isr0
isr0:
    cld
    mov rdi, 0
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr1
isr1:
    cld
    mov rdi, 1
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr2
isr2:
    cld
    mov rdi, 2
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr3
isr3:
    cld
    mov rdi, 3
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr4
isr4:
    cld
    mov rdi, 4
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr5
isr5:
    cld
    mov rdi, 5
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr6
isr6:
    cld
    mov rdi, 6
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr7
isr7:
    cld
    mov rdi, 7
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr8
isr8:
    cld
    mov rdi, 8
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr9
isr9:
    cld
    mov rdi, 9
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr10
isr10:
    cld
    mov rdi, 10
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr11
isr11:
    cld
    mov rdi, 11
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr12
isr12:
    cld
    mov rdi, 12
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr13
isr13:
    cld
    mov rdi, 13
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr14
isr14:
    cld
    mov rdi, 14
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr15
isr15:
    cld
    mov rdi, 15
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr16
isr16:
    cld
    mov rdi, 16
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr17
isr17:
    cld
    mov rdi, 17
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr18
isr18:
    cld
    mov rdi, 18
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr19
isr19:
    cld
    mov rdi, 19
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr20
isr20:
    cld
    mov rdi, 20
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr21
isr21:
    cld
    mov rdi, 21
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr22
isr22:
    cld
    mov rdi, 22
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr23
isr23:
    cld
    mov rdi, 23
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr24
isr24:
    cld
    mov rdi, 24
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr25
isr25:
    cld
    mov rdi, 25
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr26
isr26:
    cld
    mov rdi, 26
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr27
isr27:
    cld
    mov rdi, 27
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr28
isr28:
    cld
    mov rdi, 28
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.global isr29
isr29:
    cld
    mov rdi, 29
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr30
isr30:
    cld
    mov rdi, 30
    mov rsi, [rsp]
    lea rdx, [rsp + 8]
    call rust_exception_handler

.global isr31
isr31:
    cld
    mov rdi, 31
    xor rsi, rsi
    mov rdx, rsp
    call rust_exception_handler

.macro irq_stub name vector
.global \name
\name:
    cld
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    mov rdi, \vector
    call rust_irq_handler
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax
    iretq
.endm

irq_stub irq0, 32
irq_stub irq1, 33
irq_stub irq2, 34
irq_stub irq3, 35
irq_stub irq4, 36
irq_stub irq5, 37
irq_stub irq6, 38
irq_stub irq7, 39
irq_stub irq8, 40
irq_stub irq9, 41
irq_stub irq10, 42
irq_stub irq11, 43
irq_stub irq12, 44
irq_stub irq13, 45
irq_stub irq14, 46
irq_stub irq15, 47
"#
);

unsafe extern "C" {
    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();
    fn irq0();
    fn irq1();
    fn irq2();
    fn irq3();
    fn irq4();
    fn irq5();
    fn irq6();
    fn irq7();
    fn irq8();
    fn irq9();
    fn irq10();
    fn irq11();
    fn irq12();
    fn irq13();
    fn irq14();
    fn irq15();
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    fn set_handler(&mut self, handler: unsafe extern "C" fn()) {
        let addr = handler as usize as u64;
        self.offset_low = addr as u16;
        self.selector = KERNEL_CODE_SELECTOR;
        self.ist = 0;
        self.type_attr = INTERRUPT_GATE;
        self.offset_mid = (addr >> 16) as u16;
        self.offset_high = (addr >> 32) as u32;
        self.zero = 0;
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

struct IdtTable(UnsafeCell<[IdtEntry; IDT_LEN]>);

unsafe impl Sync for IdtTable {}

static IDT: IdtTable = IdtTable(UnsafeCell::new([IdtEntry::missing(); IDT_LEN]));

#[repr(C)]
pub struct InterruptFrame {
    rip: u64,
    cs: u64,
    rflags: u64,
}

pub fn init() {
    unsafe {
        let idt = &mut *IDT.0.get();
        idt[0].set_handler(isr0);
        idt[1].set_handler(isr1);
        idt[2].set_handler(isr2);
        idt[3].set_handler(isr3);
        idt[4].set_handler(isr4);
        idt[5].set_handler(isr5);
        idt[6].set_handler(isr6);
        idt[7].set_handler(isr7);
        idt[8].set_handler(isr8);
        idt[9].set_handler(isr9);
        idt[10].set_handler(isr10);
        idt[11].set_handler(isr11);
        idt[12].set_handler(isr12);
        idt[13].set_handler(isr13);
        idt[14].set_handler(isr14);
        idt[15].set_handler(isr15);
        idt[16].set_handler(isr16);
        idt[17].set_handler(isr17);
        idt[18].set_handler(isr18);
        idt[19].set_handler(isr19);
        idt[20].set_handler(isr20);
        idt[21].set_handler(isr21);
        idt[22].set_handler(isr22);
        idt[23].set_handler(isr23);
        idt[24].set_handler(isr24);
        idt[25].set_handler(isr25);
        idt[26].set_handler(isr26);
        idt[27].set_handler(isr27);
        idt[28].set_handler(isr28);
        idt[29].set_handler(isr29);
        idt[30].set_handler(isr30);
        idt[31].set_handler(isr31);
        idt[32].set_handler(irq0);
        idt[33].set_handler(irq1);
        idt[34].set_handler(irq2);
        idt[35].set_handler(irq3);
        idt[36].set_handler(irq4);
        idt[37].set_handler(irq5);
        idt[38].set_handler(irq6);
        idt[39].set_handler(irq7);
        idt[40].set_handler(irq8);
        idt[41].set_handler(irq9);
        idt[42].set_handler(irq10);
        idt[43].set_handler(irq11);
        idt[44].set_handler(irq12);
        idt[45].set_handler(irq13);
        idt[46].set_handler(irq14);
        idt[47].set_handler(irq15);

        let pointer = IdtPointer {
            limit: (size_of::<[IdtEntry; IDT_LEN]>() - 1) as u16,
            base: idt.as_ptr() as u64,
        };

        asm!("lidt [{}]", in(reg) &pointer, options(readonly, nostack, preserves_flags));
    }
}

pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_exception_handler(
    vector: u64,
    error_code: u64,
    frame: *const InterruptFrame,
) -> ! {
    let title = vga::make_color(vga::Color::White, vga::Color::Red);
    let label = vga::make_color(vga::Color::LightRed, vga::Color::Black);
    let normal = vga::make_color(vga::Color::LightGray, vga::Color::Black);

    vga::clear_screen();
    vga::write_line(0, "Nagi OS kernel exception", title);
    vga::write_line(2, exception_name(vector), label);
    write_decimal_line(4, "vector: ", vector, normal);
    write_hex_line(5, "error code: 0x", error_code, normal);

    if !frame.is_null() {
        let rip = unsafe { (*frame).rip };
        write_hex_line(6, "rip: 0x", rip, normal);
    }

    vga::write_line(8, "system halted to preserve diagnostic state", normal);

    serial::write_str("Nagi OS kernel exception\r\n");
    serial::write_str("exception: ");
    serial::write_str(exception_name(vector));
    serial::write_str("\r\nvector: ");
    write_serial_u64(vector);
    serial::write_str("\r\nerror code: 0x");
    write_serial_hex(error_code);
    if !frame.is_null() {
        serial::write_str("\nrip: 0x");
        write_serial_hex(unsafe { (*frame).rip });
    }
    serial::write_str("\r\nsystem halted\r\n");

    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_irq_handler(vector: u64) {
    if vector == pic::PIC1_OFFSET as u64 {
        let tick = pit::tick();
        if tick == 1 {
            serial::write_str("PIT timer interrupt online\r\n");
            trace::record(trace::TraceKind::Timer, tick, "irq0-online");
        } else if tick % 100 == 0 {
            trace::record(trace::TraceKind::Timer, tick, "irq0-1s");
        }
    } else if vector == pic::PIC1_OFFSET as u64 + 1 {
        keyboard::handle_interrupt();
    }

    pic::end_of_interrupt(vector as u8);
}

fn exception_name(vector: u64) -> &'static str {
    match vector {
        0 => "divide error",
        1 => "debug exception",
        2 => "non-maskable interrupt",
        3 => "breakpoint",
        4 => "overflow",
        5 => "bound range exceeded",
        6 => "invalid opcode",
        7 => "device not available",
        8 => "double fault",
        10 => "invalid TSS",
        11 => "segment not present",
        12 => "stack-segment fault",
        13 => "general protection fault",
        14 => "page fault",
        16 => "x87 floating-point exception",
        17 => "alignment check",
        18 => "machine check",
        19 => "SIMD floating-point exception",
        20 => "virtualization exception",
        21 => "control protection exception",
        28 => "hypervisor injection exception",
        29 => "VMM communication exception",
        30 => "security exception",
        _ => "reserved CPU exception",
    }
}

fn write_decimal_line(row: usize, prefix: &str, value: u64, color: u8) {
    let mut buf = [0u8; 80];
    let mut len = copy_bytes(&mut buf, 0, prefix.as_bytes());
    len = append_u64(&mut buf, len, value);
    vga::write_line(row, as_str(&buf[..len]), color);
}

fn write_hex_line(row: usize, prefix: &str, value: u64, color: u8) {
    let mut buf = [0u8; 80];
    let mut len = copy_bytes(&mut buf, 0, prefix.as_bytes());
    len = append_hex(&mut buf, len, value);
    vga::write_line(row, as_str(&buf[..len]), color);
}

fn write_serial_u64(value: u64) {
    let mut buf = [0u8; 20];
    let len = append_u64(&mut buf, 0, value);
    serial::write_str(as_str(&buf[..len]));
}

fn write_serial_hex(value: u64) {
    let mut buf = [0u8; 16];
    let len = append_hex(&mut buf, 0, value);
    serial::write_str(as_str(&buf[..len]));
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

fn append_hex(buf: &mut [u8], mut idx: usize, value: u64) -> usize {
    let mut started = false;
    for shift in (0..16).rev() {
        let nibble = ((value >> (shift * 4)) & 0xF) as u8;
        if nibble != 0 || started || shift == 0 {
            started = true;
            if idx >= buf.len() {
                break;
            }
            buf[idx] = match nibble {
                0..=9 => b'0' + nibble,
                _ => b'a' + nibble - 10,
            };
            idx += 1;
        }
    }
    idx
}

fn as_str(bytes: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(bytes) }
}
