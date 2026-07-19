use crate::{klog, mem, trace, vga};

const FILE_CAPACITY: usize = 4;
const NAME_LEN: usize = 12;
const CONTENT_LEN: usize = 64;

static mut FILES: [RamFile; FILE_CAPACITY] = [
    RamFile::empty(),
    RamFile::empty(),
    RamFile::empty(),
    RamFile::empty(),
];

#[derive(Clone, Copy)]
struct RamFile {
    used: bool,
    name: [u8; NAME_LEN],
    content: [u8; CONTENT_LEN],
    len: usize,
    page: usize,
    revision: u64,
    created_tick: u64,
    modified_tick: u64,
}

impl RamFile {
    const fn empty() -> Self {
        Self {
            used: false,
            name: [0; NAME_LEN],
            content: [0; CONTENT_LEN],
            len: 0,
            page: 0,
            revision: 0,
            created_tick: 0,
            modified_tick: 0,
        }
    }
}

pub fn init() {
    unsafe {
        let mut i = 0;
        while i < FILE_CAPACITY {
            FILES[i] = RamFile::empty();
            i += 1;
        }
    }
    create_or_write("readme", "Nagi RAMFS: ls cat echo rm");
    create_or_write("motd", "observable kernel, teachable internals");
    create_or_write("note", "echo text > note");
    trace::record(trace::TraceKind::File, FILE_CAPACITY as u64, "ramfs-init");
}

pub fn create_or_write(name: &str, content: &str) -> bool {
    let idx = find_or_empty(name);
    if idx >= FILE_CAPACITY {
        trace::record(trace::TraceKind::File, 0, "fs-full");
        return false;
    }

    unsafe {
        if !FILES[idx].used {
            let page = match mem::alloc_page_owned(mem::PageOwner::File, name) {
                Some(page) => page,
                None => return false,
            };
            FILES[idx].page = page;
            FILES[idx].revision = 1;
            FILES[idx].created_tick = crate::pit::ticks();
        } else {
            FILES[idx].revision = FILES[idx].revision.saturating_add(1);
        }
        FILES[idx].used = true;
        FILES[idx].len = 0;
        clear_bytes(&mut FILES[idx].name);
        clear_bytes(&mut FILES[idx].content);
        copy_into(&mut FILES[idx].name, name.as_bytes());
        FILES[idx].len = copy_into(&mut FILES[idx].content, content.as_bytes());
        FILES[idx].modified_tick = crate::pit::ticks();
    }

    trace::record(trace::TraceKind::File, content.len() as u64, name);
    klog::record(klog::EventType::File, content.len() as u64, 0, name);
    true
}

pub fn remove(name: &str) -> bool {
    let idx = find(name);
    if idx >= FILE_CAPACITY {
        return false;
    }

    unsafe {
        let page = FILES[idx].page;
        FILES[idx] = RamFile::empty();
        let _ = mem::free_page(page, name);
    }

    trace::record(trace::TraceKind::File, 0, "rm");
    klog::record(klog::EventType::File, 0, 0, "rm");
    true
}

pub fn count() -> usize {
    let mut count = 0;
    unsafe {
        let mut i = 0;
        while i < FILE_CAPACITY {
            if FILES[i].used {
                count += 1;
            }
            i += 1;
        }
    }
    count
}

pub fn list_to_vga(start_row: usize, col: usize, max_rows: usize) {
    let color = vga::make_color(vga::Color::LightGray, vga::Color::Black);
    let mut row = 0;
    unsafe {
        let mut i = 0;
        while i < FILE_CAPACITY && row < max_rows {
            if FILES[i].used {
                let file = FILES[i];
                let mut line = [0u8; 80];
                let mut len = copy_bytes(&mut line, 0, b"  ");
                len = copy_bytes(&mut line, len, name_as_str(&file.name).as_bytes());
                len = copy_bytes(&mut line, len, b" len=");
                len = append_u64(&mut line, len, file.len as u64);
                len = copy_bytes(&mut line, len, b" rev=");
                len = append_u64(&mut line, len, file.revision);
                len = copy_bytes(&mut line, len, b" page=");
                len = append_u64(&mut line, len, file.page as u64);
                vga::write_at(start_row + row, col, as_str(&line[..len]), color);
                row += 1;
            }
            i += 1;
        }
    }
}

#[derive(Clone, Copy)]
pub struct FileMetadata {
    pub size: usize,
    pub page: usize,
    pub revision: u64,
    pub created_tick: u64,
    pub modified_tick: u64,
}

pub fn metadata(name: &str) -> Option<FileMetadata> {
    let idx = find(name);
    if idx >= FILE_CAPACITY {
        return None;
    }
    let file = unsafe { FILES[idx] };
    Some(FileMetadata {
        size: file.len,
        page: file.page,
        revision: file.revision,
        created_tick: file.created_tick,
        modified_tick: file.modified_tick,
    })
}

pub fn cat_to_vga(name: &str, row: usize, col: usize) -> bool {
    let idx = find(name);
    if idx >= FILE_CAPACITY {
        return false;
    }

    unsafe {
        let file = FILES[idx];
        vga::write_at(
            row,
            col,
            as_str(&file.content[..file.len]),
            vga::make_color(vga::Color::LightGray, vga::Color::Black),
        );
    }
    trace::record(trace::TraceKind::File, 0, name);
    true
}

fn find_or_empty(name: &str) -> usize {
    let found = find(name);
    if found < FILE_CAPACITY {
        return found;
    }

    unsafe {
        let mut i = 0;
        while i < FILE_CAPACITY {
            if !FILES[i].used {
                return i;
            }
            i += 1;
        }
    }
    FILE_CAPACITY
}

fn find(name: &str) -> usize {
    unsafe {
        let mut i = 0;
        while i < FILE_CAPACITY {
            if FILES[i].used && name_as_str(&FILES[i].name) == name {
                return i;
            }
            i += 1;
        }
    }
    FILE_CAPACITY
}

fn clear_bytes<const N: usize>(dst: &mut [u8; N]) {
    let mut i = 0;
    while i < N {
        dst[i] = 0;
        i += 1;
    }
}

fn copy_into<const N: usize>(dst: &mut [u8; N], src: &[u8]) -> usize {
    let mut i = 0;
    while i < src.len() && i + 1 < N {
        dst[i] = src[i];
        i += 1;
    }
    i
}

fn name_as_str(bytes: &[u8; NAME_LEN]) -> &str {
    let mut len = 0;
    while len < bytes.len() && bytes[len] != 0 {
        len += 1;
    }
    core::str::from_utf8(&bytes[..len]).unwrap_or("?")
}

fn copy_bytes(dst: &mut [u8], mut idx: usize, src: &[u8]) -> usize {
    for byte in src {
        if idx >= dst.len() {
            break;
        }
        dst[idx] = *byte;
        idx += 1;
    }
    idx
}

fn append_u64(buf: &mut [u8], idx: usize, mut value: u64) -> usize {
    if value == 0 {
        return copy_bytes(buf, idx, b"0");
    }

    let mut digits = [0u8; 20];
    let mut digit_idx = digits.len();
    while value > 0 {
        digit_idx -= 1;
        digits[digit_idx] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    copy_bytes(buf, idx, &digits[digit_idx..])
}

fn as_str(bytes: &[u8]) -> &str {
    unsafe { core::str::from_utf8_unchecked(bytes) }
}
