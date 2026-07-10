use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{fs, keyboard, klog, mem, pit, serial, syscall, task, trace, vga};

const OUTPUT_START_ROW: usize = 15;
const OUTPUT_ROWS: usize = 9;
static TOUR_STEP: AtomicUsize = AtomicUsize::new(0);

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
        "trace on" => {
            show_trace_control(true);
            return;
        }
        "trace off" => {
            show_trace_control(false);
            return;
        }
        "trace status" => {
            show_trace_status();
            return;
        }
        _ => {}
    }

    if let Some(filter) = command_arg(command, "trace") {
        show_trace_filtered(filter);
        return;
    }
    if let Some(topic) = command_arg(command, "bench") {
        show_bench(topic);
        return;
    }
    if let Some(topic) = command_arg(command, "explain") {
        show_explain(topic);
        return;
    }
    if let Some(topic) = command_arg(command, "tour") {
        show_tour(topic);
        return;
    }
    if let Some(topic) = command_arg(command, "demo") {
        show_demo(topic);
        return;
    }
    if let Some(name) = command_arg(command, "cat") {
        show_cat(name);
        return;
    }
    if let Some(text) = command_arg(command, "echo") {
        show_echo(text);
        return;
    }
    if let Some(name) = command_arg(command, "rm") {
        show_rm(name);
        return;
    }

    match command {
        "" => {}
        "help" => show_help(),
        "ticks" => show_ticks(),
        "sysstat" => show_sysstat(),
        "mem" => show_mem(),
        "viz" => show_viz(),
        "ls" => show_ls(),
        "ps" => show_ps(),
        "sched" => show_sched(),
        "syscall" => show_syscall("demo"),
        "syscall write" => show_syscall("write"),
        "syscall time" => show_syscall("time"),
        "syscall trace" => show_syscall("trace"),
        "syscall stats" => show_syscall("stats"),
        "klog" => show_klog(),
        "trace" => show_trace(),
        "bench" => show_bench("overview"),
        "timeline" => show_timeline(),
        "explain" => show_explain("overview"),
        "tour" => show_tour("overview"),
        "demo" => show_demo("overview"),
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
    write_output(5, "  viz    - visual kernel dashboard");
    write_output(6, "  ps sched - task and scheduler state");
    write_output(7, "  ls cat echo rm");
    write_output(8, "  syscall timeline explain tour bench");
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
    write_stat_pair(8, "skipped/syscalls", trace::skipped() as u64, syscall::calls());
}

fn show_trace() {
    clear_output();
    write_output(0, "recent trace events:");
    trace::dump_to_vga(OUTPUT_START_ROW + 1, OUTPUT_ROWS - 1);
}

fn show_trace_filtered(filter: &str) {
    clear_output();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"trace filter: ");
    len = copy_bytes(&mut line, len, filter.as_bytes());
    write_output(0, as_str(&line[..len]));
    trace::dump_filtered_to_vga(OUTPUT_START_ROW + 1, OUTPUT_ROWS - 1, Some(filter));
}

fn show_trace_control(enabled: bool) {
    trace::set_enabled(enabled);
    clear_output();
    if enabled {
        trace::record(trace::TraceKind::Demo, 1, "trace-on");
        write_output(0, "trace recording: on");
    } else {
        write_output(0, "trace recording: off");
    }
    write_stat_line(1, "trace events", trace::len() as u64);
    write_stat_line(2, "skipped events", trace::skipped() as u64);
    write_output(3, "try: trace status");
}

fn show_trace_status() {
    clear_output();
    if trace::is_enabled() {
        write_output(0, "trace recording: on");
    } else {
        write_output(0, "trace recording: off");
    }
    write_stat_pair(1, "trace events", trace::len() as u64, trace::capacity() as u64);
    write_stat_line(2, "skipped events", trace::skipped() as u64);
    write_output(3, "commands: trace on, trace off");
}

fn show_timeline() {
    clear_output();
    write_output(0, "kernel event timeline:");
    trace::dump_to_vga(OUTPUT_START_ROW + 1, OUTPUT_ROWS - 1);
}

