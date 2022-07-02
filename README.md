# vfuzz
Start date: June 2022

This is currently still in very early stages. If the project gets that far, it
is supposed to become a hypervisor based fuzzer.

Currently only have the beginnings of the bootloader

#### Memory Layout
```
0x00000000 : 0x000003FF - Real Mode IVT     [1024]
0x00000400 : 0x000004FF - BIOS data Area    [256]
0x00007C00 : 0x00007DFF - Stage0 Bootloader [512]
0x00007E00 : 0x000085FF - Stage1 Bootloader [2048]
0x00010000 : 0x0001c800 - Stage2 Bootloader [50 * 1024]
0x00080000 : 0x0009FFFF - ExtBIOS Data Area [1024 * 128]
0x01000000 :    ...     - Kernel            [...]
```

#### Install Dependencies
```
sudo apt install -y nasm qemu-system-x86 lld
```

#### Usage
```
make
./run.sh
```

