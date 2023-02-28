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

        let mut raw_data = stage1[phdr.offset..phdr.offset + phdr.filesz]
            .to_vec().clone();

        // Pad with nullbytes if the section has a smaller size on disk than
        // in memory
        let filler = vec![0; phdr.memsz - raw_data.len()];
        raw_data.extend_from_slice(&filler);
        assert_eq!(phdr.memsz, raw_data.len());

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

    assert!(sections.len() != 0, "No sections found in parsed binary");

    // Write the number of loadable sections to the file
    let num_sections = (sections.len() as u32).to_le_bytes();
    bytes.extend_from_slice(&num_sections);

    for (vaddr, memsz, raw) in sections {
        bytes.extend_from_slice(&(vaddr as u32).to_le_bytes());
        bytes.extend_from_slice(&(memsz as u32).to_le_bytes());
        bytes.extend_from_slice(&raw);
    }

    assert!(bytes.len() < 512 * 60, "stage2 bootloader too large: {}", bytes.len());
    println!("Stage-2 Bootloader Size: {:#0x?}", bytes.len());
    let filler = vec![0; (512 * 60) - bytes.len()];
    bytes.extend_from_slice(&filler);

    std::fs::write("flattened_stage2.bin", bytes)
        .expect("Failed to write flattened bootloader to disk");
}
