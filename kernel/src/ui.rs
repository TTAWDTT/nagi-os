use crate::{pit, trace, vga};

pub const PROMPT_ROW: usize = 13;
pub const OUTPUT_TITLE_ROW: usize = 14;
pub const OUTPUT_START_ROW: usize = 15;
pub const OUTPUT_ROWS: usize = 9;
const FOOTER_ROW: usize = 24;

pub fn draw_desktop() {
    vga::clear_screen();
    draw_header();
    draw_footer("ready");
    draw_output_title("welcome");
}

pub fn draw_header() {
    let title = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let muted = vga::make_color(vga::Color::DarkGray, vga::Color::Black);

    vga::write_line(0, "", muted);
    vga::write_at(0, 2, "Nagi OS", title);
    vga::write_at(0, 12, "quiet observable kernel", muted);
    vga::write_at(0, 61, "h help  g guide", muted);
    vga::write_line(1, "", muted);
    vga::write_at(1, 2, "Tab/Right completes.  F1/Up recalls.  Esc clears.", muted);
    vga::write_line(2, "", muted);
}

pub fn draw_boot_line(row: usize, label: &str, text: &str) {
    let label_color = vga::make_color(vga::Color::LightGreen, vga::Color::Black);
    let text_color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let screen_row = 3 + row;
    if screen_row >= PROMPT_ROW {
        return;
    }
    vga::write_line(screen_row, "", text_color);
    vga::write_at(screen_row, 2, "[ok]", label_color);
    vga::write_at(screen_row, 8, label, label_color);
    vga::write_at(screen_row, 24, text, text_color);
}

pub fn draw_output_title(title: &str) {
    let color = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    let accent = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    vga::write_line(OUTPUT_TITLE_ROW, "", color);
    vga::write_at(OUTPUT_TITLE_ROW, 2, title, accent);
}

pub fn clear_output(title: &str) {
    draw_output_title(title);
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let mut row = 0;
    while row < OUTPUT_ROWS {
        vga::write_line(OUTPUT_START_ROW + row, "", color);
        row += 1;
    }
}

pub fn draw_footer(status: &str) {
    let color = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    vga::write_line(FOOTER_ROW, "", color);
    vga::write_at(FOOTER_ROW, 2, "ticks ", color);
    write_u64_at(FOOTER_ROW, 8, pit::ticks(), color);
    vga::write_at(FOOTER_ROW, 22, "trace ", color);
    if trace::is_enabled() {
        vga::write_at(FOOTER_ROW, 28, "on ", color);
    } else {
        vga::write_at(FOOTER_ROW, 28, "off", color);
    }
    vga::write_at(FOOTER_ROW, 39, status, color);
    vga::write_at(FOOTER_ROW, 59, "Enter to run", color);
}

pub fn draw_prompt(input: &str, suggestion: Option<&str>, full: bool) {
    let base = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let prompt = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let ghost = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    let warn = vga::make_color(vga::Color::Yellow, vga::Color::Black);
    vga::write_line(PROMPT_ROW, "", base);
    vga::write_at(PROMPT_ROW, 2, ">", prompt);
    vga::write_at(PROMPT_ROW, 4, input, base);
    let cursor_col = 4 + input.len();

    if let Some(candidate) = suggestion {
        if starts_with(candidate, input) && candidate.len() > input.len() {
            vga::write_at(PROMPT_ROW, cursor_col, &candidate[input.len()..], ghost);
        }
    }

    if cursor_col < 80 {
        vga::set_cursor(PROMPT_ROW, cursor_col);
    }

    if full {
        vga::write_at(PROMPT_ROW, 70, "full", warn);
    }
}

fn starts_with(text: &str, prefix: &str) -> bool {
    let text = text.as_bytes();
    let prefix = prefix.as_bytes();
    if prefix.len() > text.len() {
        return false;
    }
    let mut i = 0;
    while i < prefix.len() {
        if text[i] != prefix[i] {
            return false;
        }
        i += 1;
    }
    true
}

fn write_u64_at(row: usize, col: usize, value: u64, color: u8) {
    let mut buf = [0u8; 20];
    let len = append_u64(&mut buf, 0, value);
    vga::write_at(row, col, as_str(&buf[..len]), color);
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

fn as_str(bytes: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(bytes) }
}
