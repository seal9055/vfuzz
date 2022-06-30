use std::process::Command;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 && args[1] == "clean" {
        println!("Cleaning up directories");

        if Path::new("vfuzz.boot").exists() {
            std::fs::remove_file("vfuzz.boot")
                .expect(&format!("Failed to remove {}", "vfuzz.boot"));
        }
        if Path::new("stage1.bin").exists() {
            std::fs::remove_file("stage1.bin")
                .expect(&format!("Failed to remove {}", "stage1.bin"));
        }
    } else {
        // Stage1 is assembled before stage0, so that stage0 can read in the 
        // bytes of stage1 using the `incbin` macro
        println!("Assembling stage1");
        let stage1_status = Command::new("nasm")
            .args(&["-f", "bin", "-o", "stage1.bin", 
                  "bootloader/src/stage1.asm"])
            .status()
            .expect("Failed to invoke NASM for stage1");
        assert!(stage1_status.success(), "Failed to assemble bootloader");

        println!("Assembling stage0");
        let stage0_status = Command::new("nasm")
            .args(&["-f", "bin", "-o", "vfuzz.boot", 
                  "bootloader/src/stage0.asm"])
            .status()
            .expect("Failed to invoke NASM for stage0");
        assert!(stage0_status.success(), "Failed to assemble bootloader");

        println!("Removing intermediate files");
        std::fs::remove_file("stage1.bin")
            .expect(&format!("Failed to remove {}", "stage0.bin"));
    }
}