fn show_bench(topic: &str) {
    clear_output();
    match topic {
        "trace" => {
            let skipped_before = trace::skipped();
            trace::set_enabled(true);
            let mut i = 0;
            while i < 16 {
                trace::record(trace::TraceKind::Demo, i, "bench-on");
                i += 1;
            }

            trace::set_enabled(false);
            let mut j = 0;
            while j < 16 {
                trace::record(trace::TraceKind::Demo, j, "bench-off");
                j += 1;
            }
            let skipped_after = trace::skipped();
            trace::set_enabled(true);
            trace::record(trace::TraceKind::Demo, 16, "bench-done");

            write_output(0, "bench trace:");
            write_stat_line(1, "enabled attempts", 16);
            write_stat_line(2, "disabled attempts", 16);
            write_stat_line(3, "new skipped events", (skipped_after - skipped_before) as u64);
            write_stat_pair(4, "trace events", trace::len() as u64, trace::capacity() as u64);
            write_output(5, "meaning: disabled tracing drops events");
            write_output(6, "observe: trace demo, trace status");
        }
        _ => {
            write_output(0, "bench topics:");
            write_output(1, "  bench trace");
            write_output(2, "shows trace recording vs skipped events");
            write_output(3, "useful for discussing observability cost");
        }
    }
}

fn show_explain(topic: &str) {
    clear_output();
    match topic {
        "irq" | "interrupt" => {
            write_output(0, "IRQ path:");
            write_output(1, "hardware -> IDT gate -> rust_irq_handler");
            write_output(2, "timer irq0 increments PIT ticks");
            write_output(3, "keyboard irq1 feeds shell input");
            write_output(4, "each handled IRQ sends PIC EOI");
            write_output(5, "observe: ticks, trace irq, sysstat");
        }
        "sched" | "schedule" => {
            write_output(0, "Scheduler model:");
            write_output(1, "PIT tick drives a round-robin task table");
            write_output(2, "every 25 ticks current task rotates");
            write_output(3, "task switch events enter trace and klog");
            write_output(4, "observe: ps, sched, trace sched");
        }
        "mem" | "memory" => {
            write_output(0, "Memory model:");
            write_output(1, "Nagi uses a 4 KiB physical page pool");
            write_output(2, "early pages are reserved for the kernel");
            write_output(3, "tasks allocate stack pages from the pool");
            write_output(4, "observe: mem, sysstat, trace mem");
        }
        "syscall" | "sys" => {
            write_output(0, "Syscall model:");
            write_output(1, "user intent is routed through syscall table");
            write_output(2, "write/time/trace/stats are implemented");
            write_output(3, "each call records trace and klog entries");
            write_output(4, "observe: syscall, trace syscall");
        }
        _ => {
            write_output(0, "explain topics:");
            write_output(1, "  explain irq");
            write_output(2, "  explain sched");
            write_output(3, "  explain mem");
            write_output(4, "  explain syscall");
            write_output(5, "goal: make kernel internals teachable");
        }
    }
}

fn show_tour(topic: &str) {
    let topic = if topic == "next" {
        let step = (TOUR_STEP.fetch_add(1, Ordering::Relaxed) + 1) % 7;
        tour_topic(step)
    } else {
        topic
    };

    clear_output();
    trace::record(trace::TraceKind::Demo, topic.len() as u64, "tour");
    match topic {
        "boot" => {
            TOUR_STEP.store(0, Ordering::Relaxed);
            write_output(0, "tour 1/7: boot path");
            write_output(1, "stage1 loads stage2 from the disk image");
            write_output(2, "stage2 enters protected mode and long mode");
            write_output(3, "Rust no_std kernel starts at 0x10000");
            write_output(4, "observe: klog, trace boot, explain irq");
        }
        "mem" | "memory" => {
            TOUR_STEP.store(1, Ordering::Relaxed);
            write_output(0, "tour 2/7: memory");
            write_output(1, "4 KiB pages form a visible physical pool");
            write_output(2, "kernel/task/RAMFS allocations consume pages");
            write_output(3, "observe: mem, viz, trace mem");
        }
        "sched" | "scheduler" => {
            TOUR_STEP.store(2, Ordering::Relaxed);
            write_output(0, "tour 3/7: scheduler");
            write_output(1, "PIT ticks drive round-robin task rotation");
            write_output(2, "task switches are recorded as events");
            write_output(3, "observe: ps, sched, trace sched");
        }
        "syscall" | "sys" => {
            TOUR_STEP.store(3, Ordering::Relaxed);
            write_output(0, "tour 4/7: syscall");
            write_output(1, "sys_write/sys_time/sys_trace/sys_stats");
            write_output(2, "each syscall records trace and klog");
            write_output(3, "observe: syscall, demo syscall");
        }
        "fs" | "file" => {
            TOUR_STEP.store(4, Ordering::Relaxed);
            write_output(0, "tour 5/7: RAMFS");
            write_output(1, "single-directory memory filesystem");
            write_output(2, "files occupy visible page-pool entries");
            write_output(3, "observe: ls, cat readme, echo hi > note");
        }
        "observe" | "obs" => {
            TOUR_STEP.store(5, Ordering::Relaxed);
            write_output(0, "tour 6/7: observability");
            write_output(1, "klog records kernel events");
            write_output(2, "trace filters events by subsystem");
            write_output(3, "timeline turns events into a story");
            write_output(4, "observe: viz, timeline, trace irq");
        }
        "demo" => {
            TOUR_STEP.store(6, Ordering::Relaxed);
            write_output(0, "tour 7/7: presentation flow");
            write_output(1, "run: viz -> mem -> ps -> syscall");
            write_output(2, "run: ls -> demo fs -> timeline");
            write_output(3, "close with: explain sched");
            write_output(4, "use: tour next to cycle again");
        }
        _ => {
            write_output(0, "tour topics:");
            write_output(1, "  tour boot");
            write_output(2, "  tour mem");
            write_output(3, "  tour sched");
            write_output(4, "  tour syscall");
            write_output(5, "  tour fs");
            write_output(6, "  tour observe");
            write_output(7, "  tour demo");
            write_output(8, "  tour next");
        }
    }
}

