use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::{fs, keyboard, klog, mem, pit, serial, syscall, task, trace, ui, user, vga};

const OUTPUT_START_ROW: usize = ui::OUTPUT_START_ROW;
const OUTPUT_ROWS: usize = ui::OUTPUT_ROWS;
const CONTENT_TEXT_COL: usize = ui::CONTENT_COL + 2;
const COMMANDS: &[&str] = &[
    "h",
    "s",
    "v",
    "m",
    "p",
    "f",
    "t",
    "g",
    "n",
    "r",
    "d",
    "b",
    "q",
    "help",
    "help obs",
    "help fs",
    "help demo",
    "?",
    "status",
    "sysstat",
    "ticks",
    "viz",
    "watch",
    "watch off",
    "mem",
    "ps",
    "sched",
    "syscall",
    "syscall write",
    "syscall time",
    "syscall trace",
    "syscall stats",
    "run",
    "run hello",
    "run time",
    "run trace",
    "run files",
    "programs",
    "files",
    "ls",
    "cat readme",
    "cat note",
    "cat demo",
    "cat userlog",
    "echo hello > note",
    "rm note",
    "trace",
    "trace irq",
    "trace sched",
    "trace mem",
    "trace syscall",
    "trace file",
    "trace demo",
    "trace on",
    "trace off",
    "trace status",
    "timeline",
    "replay",
    "flow",
    "flow irq",
    "flow syscall",
    "flow file",
    "why",
    "why irq",
    "why mem",
    "why sched",
    "why syscall",
    "why file",
    "explain",
    "explain irq",
    "explain sched",
    "explain mem",
    "explain syscall",
    "tour",
    "tour next",
    "tour boot",
    "tour mem",
    "tour sched",
    "tour syscall",
    "tour fs",
    "tour observe",
    "tour demo",
    "guide",
    "demo",
    "demo sched",
    "demo fs",
    "demo syscall",
    "demo trace",
    "bench trace",
    "colors",
    "logo",
    "present",
    "present cover",
    "present boot",
    "present irq",
    "present mem",
    "present sched",
    "present syscall",
    "present fs",
    "present observe",
    "present summary",
    "clear",
    "cls",
];
const PAGE_HOME: usize = 0;
const PAGE_HELP: usize = 1;
const PAGE_STATUS: usize = 2;
const PAGE_MEMORY: usize = 3;
const PAGE_TASKS: usize = 4;
const PAGE_FILES: usize = 5;
const PAGE_TRACE: usize = 6;
const PAGE_RUN: usize = 7;
const PAGE_TOUR: usize = 8;
const PAGE_DEMO: usize = 9;
const PAGE_BENCH: usize = 10;
const PAGE_EXPLAIN: usize = 11;
const PAGE_DIAG: usize = 12;
const PAGE_SHELL: usize = 13;
const PAGE_LOGO: usize = 14;
const PAGE_PRESENT: usize = 15;

static CURRENT_PAGE: AtomicUsize = AtomicUsize::new(PAGE_HOME);
static TOUR_STEP: AtomicUsize = AtomicUsize::new(0);
static PRESENT_STEP: AtomicUsize = AtomicUsize::new(0);
static PRESENT_ACTIVE: AtomicBool = AtomicBool::new(false);
static WATCH_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn init() {
    set_page(PAGE_HOME);
    clear_output();
    write_output(0, "Type a letter. Tab or Right accepts the ghost text.");
    write_output(2, "Try h, s, m, p, t, g, r, f, d.");
    write_output(4, "The left panel updates with matches and the current page.");
}

pub fn complete(input: &str) -> Option<&'static str> {
    let prefix = trim(input);
    if prefix.is_empty() {
        return None;
    }

    let mut i = 0;
    while i < COMMANDS.len() {
        let command = COMMANDS[i];
        if command != prefix && starts_with(command, prefix) {
            return Some(command);
        }
        i += 1;
    }
    None
}

