#![no_std]

pub mod vga_buffer;
pub mod mm;
pub mod acpi;
pub mod apic;

pub unsafe fn read_phys<T>(addr: u64) -> T {
    core::ptr::read_volatile((addr) as *mut T)
}

pub unsafe fn write_phys<T>(addr: u64, val: T) {
    core::ptr::write_volatile((addr) as *mut T, val);
}
