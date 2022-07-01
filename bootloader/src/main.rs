#![no_std]
#![no_main]

use bootloader::println;
use core::panic::PanicInfo;

static X: u64 = 3;

#[no_mangle]
/// Entry-point of the stage2 bootloader
pub extern "C" fn entry(_arg1: u64) -> ! {
    // Next steps:
    // Figure out how to load the stage2 elf binary onto disk
    // Figure out how to call into the rust code

    println!("Entered rust part of bootloader: {}", X);
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