pub fn sidebar_matches(input: &str, out: &mut [&'static str]) -> usize {
    let prefix = trim(input);
    if prefix.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut i = 0;
    while i < COMMANDS.len() && count < out.len() {
        let command = COMMANDS[i];
        if command != prefix && starts_with(command, prefix) {
            out[count] = command;
            count += 1;
        }
        i += 1;
    }
    count
}

pub fn current_page() -> &'static str {
    match CURRENT_PAGE.load(Ordering::Relaxed) {
        PAGE_HELP => "help",
        PAGE_STATUS => "status",
        PAGE_MEMORY => "memory",
        PAGE_TASKS => "tasks",
        PAGE_FILES => "files",
        PAGE_TRACE => "trace",
        PAGE_RUN => "run",
        PAGE_TOUR => "tour",
        PAGE_DEMO => "demo",
        PAGE_BENCH => "bench",
        PAGE_EXPLAIN => "explain",
        PAGE_DIAG => "diag",
        PAGE_SHELL => "shell",
        PAGE_LOGO => "logo",
        PAGE_PRESENT => "present",
        _ => "welcome",
    }
}

pub fn run(command: &str) {
    let command = trim(command);
    if command != "watch" && command != "watch off" {
        WATCH_ACTIVE.store(false, Ordering::Relaxed);
    }
    if presentation_active() && command == "n" {
        show_present("next");
        return;
    }
    if presentation_active() && command == "b" {
        show_present("prev");
        return;
    }
    if !starts_with(command, "present") {
        PRESENT_ACTIVE.store(false, Ordering::Relaxed);
    }
    set_page(page_for_command(command));
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
    if let Some(topic) = command_arg(command, "help") {
        show_help_topic(topic);
        return;
    }
    if let Some(name) = command_arg(command, "run") {
        show_run(name);
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
    if let Some(topic) = command_arg(command, "flow") {
        show_flow(topic);
        return;
    }
    if let Some(topic) = command_arg(command, "why") {
        show_why(topic);
        return;
    }
    if let Some(topic) = command_arg(command, "present") {
        show_present(topic);
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
        "help" | "?" | "h" => show_help(),
        "ticks" => show_ticks(),
        "sysstat" | "status" | "s" => show_sysstat(),
        "mem" | "m" => show_mem(),
        "viz" | "v" => show_viz(),
        "watch" => show_watch(),
        "watch off" => stop_watch(),
        "ls" | "files" | "f" => show_ls(),
        "ps" | "p" => show_ps(),
        "sched" => show_sched(),
        "syscall" => show_syscall("demo"),
        "syscall write" => show_syscall("write"),
        "syscall time" => show_syscall("time"),
        "syscall trace" => show_syscall("trace"),
        "syscall stats" => show_syscall("stats"),
        "klog" => show_klog(),
        "trace" => show_trace(),
        "bench" => show_bench("overview"),
        "bench trace" | "b" => show_bench("trace"),
        "colors" => show_colors(),
        "logo" => show_logo(),
        "present" => show_present("cover"),
        "run" | "programs" | "r" => show_run("overview"),
        "timeline" | "t" => show_timeline(),
        "replay" => show_replay(),
        "flow" => show_flow("overview"),
        "why" => show_why("overview"),
        "explain" => show_explain("overview"),
        "tour" | "guide" | "g" => show_tour("overview"),
        "tour next" | "n" => show_tour("next"),
        "demo" | "d" => show_demo("overview"),
        "clear" | "cls" | "q" => {
            set_page(PAGE_SHELL);
            clear_output();
        }
        _ => show_unknown(command),
    }
}

pub fn watch_tick(tick: u64) {
    if WATCH_ACTIVE.load(Ordering::Relaxed) && tick % 25 == 0 {
        render_watch(tick);
    }
}

fn show_watch() {
    PRESENT_ACTIVE.store(false, Ordering::Relaxed);
    WATCH_ACTIVE.store(true, Ordering::Relaxed);
    set_page(PAGE_STATUS);
    render_watch(pit::ticks());
}

fn stop_watch() {
    WATCH_ACTIVE.store(false, Ordering::Relaxed);
    set_page(PAGE_STATUS);
    clear_page("LIVE WATCH");
    ui::draw_badge(1, "PAUSED", "live dashboard stopped");
    ui::draw_next("watch to resume / s status");
}

fn render_watch(ticks: u64) {
    ui::clear_output("LIVE WATCH");
    ui::draw_activity(0, ticks, "kernel activity");
    ui::draw_metric(2, 0, "uptime sec", ticks / pit::CONFIGURED_FREQUENCY_HZ as u64);
    ui::draw_metric(2, 1, "timer IRQ", ticks);
    ui::draw_metric(3, 0, "keyboard IRQ", keyboard::irq_count());
    ui::draw_metric(3, 1, "task switches", task::switches() as u64);
    let memory = mem::stats();
    ui::draw_metric(5, 0, "memory used", memory.used_pages as u64);
    ui::draw_metric(5, 1, "trace events", trace::len() as u64);
    ui::draw_metric(6, 0, "files", fs::count() as u64);
    ui::draw_metric(6, 1, "syscalls", syscall::calls());
    ui::draw_progress(7, "memory", memory.used_pages, memory.total_pages);
    ui::draw_next("type any command to leave live mode");
    ui::draw_footer_path("watch", "live 4 Hz");
}

pub fn presentation_active() -> bool {
    PRESENT_ACTIVE.load(Ordering::Relaxed)
}

pub fn presentation_navigate(forward: bool) {
    if presentation_active() {
        show_present(if forward { "next" } else { "prev" });
    }
}

fn show_present(topic: &str) {
    const PAGE_COUNT: usize = 9;
    let step = match topic {
        "next" => core::cmp::min(PRESENT_STEP.load(Ordering::Relaxed) + 1, PAGE_COUNT - 1),
        "prev" | "back" => PRESENT_STEP.load(Ordering::Relaxed).saturating_sub(1),
        "cover" => 0,
        "boot" => 1,
        "irq" => 2,
        "mem" | "memory" => 3,
        "sched" | "scheduler" => 4,
        "syscall" | "sys" => 5,
        "fs" | "file" => 6,
        "observe" | "obs" => 7,
        "summary" => 8,
        _ => 0,
    };
    PRESENT_STEP.store(step, Ordering::Relaxed);
    PRESENT_ACTIVE.store(true, Ordering::Relaxed);
    set_page(PAGE_PRESENT);
    clear_output();
    trace::record(trace::TraceKind::Demo, step as u64, "present");

    match step {
        0 => present_page("NAGI OS", "Rust x86_64 kernel from boot sector to shell",
            "run present and follow a live kernel story",
            "calm interface makes invisible kernel motion visible"),
        1 => present_page("BOOT PATH", "stage1 + stage2 enter 64-bit long mode",
            "klog / trace boot",
            "hand-built boot chain reaches a no_std Rust kernel"),
        2 => present_page("INTERRUPTS", "IDT, PIC, PIT 100 Hz, keyboard IRQ1",
            "status / timeline / flow irq",
            "hardware events become readable evidence"),
        3 => present_page("MEMORY", "4 KiB page pool with ownership tracking",
            "mem / mem map / mem demo",
            "allocation state is visual, inspectable, reproducible"),
        4 => present_page("SCHEDULER", "round-robin task model and state transitions",
            "ps / sched / trace sched",
            "runtime behavior is taught through live state"),
        5 => present_page("SYSCALL", "write, time, trace, stats dispatch table",
            "syscall / flow syscall / syscall stats",
            "user intent is traceable across the kernel boundary"),
        6 => present_page("RAMFS", "create, read, update, remove memory files",
            "files / cat readme / flow file",
            "file metadata links storage to physical pages"),
        7 => present_page("OBSERVABILITY", "trace, timeline, watch, replay, why",
            "watch / timeline / replay",
            "the OS explains itself from its own runtime data"),
        _ => present_page("SUMMARY", "boot + IRQ + memory + tasks + syscall + FS",
            "demo / present cover",
            "a cohesive teaching OS, not disconnected demos"),
    }

    let mut progress = [0u8; 40];
    let mut len = copy_bytes(&mut progress, 0, b"page ");
    len = append_u64(&mut progress, len, (step + 1) as u64);
    len = copy_bytes(&mut progress, len, b"/9  ");
    if step > 0 {
        len = copy_bytes(&mut progress, len, b"b/Left back  ");
    }
    if step + 1 < PAGE_COUNT {
        len = copy_bytes(&mut progress, len, b"n/Right next");
    } else {
        len = copy_bytes(&mut progress, len, b"present cover");
    }
    write_output(8, as_str(&progress[..len]));
    ui::draw_footer("defense / present");
}

fn present_page(title: &str, implemented: &str, observe: &str, innovation: &str) {
    write_output(0, title);
    write_key_line(2, "OK", implemented);
    write_key_line(4, "SEE", observe);
    write_key_line(6, "NEW", innovation);
}

fn show_help() {
    set_page(PAGE_HELP);
    clear_page("COMMAND INDEX");
    ui::draw_badge(0, "CORE", "s status   m memory   p tasks");
    ui::draw_badge(2, "OBSERVE", "v watch   t timeline   trace");
    ui::draw_badge(4, "STORAGE", "f files   cat   echo   rm");
    ui::draw_badge(6, "DEFENSE", "present   tour   demo   bench");
    ui::draw_next("help obs / help fs / help demo");
}

fn show_colors() {
    set_page(PAGE_DIAG);
    clear_output();
    write_output(0, "VGA palette check:");

    write_color_sample(1, 2, "0 black", vga::Color::Black);
    write_color_sample(1, 22, "1 blue", vga::Color::Blue);
    write_color_sample(1, 42, "2 green", vga::Color::Green);
    write_color_sample(1, 62, "3 cyan", vga::Color::Cyan);

    write_color_sample(3, 2, "4 red", vga::Color::Red);
    write_color_sample(3, 22, "5 magenta", vga::Color::Magenta);
    write_color_sample(3, 42, "6 brown", vga::Color::Brown);
    write_color_sample(3, 62, "7 lightgray", vga::Color::LightGray);

    write_color_sample(5, 2, "8 darkgray", vga::Color::DarkGray);
    write_color_sample(5, 22, "9 lightblue", vga::Color::LightBlue);
    write_color_sample(5, 42, "10 lightgreen", vga::Color::LightGreen);
    write_color_sample(5, 62, "11 lightcyan", vga::Color::LightCyan);

    write_color_sample(7, 2, "12 lightred", vga::Color::LightRed);
    write_color_sample(7, 22, "13 pink", vga::Color::Pink);
    write_color_sample(7, 42, "14 yellow", vga::Color::Yellow);
    write_color_sample(7, 62, "15 white", vga::Color::White);

    write_output(8, "If gray looks wrong in curses, run with -Display vnc.");
}

fn show_logo() {
    set_page(PAGE_LOGO);
    clear_output();
    write_output(0, "Nagi motion mark");
    ui::draw_logo_card();
    write_output(7, "next: s status, g guide, d demo");
}

fn show_help_topic(topic: &str) {
    set_page(PAGE_HELP);
    clear_output();
    match topic {
        "obs" | "observe" => {
            write_output(0, "observability commands:");
            write_output(1, "  viz             dashboard");
            write_output(2, "  timeline        chronological events");
            write_output(3, "  trace irq       interrupt events");
            write_output(4, "  trace sched     scheduler events");
            write_output(5, "  trace mem       allocator events");
            write_output(6, "  trace syscall   syscall events");
            write_output(7, "  trace on/off/status");
            write_output(8, "  bench trace     tracing overhead demo");
        }
        "fs" | "file" => {
            write_output(0, "RAMFS commands:");
            write_output(1, "  ls or files     list files");
            write_output(2, "  cat readme      read a file");
            write_output(3, "  echo hi > note  write note");
            write_output(4, "  cat note        read note");
            write_output(5, "  rm note         remove note");
            write_output(6, "  demo fs         create demo file");
            write_output(7, "  run files       user program writes file");
        }
        "demo" | "tour" => {
            write_output(0, "presentation commands:");
            write_output(1, "  tour            list tour pages");
            write_output(2, "  tour next       advance tour");
            write_output(3, "  demo            list demos");
            write_output(4, "  demo syscall    syscall story");
            write_output(5, "  demo fs         RAMFS story");
            write_output(6, "  run hello       user program story");
            write_output(7, "  explain sched   teaching note");
        }
        _ => show_help(),
    }
}

fn show_ticks() {
    set_page(PAGE_STATUS);
    clear_output();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"pit ticks: ");
    len = append_u64(&mut line, len, pit::ticks());
    write_output(0, as_str(&line[..len]));
}

