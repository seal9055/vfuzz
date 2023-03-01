### TODO
- ACPI
    - Initialize local APIC
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

### ACPI - Advanced Configuration and Power Interface
    - This is used for general power management and to manage peripherals
    - It also contains the RSDP, RSDT & MADT tables that we need to parse out APICS

### APIC - Advanced Programmable Interrupt Controller
    - Manages IRQ lines (Can extend the traditional 16 that PIC handles to 24)
    - PIC needs to be disabled to use this and interrupts need to be mapped to start at 32 instead
      of 0
    - LAPIC
        - Detecting if a CPU has a built-in local APIC is done using CPUID.01h:EDX[bit 9]
         - Each CPU is made of a "core" and a "local APIC"
            - LAPIC handles cpu-specific interrupts configuration and contains LVT
    - I/O APIC
        - Multi-processor Interrupt management, used to send interrupts to LAPICs
    - Inter-Processor Interrupts (IPIs) are used by local APICs to signal other APICS

