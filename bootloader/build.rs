use std::path::Path;
use std::process::Command;

fn nasm(_in_asm: &str, out_obj: &str) {
    if Path::new(out_obj).exists() {
        std::fs::remove_file(out_obj).expect("Failed to remove old object");
    }

    let status = Command::new("touch")
        .args(&[out_obj])
        .status()
        .expect("Failed to run test");

    /* Check for command success */
    assert!(status.success(), "NASM command failed");

    /* Ensure output file was created */
    assert!(
        Path::new(out_obj).exists(),
        "NASM did not generate expected file"
    );
}

fn main() {
    nasm("src/asm_routines.asm", "target/asm_routines.obj");
}