fn show_klog() {
    set_page(PAGE_TRACE);
    clear_output();
    write_output(0, "early kernel log:");
    klog::dump_to_vga(OUTPUT_START_ROW + 1, CONTENT_TEXT_COL, OUTPUT_ROWS - 1);
}

fn show_sysstat() {
    set_page(PAGE_STATUS);
    clear_page("SYSTEM STATUS");
    let ticks = pit::ticks();
    ui::draw_badge(0, "ONLINE", "interrupts and shell are active");
    write_stat_pair(2, "uptime / ticks", ticks / pit::CONFIGURED_FREQUENCY_HZ as u64, ticks);
    write_stat_pair(3, "timer / keyboard IRQ", ticks, keyboard::irq_count());
    write_stat_pair(4, "klog / trace events", klog::len() as u64, trace::len() as u64);
    let memory = mem::stats();
    ui::draw_metric(5, 0, "memory used", memory.used_pages as u64);
    ui::draw_metric(5, 1, "total pages", memory.total_pages as u64);
    ui::draw_metric(6, 0, "tasks", task::count() as u64);
    ui::draw_metric(6, 1, "switches", task::switches() as u64);
    ui::draw_metric(7, 0, "files", fs::count() as u64);
    ui::draw_metric(7, 1, "syscalls", syscall::calls());
    ui::draw_next("watch for live data / m memory / p tasks");
}

