#![no_std]
#![no_main]

use bootloader::println;
use bootloader::mm;
use core::panic::PanicInfo;

use core::sync::atomic::{AtomicUsize, Ordering};

#[repr(packed, C)]
pub struct MemLayout {
    num_entries: u64,
    mem_layout: [mm::E820Entry; 32],
}

#[no_mangle]
/// Entry-point of the stage2 bootloader
pub extern "C" fn entry(arg1: &MemLayout) -> ! {
    println!("Entered rust part of bootloader");
    assert!(arg1.num_entries < 32, "Too many memory regions found");

    for i in 0..arg1.num_entries {
        let i = i as usize;
        println!("[{:#0X}:{:#0X}] - {}", arg1.mem_layout[i].base, 
                 arg1.mem_layout[i].base + arg1.mem_layout[i].length, arg1.mem_layout[i].typ); 
    }

    static CORE_IDS: AtomicUsize = AtomicUsize::new(0);

    let core_id = CORE_IDS.fetch_add(1, Ordering::SeqCst);

    // If this is the first core booting up
    if core_id == 0 {

        // Initialize memory
            // Load memory map

        // Download kernel

        // Setup page tables

        // Load the kernel

        // for each core {
            // allocate stack
            // 

    }

    // launch kernel[core_id]

    hlt_loop();
}

/// Executes hlt instruction in a loop which stops the cpu until a new interrupt
/// is received
pub fn hlt_loop() -> ! {
    loop {}
}

#[panic_handler]
/// Panic handler
fn panic(_info: &PanicInfo) -> ! {
    hlt_loop();
}

