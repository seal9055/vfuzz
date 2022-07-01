#![no_std]
#![no_main]

use bootloader::println;
use core::panic::PanicInfo;

#[no_mangle]
/// Entry-point of the stage2 bootloader
pub extern "C" fn entry(arg1: u64) -> ! {
    let x = 3;
    println!("Entered rust part of bootloader");
    println!("Entered rust part of bootloader: {}", arg1);
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