fn tour_topic(step: usize) -> &'static str {
    match step {
        0 => "boot",
        1 => "mem",
        2 => "sched",
        3 => "syscall",
        4 => "fs",
        5 => "observe",
        _ => "demo",
    }
}

fn show_demo(topic: &str) {
    clear_output();
    trace::record(trace::TraceKind::Demo, topic.len() as u64, topic);
    match topic {
        "timer" => {
            write_output(0, "demo timer:");
            write_stat_line(1, "current PIT ticks", pit::ticks());
            write_stat_line(2, "PIT frequency Hz", pit::CONFIGURED_FREQUENCY_HZ as u64);
            write_output(3, "wait one second, then run ticks");
            write_output(4, "observe: trace timer, sysstat");
        }
        "keyboard" | "kbd" => {
            write_output(0, "demo keyboard:");
            write_stat_line(1, "keyboard IRQ count", keyboard::irq_count());
            write_output(2, "type any command and press Enter");
            write_output(3, "observe: trace keyboard");
        }
        "sched" | "schedule" => {
            write_output(0, "demo scheduler:");
            write_stat_line(1, "current pid", task::current_pid() as u64);
            write_stat_line(2, "switches", task::switches() as u64);
            write_output(3, "wait, then run ps or sched again");
            write_output(4, "observe: trace sched");
        }
        "mem" | "memory" => {
            let page = mem::alloc_page("demo");
            if let Some(page) = page {
                let _ = mem::free_page(page, "demo-free");
                write_output(0, "demo memory:");
                write_stat_line(1, "allocated and freed page", page as u64);
                write_output(2, "observe: mem, trace mem");
            } else {
                write_output(0, "demo memory: allocation failed");
            }
        }
        "fs" | "file" => {
            let _ = fs::create_or_write("demo", "RAMFS demo file");
            write_output(0, "demo filesystem:");
            write_output(1, "created file: demo");
            write_output(2, "try: ls");
            write_output(3, "try: cat demo");
            write_output(4, "observe: trace file");
        }
        "syscall" | "sys" => {
            let ret = syscall::invoke(syscall::SYS_TIME, 0, "demo-time");
            write_output(0, "demo syscall:");
            write_stat_line(1, "sys_time return", ret);
            write_stat_line(2, "last syscall", syscall::last_number());
            write_stat_line(3, "last return", syscall::last_return());
            write_output(4, "observe: trace syscall");
        }
        "trace" => {
            trace::record(trace::TraceKind::Demo, 1, "trace-demo");
            write_output(0, "demo trace:");
            write_output(1, "emitted DEMO trace event");
            write_output(2, "try: trace demo");
            write_output(3, "try: timeline");
        }
        _ => {
            write_output(0, "demo topics:");
            write_output(1, "  demo timer");
            write_output(2, "  demo keyboard");
            write_output(3, "  demo sched");
            write_output(4, "  demo mem");
            write_output(5, "  demo fs");
            write_output(6, "  demo syscall");
            write_output(7, "  demo trace");
        }
    }
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

fn show_viz() {
    clear_output();
    let memory = mem::stats();
    write_output(0, "observable kernel dashboard:");
    write_bar(1, "mem", memory.used_pages as u64, memory.total_pages as u64);
    write_bar(2, "trace", trace::len() as u64, trace::capacity() as u64);
    write_bar(3, "klog", klog::len() as u64, klog::capacity() as u64);
    write_bar(4, "irq", keyboard::irq_count(), 32);
    write_bar(5, "sched", task::switches() as u64, 32);
    write_stat_line(6, "pit ticks", pit::ticks());
    write_stat_line(7, "syscall calls", syscall::calls());
    write_stat_line(8, "ramfs files", fs::count() as u64);
}

fn show_ls() {
    clear_output();
    write_output(0, "RAMFS files:");
    fs::list_to_vga(OUTPUT_START_ROW + 1, OUTPUT_ROWS - 1);
}

fn show_cat(name: &str) {
    clear_output();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"cat ");
    len = copy_bytes(&mut line, len, name.as_bytes());
    write_output(0, as_str(&line[..len]));
    if !fs::cat_to_vga(name, OUTPUT_START_ROW + 1) {
        write_output(1, "file not found");
    }
}

