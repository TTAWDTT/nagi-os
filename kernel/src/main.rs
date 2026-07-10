#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod console;
mod fs;
mod idt;
mod keyboard;
mod klog;
mod mem;
mod pic;
mod pit;
mod port;
mod serial;
mod shell;
mod syscall;
mod task;
mod trace;
mod ui;
mod user;
mod vga;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
pub extern "C" fn _start() -> ! {
    serial::init();
    ui::draw_desktop();

    klog::init();
    trace::init();
    trace::record(trace::TraceKind::Boot, 0, "kernel");
    klog::record(klog::EventType::Boot, 0, 0, "kernel");
    mem::init();
    task::init();
    fs::init();
    syscall::init();
    user::init();
    klog::record(klog::EventType::Trace, 1, 0, "trace-ready");
    idt::init();
    klog::record(klog::EventType::Trace, 2, 0, "idt-ready");
    pic::init();
    klog::record(klog::EventType::Trace, 3, 0, "pic-ready");
    pit::init(100);
    pic::enable_timer();
    pic::enable_keyboard();
    idt::enable_interrupts();
    klog::record(klog::EventType::Trace, 4, 100, "pit-ready");
    klog::record(klog::EventType::Trace, 5, 0, "kbd-ready");

    ui::draw_boot_line(0, "long mode", "64-bit kernel entry is active");
    ui::draw_boot_line(1, "console", "VGA text and serial output online");
    ui::draw_boot_line(2, "events", "klog and trace buffers initialized");
    ui::draw_boot_line(3, "interrupts", "IDT, PIC, PIT 100Hz, keyboard IRQ1 ready");
    ui::draw_boot_line(4, "memory", "observable physical page allocator ready");
    ui::draw_boot_line(5, "tasks", "round-robin task model ready");
    ui::draw_boot_line(6, "services", "RAMFS, syscall, and user demos ready");
    ui::draw_footer("booted");
    keyboard::init_screen();
    shell::init();

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
