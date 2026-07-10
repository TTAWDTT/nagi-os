bits 16
org 0x7C00

STAGE2_ADDR equ 0x8000
STAGE2_SECTORS equ 8

start:
    cli
    cld
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti

    mov [boot_drive], dl

    mov si, stage1_msg
    call print_string

    mov ax, STAGE2_ADDR >> 4
    mov es, ax
    xor bx, bx
    mov ah, 0x02
    mov al, STAGE2_SECTORS
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13
    jc disk_error

    mov dl, [boot_drive]
    jmp 0x0000:STAGE2_ADDR

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

boot_drive: db 0
stage1_msg: db "Nagi stage1", 13, 10, 0
disk_error_msg: db "disk read error", 13, 10, 0

times 510 - ($ - $$) db 0
dw 0xAA55
