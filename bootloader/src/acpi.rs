use crate::{
    println, read_phys,
};

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

/// Root System Description Pointer
/// This table is used to locate the XSDT
/// https://wiki.osdev.org/RSDP
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

pub unsafe fn init() {
    println!("ACPI init hit");

    let ebda: u64 = read_phys::<u16>(0x40e) as u64;
    assert_eq!(ebda % 0x10, 0, "ebda not aligned on 0x10 boundary");
    
    // The rsdp is located either in the first KiB of ebda or in the hardcoded address-range
    let rsdp_possible_ranges = [
        (ebda, ebda + 1024),
        (0x000E0000, 0x000FFFFF),
    ];

    for range in rsdp_possible_ranges {
        let start = range.0;
        let end   = range.1;

        for addr in (start..end).step_by(0x10) {
            let rsdp = read_phys::<Rsdp>(addr);
            if &rsdp.signature == b"RSD PTR " {
                println!("FOUND: {}", rsdp.revision);
                continue;
            }
        }
    }
}
