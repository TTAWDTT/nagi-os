use crate::{fs, syscall, trace, vga};

const PROGRAM_COUNT: usize = 4;

pub struct ProgramResult {
    pub exit_code: u64,
    pub value: u64,
    pub message: &'static str,
}

pub fn init() {
    trace::record(trace::TraceKind::Demo, PROGRAM_COUNT as u64, "user-init");
}

pub fn run(name: &str) -> ProgramResult {
    trace::record(trace::TraceKind::Demo, name.len() as u64, "user-run");
    match name {
        "hello" => {
            let value = syscall::invoke(syscall::SYS_WRITE, 10, "user-hello");
            ProgramResult {
                exit_code: 0,
                value,
                message: "hello used sys_write",
            }
        }
        "time" => {
            let value = syscall::invoke(syscall::SYS_TIME, 0, "user-time");
            ProgramResult {
                exit_code: 0,
                value,
                message: "time used sys_time",
            }
        }
        "trace" => {
            let value = syscall::invoke(syscall::SYS_TRACE, 42, "user-trace");
            ProgramResult {
                exit_code: 0,
                value,
                message: "trace used sys_trace",
            }
        }
        "files" => {
            let _ = fs::create_or_write("userlog", "created by user program");
            let value = syscall::invoke(syscall::SYS_TRACE, 4, "user-files");
            ProgramResult {
                exit_code: 0,
                value,
                message: "files wrote RAMFS userlog",
            }
        }
        _ => ProgramResult {
            exit_code: 127,
            value: 0,
            message: "program not found",
        },
    }
}

pub fn list_to_vga(start_row: usize, col: usize, max_rows: usize) {
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let programs = [
        ("hello", "write through syscall"),
        ("time", "read PIT time through syscall"),
        ("trace", "emit user trace event"),
        ("files", "write RAMFS through kernel service"),
    ];

    let mut i = 0;
    while i < programs.len() && i < max_rows {
        let (name, desc) = programs[i];
        let mut line = [0u8; 80];
        let mut len = copy_bytes(&mut line, 0, b"  ");
        len = copy_bytes(&mut line, len, name.as_bytes());
        len = copy_bytes(&mut line, len, b" - ");
        len = copy_bytes(&mut line, len, desc.as_bytes());
        vga::write_at(start_row + i, col, as_str(&line[..len]), color);
        i += 1;
    }
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
