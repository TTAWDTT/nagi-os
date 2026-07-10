# Nagi OS Course Proposal

## Title

Nagi OS: A Rust-Based Observable Teaching Operating System

## Motivation

Traditional teaching operating systems often show results but hide the kernel's
internal activity. Nagi OS focuses on making the kernel observable: process
state, scheduling, system calls, file operations, and benchmark results should
be inspectable from inside the OS.

## Planned Modules

- Boot and long-mode entry
- VGA console and keyboard input
- IDT, PIC, PIT, and timer interrupt
- Task table and scheduler
- System call layer
- Shell and user commands
- Simple single-directory file system
- Kernel event log
- System call tracing
- Micro-benchmark commands

## Innovation

The main innovation is an observability layer:

- `klog`: inspect recent kernel events
- `trace`: enable or disable syscall/file/scheduler tracing
- `ps`: inspect process table
- `sysstat`: inspect syscall counts and scheduler statistics
- `bench`: compare selected workloads and trace overhead

## Relationship to Orange'S

Orange'S is used as a learning reference and baseline. Nagi OS is implemented
as a separate Rust-first project with its own boot path, kernel structure,
observability model, and documentation.

