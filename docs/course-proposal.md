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
- Observable task table and round-robin scheduler model
- System call demo layer
- User program demo model backed by syscalls
- Shell and user commands
- Text UI chrome, command aliases, and command recall
- Command prediction with Right/Tab completion
- RAMFS single-directory file system
- Kernel event log
- Filtered kernel tracing and timeline view
- Guided demo commands
- Presentation tour and trace overhead benchmark

## Innovation

The main innovation is an observability layer:

- `klog`: inspect recent kernel events
- `trace`: filter boot/irq/mem/scheduler/syscall/file/demo events
- `trace on/off/status`: show how observability can be controlled
- `timeline`: inspect kernel activity as a chronological story
- `ps`: inspect the task table
- `sysstat`: inspect syscall counts and scheduler statistics
- `viz`: show an ASCII dashboard for memory, logs, IRQs, and scheduling
- `run`: launch small user-program demos through the syscall layer
- `tour`: guide a presentation through boot, memory, scheduling, syscalls, and RAMFS
- `bench trace`: demonstrate recorded vs skipped trace events
- `explain`: turn kernel mechanisms into in-OS teaching notes
- `demo`: trigger guided demonstrations for presentation and grading
- F1 / Up recalls the previous command for smoother live demos
- Ghost-text command prediction makes the shell easier to operate during grading

## Relationship to Orange'S

Orange'S is used as a learning reference and baseline. Nagi OS is implemented
as a separate Rust-first project with its own boot path, kernel structure,
observability model, and documentation.