fn show_trace() {
    set_page(PAGE_TRACE);
    clear_page("KERNEL TRACE");
    ui::draw_badge(0, "RECENT", "time-ordered kernel events");
    trace::dump_to_vga(OUTPUT_START_ROW + 2, CONTENT_TEXT_COL, OUTPUT_ROWS - 3);
    ui::draw_next("timeline / trace irq / trace sched");
}

fn show_trace_filtered(filter: &str) {
    set_page(PAGE_TRACE);
    clear_output();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"trace filter: ");
    len = copy_bytes(&mut line, len, filter.as_bytes());
    write_output(0, as_str(&line[..len]));
    trace::dump_filtered_to_vga(OUTPUT_START_ROW + 1, CONTENT_TEXT_COL, OUTPUT_ROWS - 2, Some(filter));
    ui::draw_next("trace / timeline / replay");
}

fn show_trace_control(enabled: bool) {
    set_page(PAGE_TRACE);
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
    set_page(PAGE_TRACE);
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
    set_page(PAGE_TRACE);
    clear_page("EVENT TIMELINE");
    if !trace::is_enabled() {
        ui::draw_badge(0, "PAUSED", "showing events captured before trace off");
    } else if trace::len() == 0 {
        ui::draw_badge(0, "EMPTY", "run a command to create the first event");
    } else {
        ui::draw_badge(0, "LATEST", "recent events in chronological order");
    }
    ui::draw_table_header(1, "TICK   | KIND  | EVENT            | VALUE");
    trace::dump_timeline_to_vga(OUTPUT_START_ROW + 2, CONTENT_TEXT_COL, OUTPUT_ROWS - 3);
    ui::draw_next("replay / trace irq / flow syscall");
}

