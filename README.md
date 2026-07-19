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

![Nagi OS presentation console](docs/nagi-console.png)

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
- 最小交互式 shell 与课程展示命令
- `sysstat` 可观测系统状态面板
- `mem` 物理页分配器状态与页位图
- `ps` / `sched` 可观察任务表与轮转调度模型
- `syscall` 系统调用表与 write/time/trace/stats 演示
- `trace` 近期内核事件追踪，支持按类型过滤
- `timeline` 内核事件时间线
- `explain` 教学解释页
- `viz` ASCII 内核状态可视化面板
- `ls` / `cat` / `echo` / `rm` RAMFS 内存文件系统
- `run` 用户程序演示模型
- `tour` 课程答辩导览
- `bench trace` 可观测性开销演示
- `demo` 一键式课程答辩演示入口
- 轻量顶部提示、底部状态、输出区标题、输入光标
- 顶部动态 `NAGI` 风纹标识，由 PIT tick 驱动
- `present` 九页固定答辩叙事，支持方向键和 `n` / `b` 翻页
- `watch` 由 PIT 驱动的 4 Hz 实时内核仪表盘
- `mem map` 带 Kernel / Task / File / Demo 所有权的 128 页地图
- `flow` / `replay` / `why` 可解释内核路径与事件故事
- Ready / Running / Sleeping 任务状态和逐任务运行/切换计数
- syscall 分项统计、`ENOSYS` 错误路径和无效调用演示
- RAMFS 文件大小、revision、创建/修改 tick 和物理页元数据
- 8 条命令历史、草稿恢复、候选选择和未知命令纠错
- F1 / 上方向键召回上一条命令
- 左右方向键移动输入光标，Home / End 跳到行首 / 行尾
- Delete 删除光标后的字符，Backspace 删除光标前的字符
- 命令预测直接显示在输入区中，已输入内容为蓝色，预测尾巴为灰色
- 右方向键 / Tab 接受补全
- Esc 清空当前输入
- 标题、快捷键和正文使用克制高亮区分
- 早期内核事件日志 (`klog`) 骨架

## 项目目标

针对操作系统课程设计，Nagi OS 计划包含以下内容：

- 引导加载器与内核入口
- 控制台与键盘
- 中断与定时器
- 可观察任务调度模型
- 系统调用演示层
- 简单 shell
- RAMFS 内存文件系统
- 可观测内核特性：
  - `ps`
  - `sysstat`
  - `trace`
  - `klog`
  - `timeline`
  - `explain`
  - `viz`
  - `run`
  - `tour`
  - `bench`
  - `demo`

## Shell 命令速查

启动后可以在 `nagi>` 后输入命令：

```text
help              查看命令列表
?                 help 的简写
h                 help 的一键入口
help obs          查看可观测性命令
help fs           查看 RAMFS 命令
help demo         查看答辩演示命令
ticks             查看 PIT tick
sysstat           查看系统状态
status            sysstat 的简写
s                 status 的一键入口
mem               查看物理页分配器
m                 mem 的一键入口
mem map           查看 128 页所有权地图
mem demo          切换一个 Demo 页的分配/释放状态
viz               查看 ASCII 状态面板
v                 viz 的一键入口
watch             打开 4 Hz 实时内核仪表盘
watch off         停止实时仪表盘
ps                查看内核任务表
p                 ps 的一键入口
sched             查看调度状态
sched demo        切换 worker 的 Sleeping/Ready 状态
syscall           运行系统调用演示
syscall stats     查看分项调用和错误计数
syscall invalid   演示 ENOSYS 安全错误路径
klog              查看内核日志
trace             查看近期 trace
trace irq         过滤中断事件
trace sched       过滤调度事件
trace mem         过滤内存事件
trace syscall     过滤系统调用事件
trace file        过滤文件系统事件
trace on          开启 trace 记录
trace off         关闭 trace 记录
trace status      查看 trace 开关与 skipped 计数
timeline          查看统一事件时间线
replay            用叙事形式重放近期事件
flow irq          查看硬件中断处理路径
flow syscall      查看用户意图到内核服务的路径
flow file         查看 RAMFS 操作路径
why irq           解释指标变化原因，也支持 mem/sched/syscall/file
explain irq       解释中断路径
explain sched     解释调度模型
explain mem       解释内存模型
explain syscall   解释系统调用模型
run               查看用户程序
run hello         运行 hello 用户程序
run time          运行 time 用户程序
run trace         运行 trace 用户程序
run files         运行 RAMFS 用户程序
r                 run 的一键入口
ls                列出 RAMFS 文件
files             ls 的简写
f                 files 的一键入口
cat readme        读取 RAMFS 文件
echo hello > note 写入 note 文件
rm note           删除 RAMFS 文件
tour              查看答辩导览主题
guide             tour 的简写
g                 guide 的一键入口
tour next         翻到下一页导览
n                 tour next 的一键入口
present           进入九页固定答辩模式
present summary   直接打开答辩总结页
bench trace       演示 trace on/off 的记录差异
b                 bench trace 的一键入口
demo              查看演示主题
d                 demo 的一键入口
demo sched        触发/说明调度演示
demo fs           创建演示文件
demo syscall      触发系统调用演示
logo              查看 Nagi 动态标识说明
clear / cls       清空输出区
q                 clear 的一键入口
```

输入区支持完整的单行编辑：左右方向键移动光标，`Home` / `End` 跳到行首 / 行尾，
`Delete` 删除光标后的字符。输入 `r` 会在输入区中显示 `r` 加灰色预测尾巴 `un`，
输入 `trace s` 会显示灰色预测尾巴 `ched`；光标在行尾时按右方向键或 `Tab` 接受补全。
输入前缀时用上下方向键选择候选；空输入时用上下方向键浏览 8 条历史并保留原草稿，
`F1` 直接召回最新历史，`Esc` 清空当前输入。答辩模式下空输入的左右方向键负责翻页。

## 演示与课程文档

- [30 秒与完整答辩路线](docs/demo-guide.md)
- [系统架构](docs/architecture.md)
- [界面与交互设计](docs/design.md)
- [课程报告底稿](docs/course-report.md)
- [本轮完整实施清单](docs/implementation-checklist.md)
- [本轮实现总结](docs/implementation-summary.md)

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

启动后会在当前终端里看到 Nagi OS 的 VGA 文本界面。

如果想尝试单独的 QEMU 图形窗口，可以换一个显示后端：

```powershell
.\scripts\run.ps1 -Display sdl
.\scripts\run.ps1 -Display gtk
.\scripts\run.ps1 -Display vnc
.\scripts\run.ps1 -Display serial
```

默认的 `curses` 模式最稳，会直接在终端中显示和操作 VGA Shell。`sdl` 和
`gtk` 依赖 WSLg 图形窗口；如果窗口没出现，可以先用 `curses`。不过 `curses`
会经过终端颜色映射，某些终端会把 VGA 的 `darkgray` / `lightgray` 显示错。
需要检查真实 VGA 调色板时，推荐使用 VNC 模式：

```powershell
.\scripts\run.ps1 -Display vnc
```

脚本会提示类似 `localhost:5901` 的地址，用 VNC Viewer 打开即可。进入系统后输入
`colors` 可以查看 16 色调色板。`serial` 和 `none` 都会使用无窗口模式，只在终端里
输出启动日志；这种模式主要用来确认内核是否成功启动，不能操作 VGA Shell。WSL 提示
localhost 代理未镜像通常不影响运行，可以先忽略。

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
