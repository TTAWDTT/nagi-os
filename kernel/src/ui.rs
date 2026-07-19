use crate::{pit, trace, vga};

pub const SIDEBAR_COL: usize = 2;
pub const SIDEBAR_WIDTH: usize = 18;
pub const CONTENT_COL: usize = 24;
pub const CONTENT_WIDTH: usize = 56;
pub const PROMPT_ROW: usize = 13;
pub const OUTPUT_TITLE_ROW: usize = 14;
pub const OUTPUT_START_ROW: usize = 15;
pub const OUTPUT_ROWS: usize = 9;
const FOOTER_ROW: usize = 24;
const LOGO_COL: usize = 60;

pub fn draw_desktop() {
    vga::clear_screen();
    draw_header();
    draw_sidebar("welcome", "", &[]);
    draw_footer("ready");
    draw_output_title("welcome");
}

pub fn draw_header() {
    let title = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let muted = vga::make_color(vga::Color::DarkGray, vga::Color::Black);

    vga::write_line(0, "", muted);
    vga::write_at(0, 2, "Nagi OS", title);
    vga::write_at(0, 12, "quiet observable kernel", muted);
    draw_logo_frame(0);
    vga::write_line(1, "", muted);
    vga::write_at(1, 2, "Tab/Right", title);
    vga::write_at(1, 12, "complete", muted);
    vga::write_at(1, 24, "Left/Right", title);
    vga::write_at(1, 35, "move", muted);
    vga::write_at(1, 43, "F1/Up", title);
    vga::write_at(1, 49, "recall", muted);
    vga::write_at(1, 58, "Esc", title);
    vga::write_at(1, 62, "clear", muted);
    vga::write_line(2, "", muted);
    vga::write_at(2, 22, "|", muted);
}

pub fn animate_logo(tick: u64) {
    if tick % 8 != 0 {
        return;
    }
    draw_logo_frame((tick / 8) as usize);
}

pub fn draw_sidebar(page: &str, input: &str, matches: &[&str]) {
    let panel = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let muted = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    let accent = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let soft = vga::make_color(vga::Color::LightBlue, vga::Color::Black);
    let active = vga::make_color(vga::Color::LightGreen, vga::Color::Black);

    clear_left_panel(panel);
    write_left(3, 2, "command panel", accent);
    write_left(4, 2, "page:", muted);
    write_left(4, 8, page, active);
    write_left(5, 2, "filter:", muted);
    if input.is_empty() {
        write_left(5, 10, "type letters", soft);
    } else {
        write_left(5, 10, input, soft);
    }

    write_left(7, 2, "h help", panel);
    write_left(7, 12, "s status", panel);
    write_left(8, 2, "m memory", panel);
    write_left(8, 12, "p tasks", panel);
    write_left(9, 2, "t timeline", panel);
    write_left(9, 12, "g guide", panel);
    write_left(10, 2, "r run", panel);
    write_left(10, 12, "f files", panel);

    write_left(12, 2, "matches", accent);
    if matches.is_empty() {
        write_left(13, 2, "Tab/Right accepts", muted);
    } else {
        let mut row = 13;
        let mut i = 0;
        while i < matches.len() && row <= 15 {
            let color = if i == 0 { active } else { panel };
            write_left(row, 2, matches[i], color);
            row += 1;
            i += 1;
        }
    }

    vga::write_at(23, 2, "Enter runs", muted);
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
    clear_right_panel(color);
    vga::write_at(OUTPUT_TITLE_ROW, CONTENT_COL, title, accent);
    vga::write_at(OUTPUT_TITLE_ROW, CONTENT_COL + title.len() + 1, "---------------------------", color);
}

pub fn clear_output(title: &str) {
    draw_output_title(title);
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let mut row = 0;
    while row < OUTPUT_ROWS {
        clear_right_row(OUTPUT_START_ROW + row, color);
        row += 1;
    }
}

pub fn draw_badge(row: usize, label: &str, text: &str) {
    if row >= OUTPUT_ROWS {
        return;
    }
    let badge = vga::make_color(vga::Color::Black, vga::Color::LightCyan);
    let body = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let screen_row = OUTPUT_START_ROW + row;
    vga::write_at(screen_row, content_text_col(), " ", badge);
    vga::write_at(screen_row, content_text_col() + 1, clip(label, 8), badge);
    vga::write_at(screen_row, content_text_col() + 1 + core::cmp::min(label.len(), 8), " ", badge);
    vga::write_at(screen_row, content_text_col() + 12, clip(text, 42), body);
}

pub fn draw_metric(row: usize, slot: usize, label: &str, value: u64) {
    if row >= OUTPUT_ROWS || slot > 1 {
        return;
    }
    let label_color = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    let value_color = vga::make_color(vga::Color::LightGreen, vga::Color::Black);
    let col = content_text_col() + slot * 27;
    vga::write_at(OUTPUT_START_ROW + row, col, clip(label, 14), label_color);
    write_u64_at(OUTPUT_START_ROW + row, col + 15, value, value_color);
}

pub fn draw_table_header(row: usize, text: &str) {
    if row >= OUTPUT_ROWS {
        return;
    }
    let color = vga::make_color(vga::Color::LightBlue, vga::Color::Black);
    vga::write_at(OUTPUT_START_ROW + row, content_text_col(), clip(text, 52), color);
}

