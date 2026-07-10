use core::ptr::{read_volatile, write_volatile};

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_CTRL: u16 = 0x3D4;
const VGA_DATA: u16 = 0x3D5;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;

static mut WRITER: Writer = Writer {
    row: 0,
    col: 0,
    color: color_code(Color::LightGray, Color::Black),
};

#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

const fn color_code(fg: Color, bg: Color) -> u8 {
    (bg as u8) << 4 | (fg as u8)
}

pub fn set_color(fg: Color, bg: Color) {
    unsafe {
        WRITER.color = color_code(fg, bg);
    }
}

pub fn clear_screen() {
    unsafe {
        reset_origin();
        WRITER.row = 0;
        WRITER.col = 0;
        WRITER.color = color_code(Color::LightGray, Color::Black);
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                write_cell(row, col, b' ', WRITER.color);
            }
        }
    }
}

pub fn write_str(s: &str) {
    unsafe {
        let writer = core::ptr::addr_of_mut!(WRITER);
        for byte in s.bytes() {
            (*writer).write_byte(byte);
        }
    }
}

pub fn newline() {
    unsafe {
        let writer = core::ptr::addr_of_mut!(WRITER);
        (*writer).newline();
    }
}

pub fn write_line(row: usize, text: &str, color: u8) {
    if row >= HEIGHT {
        return;
    }
    for col in 0..WIDTH {
        write_cell(row, col, b' ', color);
    }
    for (col, byte) in text.bytes().take(WIDTH).enumerate() {
        write_cell(row, col, byte, color);
    }
}

pub fn write_at(row: usize, col: usize, text: &str, color: u8) {
    if row >= HEIGHT || col >= WIDTH {
        return;
    }
    for (offset, byte) in text.bytes().take(WIDTH - col).enumerate() {
        write_cell(row, col + offset, byte, color);
    }
}

pub fn set_cursor(row: usize, col: usize) {
    if row >= HEIGHT || col >= WIDTH {
        return;
    }
    let position = (row * WIDTH + col) as u16;
    unsafe {
        set_crtc_register(0x0E, (position >> 8) as u8);
        set_crtc_register(0x0F, (position & 0xFF) as u8);
    }
}

pub fn make_color(fg: Color, bg: Color) -> u8 {
    color_code(fg, bg)
}

struct Writer {
    row: usize,
    col: usize,
    color: u8,
}

impl Writer {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.col >= WIDTH {
                    self.newline();
                }
                write_cell(self.row, self.col, byte, self.color);
                self.col += 1;
            }
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row + 1 >= HEIGHT {
            self.scroll();
        } else {
            self.row += 1;
        }
    }

    fn scroll(&mut self) {
        for row in 1..HEIGHT {
            for col in 0..WIDTH {
                let from = cell_offset(row, col);
                let to = cell_offset(row - 1, col);
                unsafe {
                    let ch = read_volatile(VGA_BUFFER.add(from));
                    let color = read_volatile(VGA_BUFFER.add(from + 1));
                    write_volatile(VGA_BUFFER.add(to), ch);
                    write_volatile(VGA_BUFFER.add(to + 1), color);
                }
            }
        }
        for col in 0..WIDTH {
            write_cell(HEIGHT - 1, col, b' ', self.color);
        }
    }
}

fn write_cell(row: usize, col: usize, byte: u8, color: u8) {
    let offset = cell_offset(row, col);
    unsafe {
        write_volatile(VGA_BUFFER.add(offset), byte);
        write_volatile(VGA_BUFFER.add(offset + 1), color);
    }
}

const fn cell_offset(row: usize, col: usize) -> usize {
    (row * WIDTH + col) * 2
}

unsafe fn reset_origin() {
    unsafe {
        set_crtc_register(0x0C, 0);
        set_crtc_register(0x0D, 0);
        set_crtc_register(0x0E, 0);
        set_crtc_register(0x0F, 0);
    }
}

unsafe fn set_crtc_register(index: u8, value: u8) {
    unsafe {
        outb(VGA_CTRL, index);
        outb(VGA_DATA, value);
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