fn show_replay() {
    set_page(PAGE_TRACE);
    clear_page("TRACE REPLAY");
    if trace::len() == 0 {
        ui::draw_badge(0, "EMPTY", "nothing has been recorded yet");
    } else {
        ui::draw_badge(0, "STORY", "oldest to newest, recent window");
        trace::dump_replay_to_vga(OUTPUT_START_ROW + 2, CONTENT_TEXT_COL, OUTPUT_ROWS - 3);
    }
    ui::draw_next("timeline for raw values / why for causes");
}

fn show_flow(topic: &str) {
    set_page(PAGE_EXPLAIN);
    clear_page("KERNEL FLOW");
    match topic {
        "irq" => {
            ui::draw_badge(0, "IRQ", "hardware interrupt path");
            write_output(2, "device -> IDT gate -> Rust IRQ handler");
            write_output(4, "handler -> subsystem update -> trace event");
            write_output(6, "handler -> PIC EOI -> interrupted code");
            ui::draw_next("status / trace irq / why irq");
        }
        "syscall" | "sys" => {
            ui::draw_badge(0, "SYSCALL", "user-to-kernel service path");
            write_output(2, "user program -> syscall number + argument");
            write_output(4, "dispatch table -> kernel service -> result");
            write_output(6, "result -> stats + klog + trace -> user");
            ui::draw_next("syscall / run hello / why syscall");
        }
        "file" | "fs" => {
            ui::draw_badge(0, "RAMFS", "file operation path");
            write_output(2, "shell/user -> RAMFS lookup -> file slot");
            write_output(4, "write -> page owner + revision + content");
            write_output(6, "operation -> FILE trace -> visible metadata");
            ui::draw_next("files / demo fs / why file");
        }
        _ => {
            ui::draw_badge(0, "CHOOSE", "inspect a kernel path");
            write_key_line(2, "IRQ", "flow irq");
            write_key_line(4, "SYS", "flow syscall");
            write_key_line(6, "FILE", "flow file");
            ui::draw_next("flow irq / flow syscall / flow file");
        }
    }
}

