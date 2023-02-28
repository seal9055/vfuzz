[bits 16]
[org 0x7e00]

; Stage 1 bootloader. 512 * 4 bytes of space. 
; Loads the stage2 bootloader (rust part), retrieves the memory map, enables the a0 line and
; enters protected & long mode

; Contains information such as the offset and size of the stage2 bootloader
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

; Entrypoint of stage1 bootloader
start:
    ; Save disk id
    mov [drive_id], dl

; Enable a20 line. Without it, the address space > 1mb cannot be addressed.
; This disables the memory wrap around mechanism present in real mode
; http://therx.sourceforge.net/osdev/files/ia32_rm_addr.pdf
enable_a20:
    in al, 0x92
    or al, 2
    out 0x92, al

; Load the stage2 bootloader from disk to memory (rust portion of bootloader)
load_stage2:
    mov si, load_stage2_packet
    mov dl, [drive_id]
    mov ah, 0x42
    int 0x13
    jc read_error

; Retrieve the memory layout to determine what space we are free to use. This structure is then
; pushed onto the stack before jumping into rust code, so the rust portion of the bootloader can
; make use of this information to setup the initial memory manager
;
; https://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15.2C_EAX_.3D_0xE820
retrieve_memory_layout:
    mov dword[E820Entries], 0        ; Initialize size field to 0

    mov eax, 0xe820
    xor ebx, ebx
    mov ecx, 20
    mov edx, 0x534d4150
    mov edi, E820Entries+8
    int 0x15
    jc mem_layout_err

; This will loop until all memory has been mapped out, at which point 
; `get_mem_completed` is called
get_mem_info:
    inc dword[E820Entries]
    test ebx, ebx
    jz get_mem_completed

    mov eax, 0xe820
    mov ecx, 20
    mov edx, 0x534d4150
    add edi, 20
    int 0x15
    jnc get_mem_info

; Retrieving memory layout successfuly completed
get_mem_completed:
    
; Swap the processor to protected mode
enable_protected_mode:
    mov eax, cr0
    or al, 1
    mov cr0, eax

; Load gdt and idt tables we set up in the structures area of the code into the 
; corresponding rgdt & ridt registers
load_gdt_idt:
    lgdt [gdt]
    lidt [idt]

    ; Jump to protected mode
    jmp 0x0008:pm_entry

; Error Handling
; ------------------------------------------------------------------------------

; Setup registers for error message pertaining to reading in the stage2 bootloader
read_error:
    mov bx, 0xa
    mov bp, load_stage2_err_msg
    mov cx, load_stage2_err_len
    jmp print_message

; Setup registers for error message pertaining to retrieving the memory layout
mem_layout_err:
    mov bx, 0xa
    mov bp, memory_layout_err_msg
    mov cx, memory_layout_err_len

; Print out a message
print_message:
    mov ah, 0x13
    mov al, 0x1
    xor dx, dx
    int 0x10

; Exit the stage1 bootloader
exit:
    hlt
    jmp exit

; ------------------------------------------------------------------------------

[bits 32]

; Entry when first transitioning into protected mode
pm_entry:
    ; Set up the protected-mode data segment registers
    mov ax, 0x10    ; Data segment selector
    mov ds, ax      ; DS: Data segment
    mov es, ax      ; ES: Extra Segment
    mov fs, ax      ; SS: Stack Segment
    mov gs, ax      
    mov ss, ax

    ; Setup stack
    mov esp, 0x7c00

    ; Setup page tables 
    cld
    mov edi, 0x80000
    xor eax, eax
    mov ecx, 0x10000 / 4
    rep stosd

    mov dword[0x80000], 0x81007
    mov dword[0x81000], 0b10000111

    ; Load 64-bit gdt
    lgdt [gdt64]

    ; Enable 64-bit mode. This requires setting cr4:5, and loading the 
    ; previously setup paging structure into cr3
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax
    mov eax, 0x80000
    mov cr3, eax

    ; Enable longmode
    mov ecx, 0xc0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    ; Enable Paging
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    ; Jump to longmode entry
    jmp 0x0008:lm_entry

