#![no_std]
#![no_main]

use bootloader::{
    println, mm, apic,
    acpi::{
        self,
        NUM_APICS,
        CUR_APIC,
    },
};

use core::panic::PanicInfo;

#[repr(packed, C)]
pub struct MemLayout {
    num_entries: u64,
    mem_layout: [mm::E820Entry; 32],
}

//static mut V: [u8; 4096] = [0u8; 4096];

#[no_mangle]
/// Entry-point of the stage2 bootloader
pub extern "C" fn entry(arg1: &MemLayout) -> ! {
    println!("Entered rust part of bootloader");
    assert!(arg1.num_entries < 32, "Too many memory regions found");

    //for i in 0..arg1.num_entries {
    //    let i = i as usize;
    //    println!("[{:0>16X}:{:0>16X}] - {}", {arg1.mem_layout[i].base}, 
    //             arg1.mem_layout[i].base + arg1.mem_layout[i].length, {arg1.mem_layout[i].typ}); 
    //}

    // For some reason unwrapping here causes a segfault. Matching like this works though
    let apic = unsafe { apic::Apic::init() };
    let _apic = match apic {
        Ok(v) => v,
        Err(v) => panic!("{:?}", v),
    };
    
    // For some reason unwrapping here causes a segfault. Matching like this works though
    let acpi = unsafe { acpi::ParsedACPI::parse() };
    let mut acpi = match acpi {
        Ok(v) => v,
        Err(v) => panic!("{:?}", v),
    };

    //unsafe { println!("Done parsing acpi({}), found {} cores", acpi.version, NUM_APICS); }
    //unsafe { println!("{}", CUR_APIC); }

    let _ = unsafe { acpi.launch_next_ap() };


    // If this is the first core booting up
    //if ApicControl::bsp() {
    //for i in 0..NUM_APICS {
    //    // Initialize memory
    //        // Load memory map

    //    // Download kernel

    //    // Setup page tables

    //    // Load the kernel

    //    // for each core {
    //        // allocate stack
    //        // 
    //}

    // launch kernel[core_id]

    println!("Done with stage2");

    hlt_loop();
}

/// Executes hlt instruction in a loop which stops the cpu until a new interrupt
/// is received
pub fn hlt_loop() -> ! {
    loop {}
}

#[panic_handler]
/// Panic handler
fn panic(info: &PanicInfo) -> ! {
    println!("{}", *info);
    hlt_loop();
}