fn show_why(topic: &str) {
    set_page(PAGE_EXPLAIN);
    clear_page("WHY IS THIS CHANGING?");
    match topic {
        "irq" => {
            ui::draw_badge(0, "CAUSE", "PIT hardware fires 100 times each second");
            write_stat_pair(2, "timer / keyboard IRQ", pit::ticks(), keyboard::irq_count());
            write_output(4, "timer advances tasks, watch, logo, and uptime");
            ui::draw_next("flow irq / trace irq / watch");
        }
        "mem" | "memory" => {
            let stats = mem::stats();
            ui::draw_badge(0, "CAUSE", "kernel, tasks and files own physical pages");
            write_stat_pair(2, "used / free", stats.used_pages as u64, stats.free_pages as u64);
            write_output(4, "create/remove operations change page ownership");
            ui::draw_next("mem map / mem demo / trace mem");
        }
        "sched" | "task" => {
            ui::draw_badge(0, "CAUSE", "PIT reaches the round-robin interval");
            write_stat_pair(2, "current pid / switches", task::current_pid() as u64, task::switches() as u64);
            write_output(4, "running becomes ready; next ready task runs");
            ui::draw_next("ps / sched demo / trace sched");
        }
        "syscall" | "sys" => {
            ui::draw_badge(0, "CAUSE", "user intent crossed a service boundary");
            write_stat_pair(2, "last number / calls", syscall::last_number(), syscall::calls());
            write_output(4, "dispatcher records result, counters and trace");
            ui::draw_next("flow syscall / syscall stats / run hello");
        }
        "file" | "fs" => {
            ui::draw_badge(0, "CAUSE", "RAMFS command changed an in-memory entry");
            write_stat_line(2, "active files", fs::count() as u64);
            write_output(4, "metadata and page ownership change together");
            ui::draw_next("flow file / files / trace file");
        }
        _ => {
            ui::draw_badge(0, "ASK", "connect a metric to its kernel cause");
            write_output(2, "why irq      why mem      why sched");
            write_output(4, "why syscall  why file");
            ui::draw_next("why irq / why mem / why sched");
        }
    }
}

