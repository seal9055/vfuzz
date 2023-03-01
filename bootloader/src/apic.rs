//! Advanced Programmable Interrupt Controller
//!     - Manages IRQ lines (Can extend the traditional 16 that PIC handles to 24)
//!     - Manages CPUs

use crate::println;
use x86::{
    cpuid::CpuId,
    msr, io,
};

/// Physical address we want the local APIC to be mapped at
const APIC_BASE: u64 = 0xfee0_0000;

pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    NoApicSupport,
    Nox2ApicSupport,
}

pub struct Apic {

}

// reg contains apic base

impl Apic {
    /// Initialize processors local APIC
    pub unsafe fn init() -> Result<()> {
        let features = CpuId::new().get_feature_info().unwrap();
        if !features.has_apic() {
            return Err(Error::NoApicSupport);
        }

        if !features.has_x2apic() {
            println!("No x2APIC");
            //return Err(Error::Nox2ApicSupport);
        }

        // Retrieve apic base
        let apic_offset = get_apic_offset();

        // Or in the APIC base that we want to use
        let new_apic = APIC_BASE | apic_offset;

        // Enable the xAPIC unconditionally
        let new_apic = new_apic | (1 << 11);

        // Disable the PIC by masking off all interrupts
        io::outb(0xa1, 0xff);
        io::outb(0x21, 0xff);

        // Reprogram APIC
        msr::wrmsr(msr::IA32_APIC_BASE, new_apic);

        // For xAPIC mode, virtually map in APIC physical memory
        //if !features.has_x2apic() {

        //}

        Ok(())
    }
}

/// Get the physical base address of the APIC registers page
pub unsafe fn get_apic_base() -> u64 {
    let orig_ia32_apic_base = msr::rdmsr(msr::IA32_APIC_BASE);
    orig_ia32_apic_base & !0xfff
}

/// Get the physical address of the APIC registers page offset
unsafe fn get_apic_offset() -> u64 {
    let orig_ia32_apic_base = msr::rdmsr(msr::IA32_APIC_BASE);
    orig_ia32_apic_base & 0xfff
}
