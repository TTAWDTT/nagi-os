# Nagi OS Course Report Foundation

## Abstract

Nagi OS is a small x86_64 teaching operating system written primarily in Rust.
It implements a two-stage BIOS boot path, long-mode transition, VGA and serial
output, IDT/PIC/PIT and keyboard interrupts, a physical-page allocator, an
observable scheduling model, syscall dispatch, RAMFS, and an interactive
shell. Its central contribution is an observability layer that connects kernel
events to live dashboards, ownership maps, timelines, causal explanations, and
a repeatable defense narrative.

## Objectives and requirements

The project demonstrates the main operating-system concepts required by the
course while remaining small enough to explain completely. The functional
baseline is boot, interrupts, memory, tasks, system calls, files, and user
interaction. The quality target adds reproducibility, clear failure semantics,
cohesive UI/UX, documentation, and observable evidence for every subsystem.

## Design summary

The BIOS loads stage1, which loads stage2. stage2 reads the generated number of
kernel sectors, enables A20, installs a GDT, creates identity-mapped page
tables, enables long mode, and jumps to the Rust kernel. The kernel initializes
serial/VGA, event buffers, memory, tasks, RAMFS, syscalls, IDT, PIC, PIT, and
keyboard in dependency order.

The 128-page allocator records both allocation state and owner. Tasks use a
round-robin state model with Ready, Running, and Sleeping states. The syscall
dispatcher implements write/time/trace/stats, per-call counters, and ENOSYS.
RAMFS stores file content and metadata in fixed slots backed by owned pages.
Trace and klog rings receive events directly from these subsystems.

## Innovation

1. Kernel-native observability: `watch`, `timeline`, `replay`, filters, and
   `why` consume actual runtime counters and events.
2. Semantic memory visualization: `mem map` exposes ownership, free runs, and
   longest free span rather than only showing a percentage.
3. Explainable execution paths: `flow irq/syscall/file` links user-visible
   operations to kernel boundaries.
4. Defense as a system mode: `present` offers a fixed, keyboard-navigable story
   with implemented/observe/innovation fields.
5. Constraint-aware UI: stable page geometry, contextual completion, history,
   breadcrumb, and PIT-driven motion make VGA text mode operationally useful.

## Verification strategy

- Build verification: release `no_std` kernel, NASM boot stages, exact image
  sector count, and generated disk image.
- Boot verification: serial markers for PIT online and kernel boot, plus a
  five-second liveness check under QEMU.
- Interactive verification: routes in `docs/demo-guide.md`, especially page
  toggles, scheduler transitions, invalid syscalls, and RAMFS revisions.
- Visual verification: a QEMU VGA screendump from the built image, not a mockup.
- Trace verification: each subsystem action is followed by its filtered trace.

## Rust and C performance comparison

No defensible conclusion is "Rust is faster" or "C is faster" without matched
implementations. A fair experiment uses the same QEMU version and CPU model,
release optimization, boot protocol, memory layout, algorithms, I/O
suppression, and workload.

| Metric | Method |
|---|---|
| Kernel size | Compare stripped flat binaries |
| Boot work | Measure TSC/PIT interval from entry to ready marker |
| Syscall dispatch | Run a fixed in-kernel loop with output disabled |
| Allocation | Allocate/free the same page sequence |
| Trace overhead | Compare identical loops with trace on and off |
| Reliability | Record crashes, invalid accesses, and failed builds separately |

Repeat warm runs, report median and spread, retain compiler versions and flags,
and inspect generated assembly for outliers. Rust's strongest expected benefit
here is safety and maintainability; performance is an empirical result, not a
language slogan.

## Four-person collaboration

| Role | Main responsibility | Deliverable |
|---|---|---|
| Kernel integrator | Own buildable mainline and cross-module code | Bootable image and reviewed commits |
| Systems analyst | Requirements, Orange'S comparison, architecture | Diagrams and concept mapping |
| Verification lead | Smoke tests, command matrix, benchmarks | Reproducible evidence and issue log |
| Presentation lead | Report, slides, demo route, rehearsal | Defense narrative and screenshots |

Because most code may be produced through one person's AI-assisted workflow,
the other roles should own verifiable artifacts and review decisions rather
than manufacture artificial code ownership. Every member must be able to
explain the boot path, one subsystem, one innovation, and one limitation.

## Limitations and future work

The scheduler does not yet save and restore independent CPU contexts. User
programs do not execute in ring 3, and syscalls are direct dispatcher calls.
RAMFS is fixed-capacity and volatile. The allocator models a controlled page
pool rather than discovering firmware memory maps. Future work should add
context switching, ring-3 isolation, `syscall/sysret`, per-process page tables,
dynamic file blocks, framebuffer graphics, and automated keyboard-driven QEMU
integration tests.
