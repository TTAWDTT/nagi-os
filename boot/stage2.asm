bits 16
org 0x8000

KERNEL_ADDR equ 0x10000
KERNEL_LBA equ 9
KERNEL_SECTORS equ __KERNEL_SECTORS__
SECTORS_PER_TRACK equ 18
HEADS_PER_CYLINDER equ 2
PML4_TABLE equ 0x9000
PDPT_TABLE equ 0xA000
PD_TABLE equ 0xB000

stage2_start:
    cli
    cld
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    mov [boot_drive], dl

    mov si, stage2_msg
    call print_string

    call load_kernel

    call enable_a20
    lgdt [gdt64.pointer]

    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp CODE32:protected_mode

disk_error:
    mov si, disk_error_msg
    call print_string
    hlt
    jmp $

print_string:
    lodsb
    test al, al
    jz .done
    mov ah, 0x0E
    mov bx, 0x0007
    int 0x10
    jmp print_string
.done:
    ret

enable_a20:
    in al, 0x92
    or al, 00000010b
    out 0x92, al
    ret

load_kernel:
    xor di, di
.next_sector:
    cmp di, KERNEL_SECTORS
    jae .done

    mov ax, di
    shl ax, 5
    add ax, KERNEL_ADDR >> 4
    mov es, ax
    xor bx, bx

    mov ax, KERNEL_LBA
    add ax, di
    call lba_to_chs
    xor bx, bx

    mov ah, 0x02
    mov al, 1
    mov dl, [boot_drive]
    push di
    int 0x13
    pop di
    jc disk_error

    inc di
    jmp .next_sector
.done:
    ret

lba_to_chs:
    xor dx, dx
    mov bx, SECTORS_PER_TRACK
    div bx
    inc dl
    mov cl, dl

    xor dx, dx
    mov bx, HEADS_PER_CYLINDER
    div bx
    mov dh, dl
    mov ch, al
    ret

bits 32
protected_mode:
    mov ax, DATA32
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000

    call setup_page_tables

    mov eax, PML4_TABLE
    mov cr3, eax

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    jmp CODE64:long_mode

setup_page_tables:
    mov edi, PML4_TABLE
    xor eax, eax
    mov ecx, 4096 * 3 / 4
    rep stosd

    mov eax, PDPT_TABLE
    or eax, 0x003
    mov [PML4_TABLE], eax

    mov eax, PD_TABLE
    or eax, 0x003
    mov [PDPT_TABLE], eax

    mov eax, 0x00000083
    mov [PD_TABLE], eax
    ret

bits 64
long_mode:
    cld
    mov ax, DATA32
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov rsp, 0x90000

    mov rax, KERNEL_ADDR
    jmp rax

align 8
gdt64:
    dq 0
.code32:
    dq 0x00CF9A000000FFFF
.data32:
    dq 0x00CF92000000FFFF
.code64:
    dq 0x00AF9A000000FFFF
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

CODE32 equ gdt64.code32 - gdt64
DATA32 equ gdt64.data32 - gdt64
CODE64 equ gdt64.code64 - gdt64

boot_drive: db 0
stage2_msg: db "Nagi stage2", 13, 10, 0
disk_error_msg: db "kernel read error", 13, 10, 0

times 4096 - ($ - $$) db 0
