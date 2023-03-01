//! ACPI - Advanced Configuration and Power Interface
//!     - This can be used for general power management and to manage peripherals

use crate::{read_phys, println, apic::get_apic_base};

use core::mem::size_of;
use either::Either;

use x86::{
    apic::{
        xapic,
        ApicControl,
        ApicId::XApic,
    }
};

/// Maximum number of cores we currently support getting up and running
pub const MAX_CORES: usize = 16;

/// List of apics found on the system
pub static mut APICS: [u32; MAX_CORES] = [0u32; MAX_CORES];

/// Number of apics found and stored in `apics`
pub static mut NUM_APICS: usize = 0;

/// APICs are launched sequentially from each booting processor. This global is used to keep track 
/// which APs we have already started
pub static mut CUR_APIC: usize = 0;

pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    /// Was unable to find the `RSDP PTR ` signature while iterating through the relevant regions
    RSDPSignatureNotFound,

    /// Invalid RSDP Version found, only `0` and `2` are valid
    InvalidVersion,

    /// RSDP Checksum check failed
    RSDPChecksum,

    /// RSDp Checksum check failed for extended variant
    RSDPExtendedChecksum,

    /// Invalid signature for RSDT
    InvalidRSDTSignature,

    /// Invalid table-size for RSDT, has to be a multiple of 4
    InvalidRSDTTableSize,

    /// Invalid signature for XSDT
    InvalidXSDTSignature,

    /// Invalid table-size for XSDT, has to be a multiple of 8
    InvalidXSDTTableSize,

    /// More cores found than currently supported. Increase `MAX_CORES` configurable
    TooManyCores,
    
    /// Each Apic entry in the MADT needs to have a size of 8-bytes
    InvalidAPICEntrySize,

    /// Each x2Apic entry in the MADT needs to have a size of 16-bytes
    Invalidx2APICEntrySize,

    /// The checksum calculation failed for some SDT entry
    SDTChecksum,
}

/// Root System Description Pointer
/// This table is used to locate the XSDT
/// https://wiki.osdev.org/RSDP
#[derive(Default, Copy, Clone)]
#[repr(C, packed)]
pub struct Rsdp {
    signature:         [u8; 8],
    checksum:          u8,
    oem_id:            [u8; 6],
    revision:          u8,
    rsdt_address:      u32,
}

/// Root System Description Pointer
/// This table is used to locate the XSDT
/// https://wiki.osdev.org/RSDP
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct RsdpExtended {
    first_part:        Rsdp,
    length:            u32,
    xsdt_address:      u64,
    extended_checksum: u8,
    reserved:          [u8; 3],
}

/// Some rsdt configuration flags, these are necessary since options like the entry-size can change
/// depending on the acpi version
#[derive(Default)]
struct RsdtConfig {
    entry_size:  usize,
    start_addr:  u64,
    num_entries: usize,
}

/// System Descripter Table header
/// All SDT's have this header + the actual sdt payload
#[repr(C, packed)]
pub struct SDTHeader {
    signature:        [u8; 4],
    length:           u32,
    revision:         u8,
    checksum:         u8,
    oem_id:           [u8; 6],
    oem_table_id:     [u8; 8],
    oem_revision:     u32,
    creator_id:       u32,
    creator_revision: u32,
}

impl SDTHeader {
    pub unsafe fn new(addr: u64) -> Result<Self> {
        let header = read_phys::<SDTHeader>(addr);

        // Verify checksum over the entire SDT (Header + entry)
        let sum = (addr..addr + header.length as u64)
            .fold(0u8, |acc, addr| {
                acc.wrapping_add(read_phys(addr as u64))
            });
        if sum != 0 { return Err(Error::SDTChecksum); }

        Ok(header)
    }
}

/// This struct contains various information we parsed out from the acpi table
pub struct ParsedACPI {
    /// Version of acpi running on this system
    pub version: usize,

    /// The parsed out rsdp or extended-rsdp
    rsdp: Either<Rsdp, RsdpExtended>,

    /// Some rsdt configuration options needed for further parsing
    rsdt_config: RsdtConfig,

}


impl ParsedACPI {
    /// Default acpi
    fn default() -> Self {
        Self {
            version: 0,
            rsdp: either::Left(Rsdp::default()),
            rsdt_config: RsdtConfig::default(),
        }
    }

