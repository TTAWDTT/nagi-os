use crate::{klog, pit, serial, vga};

const OUTPUT_START_ROW: usize = 15;
const OUTPUT_ROWS: usize = 9;

pub fn init() {
    clear_output();
    write_output(0, "type 'help' and press Enter");
}

pub fn run(command: &str) {
    let command = trim(command);
    serial::write_str("shell command: ");
    serial::write_str(command);
    serial::write_str("\r\n");

    match command {
        "" => {}
        "help" => show_help(),
        "ticks" => show_ticks(),
        "klog" => show_klog(),
        "clear" => {
            clear_output();
        }
        _ => show_unknown(command),
    }
}

fn show_help() {
    clear_output();
    write_output(0, "commands:");
    write_output(1, "  help   - show this command list");
    write_output(2, "  ticks  - show PIT timer ticks");
    write_output(3, "  klog   - show early kernel events");
    write_output(4, "  clear  - clear shell output");
}

fn show_ticks() {
    clear_output();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"pit ticks: ");
    len = append_u64(&mut line, len, pit::ticks());
    write_output(0, as_str(&line[..len]));
}

fn show_klog() {
    clear_output();
    write_output(0, "early kernel log:");
    klog::dump_to_vga(OUTPUT_START_ROW + 1, OUTPUT_ROWS - 1);
}

fn show_unknown(command: &str) {
    clear_output();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"unknown command: ");
    len = copy_bytes(&mut line, len, command.as_bytes());
    write_output(0, as_str(&line[..len]));
    write_output(1, "try: help");
}

fn clear_output() {
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let mut row = 0;
    while row < OUTPUT_ROWS {
        vga::write_line(OUTPUT_START_ROW + row, "", color);
        row += 1;
    }
}

fn write_output(offset: usize, text: &str) {
    if offset >= OUTPUT_ROWS {
        return;
    }
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    vga::write_line(OUTPUT_START_ROW + offset, text, color);
}

fn trim(text: &str) -> &str {
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut end = bytes.len();

    while start < end && bytes[start] == b' ' {
        start += 1;
    }
    while end > start && bytes[end - 1] == b' ' {
        end -= 1;
    }

    unsafe { core::str::from_utf8_unchecked(&bytes[start..end]) }
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