fn show_bench(topic: &str) {
    set_page(PAGE_BENCH);
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
    set_page(PAGE_EXPLAIN);
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
    set_page(PAGE_TOUR);
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
            write_output(3, "observe: syscall, run hello, run time");
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
            write_output(2, "run: run hello -> demo fs -> timeline");
            write_output(3, "close with: bench trace");
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
    set_page(PAGE_DEMO);
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
    set_page(PAGE_MEMORY);
    clear_page("PHYSICAL MEMORY");
    let stats = mem::stats();
    ui::draw_badge(0, "4 KIB", "fixed physical page pool");
    write_stat_pair(2, "used / total pages", stats.used_pages as u64, stats.total_pages as u64);
    write_stat_pair(3, "reserved / free", stats.reserved_pages as u64, stats.free_pages as u64);
    write_stat_pair(4, "alloc / free calls", stats.allocations as u64, stats.frees as u64);
    write_stat_line(5, "failed allocations", stats.failed_allocations as u64);
    ui::draw_progress(6, "utilization", stats.used_pages, stats.total_pages);
    ui::draw_next("mem map / mem demo / trace mem");
}

fn show_viz() {
    set_page(PAGE_STATUS);
    clear_output();
    let memory = mem::stats();
    write_output(0, "overview");
    write_stat_pair(2, "memory pages", memory.used_pages as u64, memory.total_pages as u64);
    write_stat_pair(3, "trace events", trace::len() as u64, trace::capacity() as u64);
    write_stat_pair(4, "tasks / switches", task::count() as u64, task::switches() as u64);
    write_stat_pair(5, "files / syscalls", fs::count() as u64, syscall::calls());
    write_stat_line(6, "keyboard irq", keyboard::irq_count());
    write_stat_line(7, "pit ticks", pit::ticks());
    write_output(8, "next: m memory, p tasks, t timeline");
}

fn show_run(name: &str) {
    set_page(PAGE_RUN);
    clear_output();
    if name == "overview" {
        write_output(0, "user programs:");
        user::list_to_vga(OUTPUT_START_ROW + 1, CONTENT_TEXT_COL, OUTPUT_ROWS - 1);
        return;
    }

    let result = user::run(name);
    write_output(0, "user program result:");
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"program: ");
    len = copy_bytes(&mut line, len, name.as_bytes());
    write_output(1, as_str(&line[..len]));
    write_stat_line(2, "exit code", result.exit_code);
    write_stat_line(3, "return value", result.value);
    write_output(4, result.message);
    write_output(5, "observe: syscall, trace syscall, timeline");
}

fn show_ls() {
    set_page(PAGE_FILES);
    clear_page("RAMFS");
    ui::draw_badge(0, "FILES", "name  size  page metadata");
    ui::draw_table_header(1, "NAME          SIZE   PAGE");
    fs::list_to_vga(OUTPUT_START_ROW + 2, CONTENT_TEXT_COL, OUTPUT_ROWS - 3);
    ui::draw_next("cat readme / echo hello > note / rm note");
}

fn show_cat(name: &str) {
    set_page(PAGE_FILES);
    clear_output();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"cat ");
    len = copy_bytes(&mut line, len, name.as_bytes());
    write_output(0, as_str(&line[..len]));
    if !fs::cat_to_vga(name, OUTPUT_START_ROW + 1, CONTENT_TEXT_COL) {
        write_output(1, "file not found");
    }
}

fn show_echo(text: &str) {
    set_page(PAGE_FILES);
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
    set_page(PAGE_FILES);
    clear_output();
    if fs::remove(name) {
        write_output(0, "removed RAMFS file");
    } else {
        write_output(0, "file not found");
    }
}

fn show_ps() {
    set_page(PAGE_TASKS);
    clear_page("TASKS");
    ui::draw_badge(0, "CURRENT", "green row is running");
    ui::draw_table_header(1, "PID NAME         STATE    RUNTIME   STACK");
    task::dump_to_vga(OUTPUT_START_ROW + 2, CONTENT_TEXT_COL, OUTPUT_ROWS - 3);
    ui::draw_next("sched / trace sched / watch");
}

