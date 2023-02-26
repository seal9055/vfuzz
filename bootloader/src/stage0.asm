[bits 16]
[org 0x7c00]

; Stage 0 bootloader. Performs some checks to verify that the system is 
; compatible before loading the stage1 bootloader to 0x1000 and jumping to it

; Contains information such as the offset and size of the stage1 bootloader
; that is used when loading it from disk
struc disk_address_packet_type
    .size:        resb 1 ; Size
    .zero:        resb 1 ; Always zero
    .num_sectors: resw 1 ; Number of 512 byte sectors
    .offset:      resw 1 ; Memory address that this data is being read into
    .segment:     resw 1 ; In memory page zero (used together with offset)
    .address_lo:  resd 1 ; This is the block on disk that data is being read from 
    .address_hi:  resd 1 ; More storage bytes if required
endstruc

; Entrypoint
entry:
    ; Disable Interrupts
    cli

    ; Initialize Segment Registers. Real mode uses 
    ; (segment-register << 4) + register when accessing memory, so these are all
    ; 0-initialized
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; Initialize Stack
    mov sp, 0x7c00

; Disk extension service is used to load the kernel from disk, so we check if it
; is supported
test_disk_extension:
    ; Save the drive number, dl was initialized by bios
    mov [drive_id], dl

    mov ah, 0x41
    mov bx, 0x55aa
    int 0x13
    jc disk_extension_not_supported
    cmp bx, 0xaa55
    jne disk_extension_not_supported

; Verify that long mode is supported
test_long_mode:
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb long_mode_not_supported
    mov eax, 0x80000001
    cpuid
    test edx, (1 << 29)
    jz long_mode_not_supported

; Load the stage1 bootloader. 
; This is done through the 0x13 BIOS interrupt, which provides hard disk and
; floppy disk read/write functions.
; https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)
load_stage1:
    mov si, load_stage1_packet
    mov dl, [drive_id]
    mov ah, 0x42
    int 0x13
    jc read_error
    mov dl, [drive_id]
    jmp 0x7e00

; Setup registers for error message pertaining to disk extensions
disk_extension_not_supported:
    mov bx, 0xa
    mov bp, disk_extension_err_msg
    mov cx, disk_extension_err_len
    jmp print_message

; Setup registers for error message pertaining to long mode support
long_mode_not_supported:
    mov bx, 0xa
    mov bp, long_mode_err_msg
    mov cx, long_mode_err_len
    jmp print_message

; Setup registers for error message pertaining to reading in stage1
read_error:
    mov bx, 0xa
    mov bp, load_stage1_err_msg
    mov cx, load_stage1_err_len

; Print out a message
print_message:
    mov ah, 0x13
    mov al, 0x1
    xor dx, dx
    int 0x10

; Exit the stage0 bootloader, should not be hit unless an error occurs. 
; Execution should instead be transferred to the stage1 bootloader
exit:
    hlt
    jmp exit

drive_id:               db 0
disk_extension_err_msg: db "Disk Extension is not supported, exiting..."
disk_extension_err_len: equ $-disk_extension_err_msg
long_mode_err_msg: db "Long mode not supported"
long_mode_err_len: equ $-long_mode_err_msg
load_stage1_err_msg: db "Error reading stage1 bootloader from disk"
load_stage1_err_len: equ $-load_stage1_err_msg

; Initialize the structure passed to BIOS 0x13 to read 2 sectors to physical
; address 0x1000
load_stage1_packet: istruc disk_address_packet_type
    at disk_address_packet_type.size, db        0x10
    at disk_address_packet_type.zero, db        0x0
    at disk_address_packet_type.num_sectors, dw 0x4
    at disk_address_packet_type.offset, dw      0x7e00
    at disk_address_packet_type.segment, dw     0x0
    at disk_address_packet_type.address_lo, dd  0x1
    at disk_address_packet_type.address_hi, dd  0x0
iend

times 0x1fe-($-$$) db 0
dw 0xaa55

incbin "stage1.bin"
