# Nagi OS

**Nagi OS** is a Rust-based observable teaching operating system.

The name comes from **凪**: calm, windless, and stable. The project goal is to
make a small operating system that turns low-level hardware and concurrent
kernel activity into a calm, inspectable environment.

## Current Status

This repository currently boots a minimal x86_64 kernel:

- 16-bit stage1 boot sector
- stage2 loader
- transition to 64-bit long mode
- Rust `#![no_std]` kernel
- VGA text console
- early kernel event log (`klog`) skeleton

## Project Goals

For the operating system course design, Nagi OS aims to include:

- bootloader and kernel entry
- console and keyboard
- interrupts and timer
- cooperative/preemptive task scheduling
- system calls
- simple shell
- simple file system
- observable-kernel features:
  - `ps`
  - `sysstat`
  - `trace`
  - `klog`
  - benchmark commands

## Build on Windows + WSL

Required tools:

- Rust with `x86_64-unknown-none`
- WSL Ubuntu
- NASM inside WSL
- QEMU inside WSL

Install the Rust target:

```powershell
rustup target add x86_64-unknown-none
```

Install WSL tools:

```bash
sudo apt-get update
sudo apt-get install -y nasm qemu-system-x86
```

Build:

```powershell
.\scripts\build.ps1
```

Run:

```powershell
.\scripts\run.ps1
```

Smoke test:

```powershell
.\scripts\smoke.ps1
```

## Build on Linux

```bash
rustup target add x86_64-unknown-none
sudo apt-get install -y nasm qemu-system-x86 binutils
make
make run
```

## Course Design Positioning

Nagi OS is not a direct modification of Orange'S. It uses Orange'S and xv6 as
references for OS concepts, while the implementation is Rust-first and focused
on kernel observability.

The planned innovation is to expose internal kernel behavior through commands
and logs so that scheduling, system calls, and file operations can be observed
from inside the OS.