fn show_sched() {
    set_page(PAGE_TASKS);
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
    set_page(PAGE_STATUS);
    clear_page("SYSCALL TABLE");
    syscall::dump_table_to_vga(OUTPUT_START_ROW, CONTENT_TEXT_COL, 4);

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
    set_page(PAGE_SHELL);
    clear_output();
    let mut line = [0u8; 80];
    let mut len = copy_bytes(&mut line, 0, b"unknown command: ");
    len = copy_bytes(&mut line, len, command.as_bytes());
    write_output(0, as_str(&line[..len]));
    write_output(1, "try: help, ?, status, files, programs");
    write_output(2, "presentation: tour, demo, bench trace");
    write_output(3, "observability: viz, timeline, trace irq");
}

fn clear_output() {
    clear_page(current_page());
}

fn clear_page(title: &str) {
    ui::clear_output(title);
    ui::draw_footer_path(current_page(), "ready");
}

fn write_output(offset: usize, text: &str) {
    if offset >= OUTPUT_ROWS {
        return;
    }
    let color = if offset == 0 {
        vga::make_color(vga::Color::LightCyan, vga::Color::Black)
    } else {
        vga::make_color(vga::Color::LightGray, vga::Color::Black)
    };
    vga::write_at(OUTPUT_START_ROW + offset, CONTENT_TEXT_COL, text, color);
}

fn write_key_line(offset: usize, key: &str, text: &str) {
    if offset >= OUTPUT_ROWS {
        return;
    }
    let key_color = vga::make_color(vga::Color::LightGreen, vga::Color::Black);
    let text_color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    vga::write_at(OUTPUT_START_ROW + offset, CONTENT_TEXT_COL, key, key_color);
    vga::write_at(OUTPUT_START_ROW + offset, CONTENT_TEXT_COL + 4, text, text_color);
}

fn write_color_sample(offset: usize, col: usize, text: &str, fg: vga::Color) {
    if offset >= OUTPUT_ROWS {
        return;
    }
    let color = vga::make_color(fg, vga::Color::Black);
    vga::write_at(OUTPUT_START_ROW + offset, CONTENT_TEXT_COL + col, text, color);
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
    let cells = 16usize;
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

fn set_page(page: usize) {
    CURRENT_PAGE.store(page, Ordering::Relaxed);
}

fn page_for_command(command: &str) -> usize {
    match command {
        "" => PAGE_HOME,
        "help" | "?" | "h" | "help obs" | "help fs" | "help demo" => PAGE_HELP,
        "status" | "sysstat" | "s" | "ticks" | "viz" => PAGE_STATUS,
        "mem" | "m" => PAGE_MEMORY,
        "ps" | "p" | "sched" => PAGE_TASKS,
        "files" | "f" | "ls" | "cat readme" | "cat note" | "cat demo" | "cat userlog" | "echo hello > note" | "rm note" => PAGE_FILES,
        "trace" | "trace irq" | "trace sched" | "trace mem" | "trace syscall" | "trace file" | "trace demo" | "trace on" | "trace off" | "trace status" | "timeline" | "t" => PAGE_TRACE,
        "run" | "programs" | "r" | "run hello" | "run time" | "run trace" | "run files" => PAGE_RUN,
        "tour" | "guide" | "g" | "tour next" | "tour boot" | "tour mem" | "tour sched" | "tour syscall" | "tour fs" | "tour observe" | "tour demo" => PAGE_TOUR,
        "demo" | "d" | "demo sched" | "demo fs" | "demo syscall" | "demo trace" => PAGE_DEMO,
        "bench trace" | "bench" | "b" => PAGE_BENCH,
        "explain" | "explain irq" | "explain sched" | "explain mem" | "explain syscall" => PAGE_EXPLAIN,
        "colors" => PAGE_DIAG,
        "logo" => PAGE_LOGO,
        "clear" | "cls" | "q" => PAGE_SHELL,
        _ => PAGE_SHELL,
    }
}
