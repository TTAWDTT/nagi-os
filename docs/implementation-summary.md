# Nagi OS 本轮实现总结

## 结果概览

本轮从 `376d317 Add animated Nagi logo mark` 继续推进，完成了清单中的全部
九个部分。Nagi OS 现在不仅能够从自制 bootloader 启动 Rust `no_std` 内核，
还形成了一套围绕“可观测、可解释、可答辩”的完整交互系统。

当前 release 内核为 61,474 bytes（121 sectors），磁盘镜像能够在 QEMU 中稳定
启动，release 构建零 warning，自动 smoke 和键盘驱动集成验证均通过。

## 已完成的提交

| Commit | 完成内容 |
|---|---|
| `8379d0d` | 固化 A-grade 实施清单和验收边界 |
| `8a974cc` | 九页 `present` 答辩模式与前后翻页 |
| `55b584d` | 统一页面组件、内容区约束、breadcrumb |
| `753dcb3` | PIT 驱动的 4 Hz `watch` 实时仪表盘 |
| `25360dc` | timeline、replay、flow、why 可观测故事 |
| `185087c` | 128 页所有权地图和可切换内存演示 |
| `d8b6932` | 调度、syscall、RAMFS 模型强化 |
| `bc702ab` | 8 条历史、候选选择、纠错和草稿恢复 |
| `e4fb24f` | README、架构、设计、报告、路线和截图 |
| `a7dc9e8` | 清理 dead code，实现零 warning 构建 |

## 核心能力

### 启动与硬件

- 16-bit stage1 和 stage2 加载器。
- A20、GDT、页表、protected mode 和 x86_64 long mode 切换。
- IDT、异常诊断、PIC 重映射、PIT 100 Hz、键盘 IRQ1。
- VGA text mode 和 serial 双输出。

### 内核模型

- 128 个 4 KiB 页的分配器，记录 Kernel/Task/File/Demo/Used/Free 所有权。
- free-run 和 longest-free-run 碎片指标。
- Ready/Running/Sleeping 任务状态、轮转调度、运行 tick 和切换计数。
- write/time/trace/stats syscall、分项计数、`ENOSYS` 安全错误路径。
- RAMFS create/read/update/remove、size、revision、created/modified tick 和页元数据。

### 可观测性与 UX

- `watch` 实时展示 uptime、IRQ、任务、内存、文件、syscall 和 trace。
- `timeline` 表格、`replay` 叙事重放、trace 子系统过滤。
- `flow irq/syscall/file` 路径图和 `why` 因果解释页。
- `present` 固定答辩叙事和 `tour` 自由教学导览。
- 稳定页面组件、动态 breadcrumb、PIT 驱动的 NAGI 风纹。
- 行内补全、候选高亮、Up/Down 选择、8 条历史、草稿恢复、编辑距离纠错。

## 验证证据

执行 `scripts/smoke.ps1` 得到：

```text
Built D:\Github\nagi-os\build\nagi-os.img
Kernel size: 61474 bytes (121 sectors)
PIT timer interrupt online
Nagi OS booted
QEMU_STILL_RUNNING_AFTER_5S
```

键盘驱动集成测试在同一 QEMU 实例中依次执行：

```text
mem map
mem demo
mem demo
sched demo
syscall invalid
watch
timeline
```

串口记录包含全部 `shell command:` 标记，无 `kernel exception` 或 `panic`，
QEMU monitor 最终报告 `VM status: running`。另一轮测试使用 Down+Tab 将 `tr`
补全为 `trace irq`，再用 Up 历史重复执行，串口两次记录 `trace irq`。

真实 QEMU VGA 截图位于 [nagi-console.png](nagi-console.png)。

## 推荐演示

最快路线：

```text
present -> Right -> Right -> watch -> mem map -> mem demo -> timeline
```

完整路线和答辩问答见 [demo-guide.md](demo-guide.md)，架构说明见
[architecture.md](architecture.md)，报告底稿见 [course-report.md](course-report.md)。

## 当前限制

- 调度器展示真实 timer-driven 状态变化，但尚未保存/恢复独立 CPU context。
- 用户程序和 syscall 目前在 ring 0 中走可观测模型，尚未切换到 ring 3。
- RAMFS 固定四个文件槽，重启后不持久化。
- 内存分配器使用受控 128 页池，尚未解析 BIOS/UEFI memory map。
- VGA text mode 的表现力和 curses 灰色映射受终端限制；VNC 才是调色板基准。

这些限制已在架构和课程报告中明确，不会用演示效果冒充尚未实现的隔离机制。

## 下一阶段扩展

优先级最高的是 CPU context save/restore、ring-3 用户态、`syscall/sysret`、每进程
页表和 page fault 驱动的虚拟内存。随后可以扩展动态 RAMFS block、framebuffer
图形界面、virtio block/network 驱动，以及自动化 QEMU 键盘回归测试。性能研究应按
课程报告中的同算法、同工具链条件建立 C/Rust 对照，不能仅凭语言做结论。

## 仓库状态

本轮所有实现提交均已推送到：

<https://github.com/TTAWDTT/nagi-os>

