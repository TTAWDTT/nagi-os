use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{klog, mem, trace, vga};

const TASK_CAPACITY: usize = 4;
const SCHEDULE_INTERVAL_TICKS: u64 = 25;

static mut TASKS: [Task; TASK_CAPACITY] = [
    Task::empty(),
    Task::empty(),
    Task::empty(),
    Task::empty(),
];
static TASK_COUNT: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);
static SWITCHES: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, PartialEq)]
pub enum TaskState {
    Empty,
    Ready,
    Running,
    Sleeping,
}

impl TaskState {
    fn as_str(self) -> &'static str {
        match self {
            TaskState::Empty => "empty",
            TaskState::Ready => "ready",
            TaskState::Running => "running",
            TaskState::Sleeping => "sleep",
        }
    }
}

#[derive(Clone, Copy)]
struct Task {
    pid: u32,
    name: [u8; 12],
    state: TaskState,
    ticks: u64,
    stack_page: usize,
}

impl Task {
    const fn empty() -> Self {
        Self {
            pid: 0,
            name: [0; 12],
            state: TaskState::Empty,
            ticks: 0,
            stack_page: 0,
        }
    }
}

pub fn init() {
    unsafe {
        let mut i = 0;
        while i < TASK_CAPACITY {
            TASKS[i] = Task::empty();
            i += 1;
        }
    }

    TASK_COUNT.store(0, Ordering::Relaxed);
    CURRENT.store(0, Ordering::Relaxed);
    SWITCHES.store(0, Ordering::Relaxed);

    spawn("idle", TaskState::Running);
    spawn("logger", TaskState::Ready);
    spawn("worker", TaskState::Ready);
    spawn("shell", TaskState::Sleeping);
    trace::record(trace::TraceKind::Schedule, TASK_CAPACITY as u64, "task-init");
}

pub fn on_tick(tick: u64) {
    let count = TASK_COUNT.load(Ordering::Relaxed);
    if count == 0 {
        return;
    }

    let current = CURRENT.load(Ordering::Relaxed);
    unsafe {
        TASKS[current].ticks = TASKS[current].ticks.saturating_add(1);
    }

    if tick % SCHEDULE_INTERVAL_TICKS != 0 {
        return;
    }

    let next = (current + 1) % count;
    unsafe {
        if TASKS[current].state == TaskState::Running {
            TASKS[current].state = TaskState::Ready;
        }
        TASKS[next].state = TaskState::Running;
    }
    CURRENT.store(next, Ordering::Relaxed);
    SWITCHES.fetch_add(1, Ordering::Relaxed);
    trace::record(trace::TraceKind::Schedule, next as u64, "switch");
    klog::record(klog::EventType::Schedule, current as u64, next as u64, "rr-switch");
}

pub fn count() -> usize {
    TASK_COUNT.load(Ordering::Relaxed)
}

pub fn current_pid() -> u32 {
    unsafe { TASKS[CURRENT.load(Ordering::Relaxed)].pid }
}

pub fn switches() -> usize {
    SWITCHES.load(Ordering::Relaxed)
}

pub fn interval_ticks() -> u64 {
    SCHEDULE_INTERVAL_TICKS
}

pub fn dump_to_vga(start_row: usize, col: usize, max_rows: usize) {
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let count = TASK_COUNT.load(Ordering::Relaxed);
    let mut i = 0;
    while i < count && i < max_rows {
        let task = unsafe { TASKS[i] };
        let mut line = [0u8; 80];
        let mut len = copy_bytes(&mut line, 0, b"  ");
        len = append_u64(&mut line, len, task.pid as u64);
        len = copy_bytes(&mut line, len, b" ");
        len = copy_bytes(&mut line, len, name_as_str(&task.name).as_bytes());
        len = copy_bytes(&mut line, len, b" ");
        len = copy_bytes(&mut line, len, task.state.as_str().as_bytes());
        len = copy_bytes(&mut line, len, b" ticks=");
        len = append_u64(&mut line, len, task.ticks);
        len = copy_bytes(&mut line, len, b" stack=");
        len = append_u64(&mut line, len, task.stack_page as u64);
        let row_color = if i == CURRENT.load(Ordering::Relaxed) {
            vga::make_color(vga::Color::LightGreen, vga::Color::Black)
        } else {
            color
        };
        vga::write_at(start_row + i, col, as_str(&line[..len]), row_color);
        i += 1;
    }
}

fn spawn(name: &str, state: TaskState) {
    let idx = TASK_COUNT.load(Ordering::Relaxed);
    if idx >= TASK_CAPACITY {
        return;
    }

    let stack_page = mem::alloc_page_owned(mem::PageOwner::Task, name).unwrap_or(0);
    let mut task = Task::empty();
    task.pid = idx as u32;
    task.state = state;
    task.stack_page = stack_page;
    copy_name(&mut task.name, name);

    unsafe {
        TASKS[idx] = task;
    }
    TASK_COUNT.store(idx + 1, Ordering::Relaxed);
}

fn copy_name(dst: &mut [u8; 12], name: &str) {
    for (i, byte) in name.bytes().take(dst.len() - 1).enumerate() {
        dst[i] = byte;
    }
}

fn name_as_str(bytes: &[u8; 12]) -> &str {
    let mut len = 0;
    while len < bytes.len() && bytes[len] != 0 {
        len += 1;
    }
    core::str::from_utf8(&bytes[..len]).unwrap_or("?")
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
