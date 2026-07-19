# Nagi OS A-Grade Implementation Checklist

This checklist is the implementation contract for the next development cycle.
Each numbered section is completed, verified, and committed separately.

## 1. Presentation Mode

- [x] Add a `present` command that opens a fixed defense-oriented story.
- [x] Add pages for cover, boot, IRQ, memory, scheduler, syscall, RAMFS,
      observability, and summary.
- [x] Support `n`/Right for next and `b`/Left for previous while presenting.
- [x] Give every page three explicit fields: implemented, observe, innovation.
- [x] Show page progress and the next available action.
- [x] Preserve `tour` as a free-form teaching guide.
- Verification: `present`, repeated `n`, repeated `b`, `present summary`.

## 2. Unified Page System

- [x] Introduce reusable page primitives for title, badges, metrics, table rows,
      diagrams, progress bars, and next-command hints.
- [x] Keep all command output inside the right content region.
- [x] Rework `help` as a categorized command index.
- [x] Rework `status`, `mem`, `ps`, `syscall`, `files`, and `trace` to share a
      calm title / metrics / body / next layout.
- [x] Highlight the current task and important changing values.
- [x] Add a dynamic breadcrumb/status footer.
- Verification: open each core page and confirm no sidebar overwrite or text
  outside the content region.

## 3. Live Watch Dashboard

- [x] Add `watch` and `watch off` commands.
- [x] Refresh uptime, IRQs, memory, task, syscall, file, and trace metrics from
      the PIT interrupt at a readable interval.
- [x] Add compact activity indicators and stable-width values.
- [x] Stop watch mode automatically when a normal command is entered.
- [x] Avoid redraws while the keyboard handler is mutating input state.
- Verification: run `watch`, wait for visible updates, type a command, confirm
  normal shell mode resumes.

## 4. Observable Kernel Stories

- [ ] Upgrade `timeline` into a structured time / kind / event view.
- [ ] Add `flow irq`, `flow syscall`, and `flow file` path diagrams.
- [ ] Add `replay` to narrate recent trace events in chronological order.
- [ ] Add `why` and topic variants that connect a visible metric to its cause.
- [ ] Add clear empty and trace-disabled states.
- [ ] Keep trace filters and overhead benchmark functional.
- Verification: `timeline`, all `flow` variants, `replay`, `why`, `trace off`,
  `timeline`, `trace on`.

## 5. Memory Map And Demonstration

- [ ] Track page ownership in the physical page allocator.
- [ ] Add `mem map` with a legend for kernel, task, filesystem, demo, used, and
      free pages.
- [ ] Add `mem demo` that allocates and frees a page and exposes the transition.
- [ ] Show fragmentation/longest-free-run and utilization metrics.
- [ ] Record ownership transitions in trace.
- Verification: compare `mem map` before and after `mem demo`; inspect
  `trace mem`.

## 6. Kernel Model Hardening

- [ ] Expand task states to Ready, Running, and Sleeping.
- [ ] Add scheduler state transitions and per-task runtime/switch metrics.
- [ ] Add a deterministic `sched demo` state-transition demonstration.
- [ ] Add syscall success/error result semantics and per-syscall counters.
- [ ] Add invalid syscall demonstration and readable error names.
- [ ] Extend RAMFS entries with size, revision, and page ownership metadata.
- [ ] Show structured file metadata and preserve create/read/write/remove flows.
- Verification: `ps`, `sched demo`, `syscall stats`, `syscall invalid`, `files`,
  file creation/update/removal.

## 7. Shell Interaction Polish

- [ ] Replace one-command recall with a multi-command history ring.
- [ ] Support Up/Down history navigation without losing the draft command.
- [ ] Support Up/Down completion candidate selection while filtering.
- [ ] Keep Left/Right/Home/End/Delete/Backspace editing functional.
- [ ] Let Tab/Right accept the selected inline completion.
- [ ] Highlight the selected sidebar candidate.
- [ ] Suggest the closest command for unknown input.
- [ ] Make contextual shortcuts (`n`, `b`, arrows) work naturally in
      presentation mode.
- Verification: exercise editing, history boundaries, completion selection,
  unknown command correction, and presentation navigation.

## 8. Documentation And Demonstration Assets

- [ ] Update README feature and command references.
- [ ] Add `docs/design.md` for visual and interaction decisions.
- [ ] Add `docs/architecture.md` with boot and kernel subsystem diagrams.
- [ ] Add `docs/course-report.md` as a report/defense writing foundation.
- [ ] Document a 30-second route and a full defense route.
- [ ] Capture at least one current UI screenshot from QEMU.
- [ ] Explain innovation, limitations, Rust rationale, and reproducible
      C/Rust comparison methodology without making unsupported claims.
- Verification: commands and document claims match the built image.

## 9. Final Verification And Summary

- [ ] Build the disk image from a clean incremental state.
- [ ] Pass the automated smoke test.
- [ ] Run scripted serial checks for the expected boot markers.
- [ ] Inspect git status and commit history for isolated implementation commits.
- [ ] Push all commits to `origin/main`.
- [ ] Add `docs/implementation-summary.md` with completed work, architecture,
      validation, demo route, remaining limitations, and extension directions.
- [ ] Commit the summary as the final implementation commit.
