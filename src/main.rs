use elfparser;

fn flatten_bootloader(filename: &str) -> Vec<(usize, usize, Vec<u8>)> {
    let stage1 = std::fs::read(filename).unwrap();
    let elf = elfparser::ELF::parse_elf(&stage1);
    let mut sections = Vec::new();

    for phdr in elf.program_headers {
        // If this header is not loadable, skip it
        if phdr.seg_type != 0x1 {
            continue;
        }

        let raw_data = stage1[phdr.offset..phdr.offset + phdr.memsz]
            .to_vec().clone();

        assert_eq!(raw_data.len(), phdr.memsz, 
                   "Error reading in raw data for a program header");

        sections.push((phdr.vaddr, phdr.memsz, raw_data));
    }
    sections
}

fn main() {
    let sections = flatten_bootloader(
        "./bootloader/target/bootloader_config/release/bootloader");

    let mut bytes: Vec<u8> = Vec::new();

    // Write the number of loadable sections to the file
    let num_sections = (sections.len() as u32).to_le_bytes();
    bytes.extend_from_slice(&num_sections);

    for (vaddr, memsz, raw) in sections {
        bytes.extend_from_slice(&(vaddr as u32).to_le_bytes());
        bytes.extend_from_slice(&(memsz as u32).to_le_bytes());
        bytes.extend_from_slice(&raw);
    }

    let filler = vec![0; (512 * 101) - bytes.len()];
    bytes.extend_from_slice(&filler);

    std::fs::write("flattened_stage2.bin", bytes)
        .expect("Failed to write flattened bootloader to disk");
}