pub fn draw_progress(row: usize, label: &str, used: usize, total: usize) {
    if row >= OUTPUT_ROWS || total == 0 {
        return;
    }
    let label_color = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    let used_color = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let free_color = vga::make_color(vga::Color::Blue, vga::Color::Black);
    let screen_row = OUTPUT_START_ROW + row;
    vga::write_at(screen_row, content_text_col(), clip(label, 12), label_color);
    let filled = core::cmp::min(24, used.saturating_mul(24) / total);
    let mut i = 0;
    while i < 24 {
        vga::write_at(screen_row, content_text_col() + 14 + i, if i < filled { "#" } else { "." }, if i < filled { used_color } else { free_color });
        i += 1;
    }
}

pub fn draw_next(text: &str) {
    let muted = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    let accent = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    clear_right_row(OUTPUT_START_ROW + OUTPUT_ROWS - 1, muted);
    vga::write_at(OUTPUT_START_ROW + OUTPUT_ROWS - 1, content_text_col(), ">", accent);
    vga::write_at(OUTPUT_START_ROW + OUTPUT_ROWS - 1, content_text_col() + 2, clip(text, 50), muted);
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

pub fn draw_footer_path(page: &str, status: &str) {
    let color = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    draw_footer(status);
    vga::write_at(FOOTER_ROW, 39, "nagi / ", color);
    vga::write_at(FOOTER_ROW, 46, clip(page, 10), color);
}

pub fn draw_logo_card() {
    let accent = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let muted = vga::make_color(vga::Color::DarkGray, vga::Color::Black);
    let soft = vga::make_color(vga::Color::LightBlue, vga::Color::Black);

    vga::write_at(OUTPUT_START_ROW + 1, content_text_col(), "N  A  G  I", accent);
    vga::write_at(OUTPUT_START_ROW + 2, content_text_col(), "     ~~~     ~", soft);
    vga::write_at(OUTPUT_START_ROW + 3, content_text_col(), "calm kernel, observable motion", muted);
    vga::write_at(OUTPUT_START_ROW + 5, content_text_col(), "The top-right mark is driven by PIT ticks.", muted);
}

pub fn draw_prompt(input: &str, suggestion: Option<&str>, cursor: usize, full: bool) {
    let base = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let input_color = vga::make_color(vga::Color::LightBlue, vga::Color::Black);
    let prompt = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let ghost = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let warn = vga::make_color(vga::Color::Yellow, vga::Color::Black);

    clear_right_row(PROMPT_ROW, base);
    vga::write_at(PROMPT_ROW, CONTENT_COL, ">", prompt);
    vga::write_at(PROMPT_ROW, CONTENT_COL + 2, input, input_color);
    if let Some(candidate) = suggestion {
        if cursor == input.len() && starts_with(candidate, input) && candidate.len() > input.len() {
            vga::write_at(PROMPT_ROW, CONTENT_COL + 2 + input.len(), tail(candidate, input.len()), ghost);
        }
    }
    let cursor_col = CONTENT_COL + 2 + cursor;

    if cursor_col < 80 {
        vga::set_cursor(PROMPT_ROW, cursor_col);
    }

    if full {
        vga::write_at(PROMPT_ROW, 72, "full", warn);
    }
}

fn clear_left_panel(color: u8) {
    let mut row = 3;
    while row < 24 {
        let mut col = 0;
        while col < SIDEBAR_WIDTH + SIDEBAR_COL {
            vga::write_at(row, col, " ", color);
            col += 1;
        }
        vga::write_at(row, 22, "|", color);
        row += 1;
    }
}

fn draw_logo_frame(frame: usize) {
    let accent = vga::make_color(vga::Color::LightCyan, vga::Color::Black);
    let wind_color = vga::make_color(vga::Color::LightBlue, vga::Color::Black);
    let wind = match frame % 8 {
        0 => "~      ",
        1 => "~~     ",
        2 => "~~~    ",
        3 => " ~~~   ",
        4 => "  ~~~  ",
        5 => "   ~~~ ",
        6 => "    ~~ ",
        _ => "     ~ ",
    };

    vga::write_at(0, LOGO_COL, "NAGI", accent);
    vga::write_at(0, LOGO_COL + 5, wind, wind_color);
}

const fn content_text_col() -> usize {
    CONTENT_COL + 2
}

fn clear_right_panel(color: u8) {
    let mut row = OUTPUT_TITLE_ROW;
    while row <= 23 {
        clear_right_row(row, color);
        row += 1;
    }
}

fn clear_right_row(row: usize, color: u8) {
    let mut col = CONTENT_COL;
    while col < CONTENT_COL + CONTENT_WIDTH {
        vga::write_at(row, col, " ", color);
        col += 1;
    }
}

fn write_left(row: usize, col: usize, text: &str, color: u8) {
    let text = clip(text, SIDEBAR_WIDTH.saturating_sub(col));
    vga::write_at(row, col, text, color);
}

fn tail(text: &str, start: usize) -> &str {
    unsafe { core::str::from_utf8_unchecked(&text.as_bytes()[start..]) }
}

fn clip(text: &str, width: usize) -> &str {
    let bytes = text.as_bytes();
    let len = core::cmp::min(bytes.len(), width);
    unsafe { core::str::from_utf8_unchecked(&bytes[..len]) }
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
