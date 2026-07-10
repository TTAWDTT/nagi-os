use crate::{keyboard, klog, mem, pit, serial, task, trace, vga};

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
    trace::record(trace::TraceKind::Shell, command.len() as u64, command);

    match command {
        "" => {}
        "help" => show_help(),
        "ticks" => show_ticks(),
        "sysstat" => show_sysstat(),
        "mem" => show_mem(),
        "ps" => show_ps(),
        "sched" => show_sched(),
        "klog" => show_klog(),
        "trace" => show_trace(),
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
    write_output(3, "  sysstat - show observable kernel stats");
    write_output(4, "  mem    - show physical page allocator");
    write_output(5, "  ps     - list kernel task model");
    write_output(6, "  sched  - show scheduler state");
    write_output(7, "  klog trace clear");
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

fn show_sysstat() {
    clear_output();
    let ticks = pit::ticks();
    write_output(0, "Nagi OS system status:");
    write_stat_line(1, "pit ticks", ticks);
    write_stat_line(2, "uptime seconds", ticks / pit::CONFIGURED_FREQUENCY_HZ as u64);
    write_stat_line(3, "timer irq0", ticks);
    write_stat_line(4, "keyboard irq1", keyboard::irq_count());
    write_stat_pair(5, "klog events", klog::len() as u64, klog::capacity() as u64);
    write_stat_pair(6, "trace events", trace::len() as u64, trace::capacity() as u64);
    let memory = mem::stats();
    write_stat_pair(7, "memory pages", memory.used_pages as u64, memory.total_pages as u64);
    write_stat_pair(8, "tasks/switches", task::count() as u64, task::switches() as u64);
}

fn show_trace() {
    clear_output();
    write_output(0, "recent trace events:");
    trace::dump_to_vga(OUTPUT_START_ROW + 1, OUTPUT_ROWS - 1);
}

fn show_mem() {
    clear_output();
    let stats = mem::stats();
    write_output(0, "physical page allocator:");
    write_stat_line(1, "page size bytes", stats.page_size as u64);
    write_stat_line(2, "total pages", stats.total_pages as u64);
    write_stat_line(3, "reserved pages", stats.reserved_pages as u64);
    write_stat_pair(4, "used/free pages", stats.used_pages as u64, stats.free_pages as u64);
    write_stat_line(5, "alloc calls", stats.allocations as u64);
    write_stat_line(6, "free calls", stats.frees as u64);
    write_stat_line(7, "failed allocs", stats.failed_allocations as u64);
    write_memory_bar(8);
}

fn show_ps() {
    clear_output();
    write_output(0, "kernel tasks:");
    task::dump_to_vga(OUTPUT_START_ROW + 1, OUTPUT_ROWS - 1);
}

fn show_sched() {
    clear_output();
    write_output(0, "round-robin scheduler:");
    write_stat_line(1, "task count", task::count() as u64);
    write_stat_line(2, "current pid", task::current_pid() as u64);
    write_stat_line(3, "switches", task::switches() as u64);
    write_stat_line(4, "interval ticks", task::interval_ticks());
    write_output(5, "model: observable kernel-task rotation");
    write_output(6, "note: context switch is simulated");
    write_output(7, "trace kind: SCHED, klog type: SCHED");
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

fn write_stat_line(offset: usize, label: &str, value: u64) {
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, label.as_bytes());
    len = copy_bytes(&mut line, len, b": ");
    len = append_u64(&mut line, len, value);
    write_output(offset, as_str(&line[..len]));
}

fn write_stat_pair(offset: usize, label: &str, used: u64, total: u64) {
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, label.as_bytes());
    len = copy_bytes(&mut line, len, b": ");
    len = append_u64(&mut line, len, used);
    len = copy_bytes(&mut line, len, b"/");
    len = append_u64(&mut line, len, total);
    write_output(offset, as_str(&line[..len]));
}

fn write_memory_bar(offset: usize) {
    let stats = mem::stats();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"pages: [");
    let cells = 32usize;
    let mut i = 0;
    while i < cells {
        let page = i * stats.total_pages / cells;
        let byte = if mem::is_used(page) { b'#' } else { b'.' };
        len = copy_byte(&mut line, len, byte);
        i += 1;
    }
    len = copy_bytes(&mut line, len, b"]");
    write_output(offset, as_str(&line[..len]));
}

fn copy_byte(dst: &mut [u8], idx: usize, byte: u8) -> usize {
    if idx >= dst.len() {
        return idx;
    }
    dst[idx] = byte;
    idx + 1
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
