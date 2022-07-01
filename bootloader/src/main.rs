#![no_std]
#![no_main]

use bootloader::println;
use core::panic::PanicInfo;

use core::sync::atomic::{AtomicUsize, Ordering};

const X: usize = 5;

#[no_mangle]
/// Entry-point of the stage2 bootloader
pub extern "C" fn entry(arg1: u32) -> ! {
    println!("Entered rust part of bootloader: {} : {}", arg1, X);

    static CORE_IDS: AtomicUsize = AtomicUsize::new(0);

    let core_id = CORE_IDS.fetch_add(1, Ordering::SeqCst);

    // If this is the first core booting up
    if core_id == 0 {

        // Initialize memory

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

