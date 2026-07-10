<p align="center">
  <img src="logo.png" alt="Nagi OS logo" width="200" height="200" />
</p>
<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" />
  <img src="https://img.shields.io/badge/language-Rust-orange.svg" alt="Language: Rust" />
  <img src="https://img.shields.io/badge/target-x86__64-yellow.svg" alt="Target: x86_64" />
  <img src="https://img.shields.io/badge/OS-baremetal-lightgrey.svg" alt="OS: baremetal" />
</p>

# Nagi OS

**Nagi OS** 是一个由 Rust 构建的教学性质操作系统，也是 2024 年同济国豪软工班某四人组的操作系统课程设计。

名字来源于日文中的 **「凪」**，寓意为冷静、无风、稳定。该项目的目标是打造一个微型操作系统，将底层硬件和并发的内核活动转化为一个沉浸式、可观测的环境。

## 项目当前状态

此仓库当前可引导一个极小的 x86_64 核，已实现以下特性:

- 16-bit 模式下的 stage 1 引导扇区
- stage2 加载器
- 切换/过渡到 64-bit 长模式
- Rust `#![no_std]` 内核
- VGA 文本模式控制台
- IDT 与 CPU 异常诊断
- PIC 重映射与 PIT 100Hz 定时器中断
- 键盘 IRQ1 与基础扫描码输入
- 最小交互式 shell：`help`、`ticks`、`sysstat`、`klog`、`trace`、`clear`
- `sysstat` 可观测系统状态面板
- `trace` 近期内核事件追踪
- 早期内核事件日志 (`klog`) 骨架

## 项目目标

针对操作系统课程设计，Nagi OS 计划包含以下内容：

- 引导加载器与内核入口
- 控制台与键盘
- 中断与定时器
- 协作式/抢占式任务调度
- 系统调用
- 简单 shell
- 简单文件系统
- 可观测内核特性：
  - `ps`
  - `sysstat`
  - `trace`
  - `klog`
  - 基准测试命令

## 如何构建和运行

Nagi OS 是一个裸机操作系统项目。Rust 负责编译内核，NASM 负责汇编
bootloader，QEMU 负责模拟启动这个操作系统。

### Windows + WSL

推荐在 Windows PowerShell 中操作，并通过 WSL Ubuntu 提供 NASM、QEMU、
`objcopy` 等底层工具。

#### 1. 安装 Rust

```powershell
winget install --id Rustlang.Rustup -e
```

安装完成后，重新打开 PowerShell，然后验证：

```powershell
rustc --version
cargo --version
rustup --version
```

#### 2. 添加裸机编译目标

```powershell
rustup target add x86_64-unknown-none
```

验证 target 是否装好：

```powershell
rustup target list --installed
```

输出中应该能看到：

```text
x86_64-unknown-none
```

#### 3. 安装 WSL 工具

如果还没有 WSL Ubuntu，先安装：

```powershell
wsl --install -d Ubuntu
```

安装完成后，打开 Ubuntu 初始化用户，然后回到 PowerShell，安装构建和运行
Nagi OS 需要的工具：

```powershell
wsl --exec bash -lc "sudo apt-get update && sudo apt-get install -y nasm qemu-system-x86 binutils"
```

#### 4. 构建系统镜像

```powershell
cd D:\Github\nagi-os
.\scripts\build.ps1
```

构建成功后会生成：

```text
build\nagi-os.img
```

#### 5. 快速验证能否启动

```powershell
.\scripts\smoke.ps1
```

成功时会看到类似输出：

```text
Nagi OS booted
QEMU_STILL_RUNNING_AFTER_5S
```

这表示内核已经启动，并且没有立刻崩溃退出。

#### 6. 打开 QEMU 窗口运行

```powershell
.\scripts\run.ps1
```

启动后可以看到 Nagi OS 的 VGA 文本界面。

如果窗口没有正常弹出，可以换一个 QEMU 显示后端：

```powershell
.\scripts\run.ps1 -Display gtk
.\scripts\run.ps1 -Display serial
```

`serial` 和 `none` 都会使用无窗口模式，只在终端里输出启动日志；这种模式主要
用来确认内核是否成功启动，不能操作 VGA Shell。WSL 提示 localhost 代理未镜像
通常不影响运行，可以先忽略。

### Linux

#### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

`source "$HOME/.cargo/env"` 用于让当前终端立刻识别 `rustc`、`cargo`、
`rustup` 等 Rust 命令。

验证：

```bash
rustc --version
cargo --version
rustup --version
```

#### 2. 添加裸机编译目标

```bash
rustup target add x86_64-unknown-none
```

#### 3. 安装构建和运行工具

```bash
sudo apt-get update
sudo apt-get install -y nasm qemu-system-x86 binutils
```

#### 4. 构建和运行

```bash
git clone https://github.com/TTAWDTT/nagi-os.git
cd nagi-os
make
make run
```

也可以只构建不运行：

```bash
make
```

## 课程设计定位

Nagi OS 不是对 Orange'S 的直接修改。它将 Orange'S 和 xv6 作为操作系统概念
学习参考，而实现本身以 Rust 为主，并重点聚焦内核可观测性。

项目规划中的创新点，是通过命令和日志暴露内核内部行为，让调度、系统调用和
文件操作都能在操作系统内部被观察到。
