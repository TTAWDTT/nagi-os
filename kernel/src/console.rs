use crate::{serial, vga};

pub fn print(s: &str) {
    vga::write_str(s);
    serial::write_str(s);
}

pub fn println(s: &str) {
    vga::write_str(s);
    vga::newline();
    serial::write_str(s);
    serial::write_str("\r\n");
}

pub fn print_u64(mut value: u64) {
    let mut buf = [0u8; 20];
    let mut i = buf.len();

    if value == 0 {
        print("0");
        return;
    }

    while value > 0 {
        i -= 1;
        buf[i] = b'0' + (value % 10) as u8;
        value /= 10;
    }

    print(unsafe { core::str::from_utf8_unchecked(&buf[i..]) });
}
