use crate::println;

/// Main system description tabel. Extended version of RSDT
/// If the XSDT is valid, it must be used instead of the RSDT
/// Parsed out by finding RSDP and then use XsdtPointer
/// https://wiki.osdev.org/XSDT
#[repr(C, packed)]
struct Xsdt {
    signature:        [u8; 4],
    length:           u32,
    revision:         u8,
    checksum:         u8,
    oemid:            [u8; 6],
    oem_table_id:     [u8; 8],
    oem_revision:     u32,
    creator_id:       u32,
    creator_revision: u32,
}

#[repr(C, packed)]
struct Rsdp {
    signature:         [u8; 8],
    checksum:          u8,
    oem_id:            [u8; 6],
    revision:          u8,
    rsdt_address:      u32,
    length:            u32,
    xsdt_addr:         u64,
    extended_checksum: u8,
    reserved:          [u8; 3],
}

pub fn init() {
    println!("ACPI init hit");

}
