### VFuzz
- Bootloader
    - Launch rust bootloader in 64-bit mode with memory-maps as an argument
    - Query apic information
    - Split up memory maps into various cores and distribute evenly between them
    - Allocate a stack for each core, and launch each core into the kernel with their assigned
      mem-map and stack-space as arguments
- Kernel
    - Setup paging based on provided memory-regions (TODO: Watch gamozolabs paging stuff)
    - Setup memory allocator
    - MISC
        - Cores own hardware-features, so other cores requesting to use it send a msg to the 
          owner-core

### TODO
- ACPI
    - https://wiki.osdev.org/Symmetric_Multiprocessing
    - Find MADT table in the XSDT table to get a list of cores
    - Startup Sequence: send combination of init IPI and SIPI's for every core
    - Have cores call initialization routines for rm -> pm -> lm
    - Each core needs to start the next core otherwise this becomes slow on many-core systems
        - (https://stackoverflow.com/questions/16364817/how-to-use-the-apic-to-create-ipis-to-wake-the-aps-for-smp-in-x86-assembly)
    - Parse NUMA locality so memory can be allocated in an efficient manner
- Kernel Loading
    - Can just place it in memory right after bootloader from stage-1. Stage-2 can then parse/load
    it into proper location
- Memory
    - Every core needs its own address space
    - 
