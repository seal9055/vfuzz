# vfuzz
Start date: June 2022

This is currently still in very early stages. If the project gets that far, it
is supposed to become a hypervisor based fuzzer.

Currently only have the beginnings of the bootloader

#### Memory Layout
```
0x00000000 : 0x000003FF - Real Mode IVT      [1024]
0x00000400 : 0x000004FF - BIOS data Area     [256]
0x00007C00 : 0x00007DFF - Stage-0 Bootloader [512]
0x00007E00 : 0x000085FF - Stage-1 Bootloader [512 * 4]
0x00010000 : 0x00017800 - Stage-2 Bootloader [512 * 60]
0x00080000 : 0x0009FFFF - ExtBIOS Data Area? [1024 * 128]
0x01000000 :    ...     - Kernel             [...]
```

#### Stage-0 Bootloader
This is the first code we run after the motherboard's bios transfers control to us. Only 512 bytes
are available so only extremely bare-bones initialization is handled.   

Its responsibilities include:
- Initialize segment registers
- Check for disk-extension-service support (used to initially load data from disk)
- Check for long mode support
- Load stage1 bootloader from disk and transfer execution to it

#### Stage-1 Bootloader
Now that we transferred out of the initial 512 bytes we have a little more space to work with. This
portion of the bootloader is provided with 512 * 4 bytes of memory and loaded at 0x7e00. 

Its responsibilities include:
- Enable a20 line to address >1MiB of memory
- Load Stage-2 bootloader from disk to 0x10000
- Use bios interrupts to detect available memory
- Enter 32-bit protected mode
- Setup initial page-tables and enter 64-bit long mode
- Transfer control to Stage-2 bootloader, passing in memory-maps as argument

#### Stage-2 Bootloader
This is the first part of this execution-chain that is written in rust instead of handwritten
assembly. It is provided with 1024 * 25 bytes of memory and loaded at 0x10000.

Its responsibilities include:
- Initialize serial/vga logging drivers
- Query acpi system to retrieve core-information
- Split memory maps between the cores so each core gets its own separate memory mappings
- Allocate a stack for each core
- Launch each core into the kernel with their assigned memory-mappings and stack-space as arguments

#### Kernel


#### Install Dependencies
```
sudo apt install -y nasm qemu-system-x86 lld
```

#### Usage
```
make
./run.sh
```

