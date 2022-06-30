#![no_std]
#![no_main]

use core::panic::PanicInfo;
use bootloader::println;

/// Executes hlt instruction in a loop which stops the cpu until a new interrupt
/// is received
pub fn hlt_loop() -> ! {
    loop {
    }
}

#[panic_handler]
/// Panic handler
fn panic(_info: &PanicInfo) -> ! {
    hlt_loop();
}

#[no_mangle]
/// Entry-point of the stage2 bootloader
pub extern "C" fn _start() -> ! {
    // Next steps:
        // Figure out how to load the stage2 elf binary onto disk
        // Figure out how to call into the rust code

    println!("Entered rust part of bootloader");
    hlt_loop();
}