    /// Find and parse all the acpi information we will be making use of
    pub unsafe fn parse() -> Result<Self> {
        let mut acpi = Self::default();

        // Parse out the rsdp
        acpi.parse_rsdp()?;

        // Setup some configurations we need to parse out RSDT
        acpi.rsdt_config()?;

        // Loop through the entries in the rsdt/xsdt and parse out tables we are interested in
        for i in 0..acpi.rsdt_config.num_entries {
            let addr = acpi.rsdt_config.start_addr + (i * acpi.rsdt_config.entry_size) as u64;
            let table_ptr = match acpi.rsdt_config.entry_size {
                4 => {
                    read_phys::<u32>(addr) as u64
                },
                8 => {
                    read_phys::<u64>(addr) as u64
                }
                _ => unreachable!(),
            };

            let sdt_header  = read_phys::<SDTHeader>(table_ptr);
            let start_addr  = table_ptr + size_of::<SDTHeader>() as u64;
            let end_addr    = table_ptr + sdt_header.length as u64;

            // APIC entry found, parse out all active cores
            if &sdt_header.signature == b"APIC" {
                acpi.parse_madt(start_addr, end_addr)?;
            }
        }
        Ok(acpi)
    }

    /// Parse out the rsdp
    unsafe fn parse_rsdp(&mut self) -> Result<()> {
        // Get address of extended bios data area and align it to 16-byte boundary
        let ebda: u64 = read_phys::<u16>(0x40e) as u64;
        let ebda = ebda & !(0x10 - 1);

        // The rsdp is located either in the first KiB of ebda or in the hardcoded address-range
        let rsdp_possible_ranges = [
            (ebda, ebda + 1024),
            (0x000E0000, 0x000FFFFF),
        ];

        // Iterate through the possible locations of the rsdp until the magic string is found
        // This SDP-entry can then be used to initialize the rsdp or rsdp_extended based on the acpi
        // version
        for range in rsdp_possible_ranges {
            let start = range.0;
            let end   = range.1;

            // Find the RDSP entry
            for addr in (start..end).step_by(0x10) {
                let local_rsdp = read_phys::<Rsdp>(addr);
                if &local_rsdp.signature == b"RSD PTR " {
                    // Verify checksum of the rsdp
                    let raw = read_phys::<[u8; size_of::<Rsdp>()]>(addr);
                    let sum = raw.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
                    if  sum != 0 { return Err(Error::RSDPChecksum); }

                    self.version = local_rsdp.revision as usize;

                    match self.version {
                        0 => {
                            // Version 0 means standard rsdp is used
                            self.rsdp = Either::Left(local_rsdp);
                            return Ok(());
                        },
                        2 => {
                            // If this is RSDP 2.0 or above we check the extended checksum as well
                            let local_extended_rsdp = read_phys::<RsdpExtended>(addr);
                            let raw = read_phys::<[u8; size_of::<RsdpExtended>()]>
                                (addr + size_of::<Rsdp>() as u64);
                            let sum = raw.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
                            if sum != 0 { return Err(Error::RSDPExtendedChecksum); }

                            // Version 2 means extended rsdp is used
                            self.rsdp = Either::Right(local_extended_rsdp);
                            return Ok(());
                        }
                        _ => {
                            return Err(Error::InvalidVersion);
                        }
                    }
                }
            }
        }
        Err(Error::RSDPSignatureNotFound)
    }

    /// Setup some rsdt configuration based on the version (rsdt vs xsdt)
    unsafe fn rsdt_config(&mut self) -> Result<()> {
        // Locate the rsdt and setup config values based on if we are using the rsdp or the xsdp
        let (start_addr, num_entries, entry_size) = match self.version {
            0 => {
                let entry_size  = 4;
                let rsdp        = self.rsdp.left().unwrap();
                let rsdt_header = SDTHeader::new(rsdp.rsdt_address as u64)?;

                if &rsdt_header.signature != b"RSDT" {
                    return Err(Error::InvalidRSDTSignature);
                }

                if ((rsdt_header.length as usize - size_of::<SDTHeader>()) % entry_size) != 0 {
                    return Err(Error::InvalidRSDTTableSize);
                }

                let start_addr  = rsdp.rsdt_address as u64 + size_of::<SDTHeader>() as u64;
                let num_entries = (rsdt_header.length as usize - size_of::<SDTHeader>()) /
                entry_size;
                (start_addr, num_entries, entry_size)
            }
            2 => {
                let entry_size    = 8;
                let rsdp_extended = self.rsdp.right().unwrap();
                let xsdt_header   = SDTHeader::new(rsdp_extended.xsdt_address)?;

                if &xsdt_header.signature != b"XSDT" {
                    return Err(Error::InvalidXSDTSignature);
                }

                if ((xsdt_header.length as usize - size_of::<SDTHeader>()) % entry_size) != 0 {
                    return Err(Error::InvalidXSDTTableSize);
                }

                let start_addr  = rsdp_extended.xsdt_address as u64 + size_of::<SDTHeader>() as u64;
                let num_entries = (xsdt_header.length as usize - size_of::<SDTHeader>()) /
                entry_size;
                (start_addr, num_entries, entry_size)
            }
            _ => unreachable!(),
        };

        self.rsdt_config = RsdtConfig {
            start_addr,
            num_entries,
            entry_size,
        };

        Ok(())
    }

