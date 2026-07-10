use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{klog, trace};

pub const PAGE_SIZE: usize = 4096;
const TOTAL_PAGES: usize = 128;
const RESERVED_PAGES: usize = 8;

static mut PAGE_USED: [bool; TOTAL_PAGES] = [false; TOTAL_PAGES];
static USED_PAGES: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static FREES: AtomicUsize = AtomicUsize::new(0);
static FAILED_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
pub struct MemoryStats {
    pub page_size: usize,
    pub total_pages: usize,
    pub reserved_pages: usize,
    pub used_pages: usize,
    pub free_pages: usize,
    pub allocations: usize,
    pub frees: usize,
    pub failed_allocations: usize,
}

pub fn init() {
    unsafe {
        let mut i = 0;
        while i < TOTAL_PAGES {
            PAGE_USED[i] = i < RESERVED_PAGES;
            i += 1;
        }
    }

    USED_PAGES.store(RESERVED_PAGES, Ordering::Relaxed);
    ALLOCATIONS.store(0, Ordering::Relaxed);
    FREES.store(0, Ordering::Relaxed);
    FAILED_ALLOCATIONS.store(0, Ordering::Relaxed);
    trace::record(trace::TraceKind::Memory, RESERVED_PAGES as u64, "mem-init");
    klog::record(klog::EventType::Memory, TOTAL_PAGES as u64, RESERVED_PAGES as u64, "page-pool");
}

pub fn alloc_page(label: &str) -> Option<usize> {
    unsafe {
        let mut i = RESERVED_PAGES;
        while i < TOTAL_PAGES {
            if !PAGE_USED[i] {
                PAGE_USED[i] = true;
                USED_PAGES.fetch_add(1, Ordering::Relaxed);
                ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
                trace::record(trace::TraceKind::Memory, i as u64, label);
                return Some(i);
            }
            i += 1;
        }
    }

    FAILED_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    trace::record(trace::TraceKind::Memory, 0, "alloc-fail");
    None
}

pub fn free_page(page: usize, label: &str) -> bool {
    if page < RESERVED_PAGES || page >= TOTAL_PAGES {
        return false;
    }

    unsafe {
        if !PAGE_USED[page] {
            return false;
        }
        PAGE_USED[page] = false;
    }

    USED_PAGES.fetch_sub(1, Ordering::Relaxed);
    FREES.fetch_add(1, Ordering::Relaxed);
    trace::record(trace::TraceKind::Memory, page as u64, label);
    true
}

pub fn stats() -> MemoryStats {
    let used_pages = USED_PAGES.load(Ordering::Relaxed);
    MemoryStats {
        page_size: PAGE_SIZE,
        total_pages: TOTAL_PAGES,
        reserved_pages: RESERVED_PAGES,
        used_pages,
        free_pages: TOTAL_PAGES - used_pages,
        allocations: ALLOCATIONS.load(Ordering::Relaxed),
        frees: FREES.load(Ordering::Relaxed),
        failed_allocations: FAILED_ALLOCATIONS.load(Ordering::Relaxed),
    }
}

pub fn is_used(page: usize) -> bool {
    if page >= TOTAL_PAGES {
        return false;
    }
    unsafe { PAGE_USED[page] }
}