fn show_echo(text: &str) {
    clear_output();
    let content = strip_note_redirect(text);
    if fs::create_or_write("note", content) {
        write_output(0, "wrote RAMFS file: note");
        write_output(1, "try: cat note");
    } else {
        write_output(0, "RAMFS write failed");
    }
}

fn show_rm(name: &str) {
    clear_output();
    if fs::remove(name) {
        write_output(0, "removed RAMFS file");
    } else {
        write_output(0, "file not found");
    }
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

fn show_syscall(mode: &str) {
    clear_output();
    write_output(0, "syscall table:");
    syscall::dump_table_to_vga(OUTPUT_START_ROW + 1, 4);

    if mode == "demo" || mode == "write" {
        let ret = syscall::invoke(syscall::SYS_WRITE, 14, "hello-syscall");
        write_stat_line(5, "sys_write return", ret);
    }
    if mode == "demo" || mode == "time" {
        let ret = syscall::invoke(syscall::SYS_TIME, 0, "time");
        write_stat_line(6, "sys_time ticks", ret);
    }
    if mode == "demo" || mode == "trace" {
        let ret = syscall::invoke(syscall::SYS_TRACE, 7, "trace");
        write_stat_line(7, "sys_trace return", ret);
    }
    if mode == "demo" || mode == "stats" {
        let ret = syscall::invoke(syscall::SYS_STATS, 0, "stats");
        write_stat_line(8, "sys_stats calls", ret);
    }
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

fn write_bar(offset: usize, label: &str, value: u64, max: u64) {
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, label.as_bytes());
    len = copy_bytes(&mut line, len, b": [");
    let cells = 24u64;
    let capped = if value > max { max } else { value };
    let filled = if max == 0 { 0 } else { capped * cells / max };
    let mut i = 0;
    while i < cells {
        let byte = if i < filled { b'#' } else { b'.' };
        len = copy_byte(&mut line, len, byte);
        i += 1;
    }
    len = copy_bytes(&mut line, len, b"] ");
    len = append_u64(&mut line, len, value);
    len = copy_bytes(&mut line, len, b"/");
    len = append_u64(&mut line, len, max);
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

fn command_arg<'a>(command: &'a str, name: &str) -> Option<&'a str> {
    let bytes = command.as_bytes();
    let name_bytes = name.as_bytes();
    if bytes.len() <= name_bytes.len() {
        return None;
    }
    let mut i = 0;
    while i < name_bytes.len() {
        if bytes[i] != name_bytes[i] {
            return None;
        }
        i += 1;
    }
    if bytes[name_bytes.len()] != b' ' {
        return None;
    }
    let arg = unsafe { core::str::from_utf8_unchecked(&bytes[name_bytes.len() + 1..]) };
    Some(trim(arg))
}

fn strip_note_redirect(text: &str) -> &str {
    let bytes = text.as_bytes();
    if bytes.len() < 7 {
        return text;
    }

    let suffix = b" > note";
    if bytes.len() < suffix.len() {
        return text;
    }
    let start = bytes.len() - suffix.len();
    let mut i = 0;
    while i < suffix.len() {
        if bytes[start + i] != suffix[i] {
            return text;
        }
        i += 1;
    }

    trim(unsafe { core::str::from_utf8_unchecked(&bytes[..start]) })
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