    /// Parse out the MADT table located at `start_addr` - `end_addr`
    unsafe fn parse_madt(&mut self, start_addr: u64, end_addr: u64) -> Result<()> {
        let local_apic_base = read_phys::<u32>(start_addr);
        let _flags           = read_phys::<u32>(start_addr + 4);

        let mut records_ptr = start_addr + 8;
        loop {
            if (records_ptr + 2) > end_addr {
                break;
            }
            let entry_type: u8 = read_phys(records_ptr + 0);
            let entry_len:  u8 = read_phys(records_ptr + 1);

            // Out of range for madt table
            if (records_ptr + entry_len as u64) > end_addr {
                break;
            }

            const PROCESSOR_ENABLED: u32 = 1 << 0;
            const ONLINE_CAPABLE: u32 = 1 << 1;

            match entry_type {
                0 => {
                    // Processor Local APIC - Single logical processor + its local
                    // interrupt controller
                    if entry_len != 8 {
                        return Err(Error::InvalidAPICEntrySize);
                    }

                    let _proc_id: u8 = read_phys(records_ptr + 2);
                    let apic_id:  u8 = read_phys(records_ptr + 3);
                    let flags:   u32 = read_phys(records_ptr + 4);

                    // If the processor is enabled/can be enabled, record it
                    if (flags & PROCESSOR_ENABLED) != 0 || (flags & ONLINE_CAPABLE) != 0 {
                        if NUM_APICS >= MAX_CORES {
                            return Err(Error::TooManyCores);
                        }
                        APICS[NUM_APICS] = apic_id as u32;
                        NUM_APICS += 1;
                    }
                },
                9 => {
                    // Processor Local x2APIC - Single physical processor + its local x2APIC
                    // Pretty much identical to Local APIC, just used when core numbers are
                    // required that that don't fit into a u8
                    if entry_len != 16 {
                        return Err(Error::Invalidx2APICEntrySize);
                    }

                    let apic_id:  u32 = read_phys(records_ptr + 4);
                    let flags:    u32 = read_phys(records_ptr + 8);
                    let _acpi_id: u32 = read_phys(records_ptr + 12);

                    // If the processor is enabled/can be enabled, record it
                    if (flags & PROCESSOR_ENABLED) != 0 || (flags & ONLINE_CAPABLE) != 0 {
                        if NUM_APICS >= MAX_CORES {
                            return Err(Error::TooManyCores);
                        }
                        APICS[NUM_APICS] = apic_id as u32;
                        NUM_APICS += 1;
                    }
                },
                _ => {},
            }
            records_ptr += entry_len as u64;
        }
        Ok(())
    }

    pub unsafe fn launch_next_ap(&mut self) -> Result<()> {
        let base = get_apic_base();
        let regs: &'static mut [u32] = core::slice::from_raw_parts_mut(base as *mut _, 256);

        let mut xapic = xapic::XAPIC::new(regs);

        // If we have not yet initialized all cores startup the next core
        if CUR_APIC <= NUM_APICS {

            // If this is the BSP, move on to the next processor
            if xapic.bsp() {
                CUR_APIC += 1;
            }

            println!("Launching: {}", APICS[CUR_APIC]);

            xapic.ipi_init(XApic(1));
            //xapic.ipi_init(XApic(APICS[CUR_APIC] as u8));
            //xapic.ipi_startup(XApic(APICS[CUR_APIC] as u8), 0);
            //xapic.ipi_startup(XApic(APICS[CUR_APIC] as u8), 0);

            CUR_APIC += 1;
        }

        Ok(())
    }
}