[bits 64]
 
; Entry when first transitioning to 64-bit long mode
lm_entry:
     mov rsp, 0x7c00
 
; The rust portion of the bootloader was simply appended to stage1. This means
; that it still needs to be written to the correct memory locations. It comes
; alongside some metadata describing the amount of sections and each sections
; vaddr/size
run_stage2:
	; Zero out entire range where the bootloader is loaded [0x10000, 0x20000)
    ; Ram is not necessarily 0 initialized, so this makes sure that memory is
    ; not already pre-initialized, which could cause issues
	mov edi, 0x10000
	mov ecx, 0x20000 - 0x10000
	xor eax, eax
	rep stosb

    mov eax, [rust_entry]       ; Num_sections
    lea edx, [rust_entry + 4]   ; Initialize edx to start of first struct

.loop:
    test eax, eax
    jz short .end

    mov edi, [edx]      ; Vaddr
    mov ecx, [edx + 4]  ; Size
    lea esi, [edx + 8]  ; Raw data
    add edx, ecx        ; Increment pointer by size of current chunk
    add edx, 8
    rep movsb           ; memcpy(edi, esi, ecx)

    dec eax
    jmp short .loop

.end:
    mov rdi, E820Entries
    call 0x10000

l_end:
    hlt
    jmp l_end

; Structures
; ------------------------------------------------------------------------------

drive_id: db 0
load_stage2_err_msg: db "Error reading stage2 bootloader from disk"
load_stage2_err_len: equ $-load_stage2_err_msg
memory_layout_err_msg: db "Error retrieving memory layout information"
memory_layout_err_len: equ $-memory_layout_err_msg

; Initialize the structure passed to BIOS 0x13 to read stage2 from disk
load_stage2_packet: istruc disk_address_packet_type
    at disk_address_packet_type.size, db        0x10
    at disk_address_packet_type.zero, db        0
    at disk_address_packet_type.num_sectors, dw 60
    at disk_address_packet_type.offset, dw      0x8600
    at disk_address_packet_type.segment, dw     0
    at disk_address_packet_type.address_lo, dd  0x5
    at disk_address_packet_type.address_hi, dd  0x0
iend

E820Entries times (20 * 33) db 0

; 32-bit protected mode gdt
; ------------------------------------------------------------------------------

align 8
gdt_base:
    dq 0x0000000000000000 ; 0x0000 | Null descriptor
    ; Code
    dw 0xffff       ; limit 0:15
    dw 0x0000       ; base 0:15
    db 0x00         ; base 16:23
    db 0x9a         ; access P=1, DPL=0, S=1, TYPE=1010 
    db 0xcf         
    db 0x00         ; base 24:31
    ; Data
    dw 0xffff       ; limit 0:15
    dw 0x0000       ; base 0:15
    db 0x00         ; base 16:23
    db 0x92         ; access P=1, DPL=0, S=1, Type=1010
    db 0xcf     
    db 0x00         ; base 24:31

gdt:
	dw (gdt - gdt_base) - 1
	dd gdt_base

idt: dw 0
     dd 0

; 64-bit protected mode gdt
; ------------------------------------------------------------------------------

gdt64_base:
    dq 0x0000000000000000 ; 0x0000 | Null descriptor
    ; Code
    dw 0x0000       ; limit 0:15
    dw 0x0000       ; base 0:15
    db 0x00         ; base 16:23
    db 0x98         ; access P=1, DPL=0, S=1, TYPE=1010 
    db 0x20         
    db 0x00         ; base 24:31
    dq 0x0000920000000000 ; Present, data r/w

gdt64: 
	dw (gdt64 - gdt64_base) - 1
	dd gdt64_base

; ------------------------------------------------------------------------------

; 5 because we need to include the size of stage0 (512 bytes) in this
times (512 * 5)-($-$$) db 0

; Load the stage2 bootloader onto disk. This part of the bootloader is written in rust
rust_entry:
incbin "flattened_stage2.bin"
