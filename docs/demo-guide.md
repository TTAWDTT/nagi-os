# Nagi OS Demo Guide

## 30-second route

1. Run `present` and say: "This is a Rust x86_64 OS that boots through our own
   two-stage loader. Its distinctive goal is an observable kernel."
2. Press Right twice to reach IRQ, then run `watch`. Point out changing ticks,
   IRQs, task switches, and the PIT-driven activity indicator.
3. Run `mem map`, then `mem demo`. Point out `K`, `T`, `F`, and the new `D` page.
4. Run `timeline`, then close with `present summary`.

## Full defense route

1. `present`: introduce the nine-page story and navigate with Right.
2. `klog`: show the boot sequence as kernel-owned evidence.
3. `flow irq`, then `watch`: connect hardware interrupts to changing state.
4. `ps`, `sched demo`, `trace sched`: show task state transitions.
5. `mem map`, `mem demo`, `why mem`: connect ownership to fragmentation.
6. `syscall invalid`, `syscall stats`, `flow syscall`: show safe dispatch.
7. `files`, `cat readme`, `echo hello > note`, `cat note`, `flow file`: show
   revision and page metadata through a full file lifecycle.
8. `replay`, `timeline`, `bench trace`: explain observability and its cost.
9. `present summary`: restate the complete path and extension boundaries.

## Likely questions

**Is this a real OS?** It boots on bare-metal conventions, enters x86_64 long
mode, installs an IDT/PIC/PIT, handles keyboard interrupts, and runs `no_std`
Rust without a host OS. Scheduling and user mode are currently models rather
than hardware context/ring transitions.

**Why Rust?** Rust removes many accidental memory-safety failures while still
allowing explicit `unsafe` blocks for ports, VGA memory, IDT loading, and CPU
register operations. The project does not claim that language choice alone
makes it faster than C.

**What is innovative?** Observation is part of the kernel architecture: page
ownership, event replay, causal `why` pages, live IRQ-driven dashboards, and a
defense narrative all consume the same runtime state as the kernel itself.

**What comes next?** Save/restore CPU contexts, ring-3 transitions, a real
syscall instruction path, virtual memory per process, dynamic files, and a
graphics framebuffer are the strongest next milestones.
