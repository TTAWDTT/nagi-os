#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod console;
mod idt;
mod klog;
mod pic;
mod pit;
mod port;
mod serial;
mod vga;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub extern "C" fn _start() -> ! {
    serial::init();
    vga::clear_screen();
    let title = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let normal = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let ok = vga::make_color(vga::Color::LightGreen, vga::Color::Black);

    vga::write_line(0, "Nagi OS", title);
    vga::write_line(1, "A Rust-based observable teaching operating system", normal);

    klog::init();
    klog::record(klog::EventType::Boot, 0, 0, "kernel");
    klog::record(klog::EventType::Trace, 1, 0, "trace-ready");
    idt::init();
    klog::record(klog::EventType::Trace, 2, 0, "idt-ready");
    pic::init();
    klog::record(klog::EventType::Trace, 3, 0, "pic-ready");
    pit::init(100);
    pic::enable_timer();
    idt::enable_interrupts();
    klog::record(klog::EventType::Trace, 4, 100, "pit-ready");

    vga::write_line(3, "kernel: long mode is active", ok);
    vga::write_line(4, "kernel: VGA text console online", ok);
    vga::write_line(5, "kernel: serial console online", ok);
    vga::write_line(6, "kernel: event log initialized", ok);
    vga::write_line(7, "kernel: IDT exception gates loaded", ok);
    vga::write_line(8, "kernel: PIC remapped and PIT 100Hz enabled", ok);
    vga::write_line(10, "early klog: 5 events recorded", normal);
    vga::write_line(11, "status: ready for keyboard, shell, and scheduler", normal);

    serial::write_str("Nagi OS booted\r\n");

    halt_loop()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga::set_color(vga::Color::White, vga::Color::Red);
    console::println("");
    console::println("KERNEL PANIC");
    halt_loop()
}

fn halt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
