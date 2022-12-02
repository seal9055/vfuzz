#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct E820Entry {
    pub base: u64,
    pub length:  u64,
    pub typ:  u32,
}

#[derive(Debug)]
pub struct MMU {
    pub free_mem_region: [E820Entry; 50],
    pub count: i32,
}

impl MMU {
    pub fn init(_addr: usize) -> Option<MMU> {
        /*
        let count: i32 = unsafe { *(addr as *const i32) };
        let mut _total_mem: u64 = 0;
        let mem_map: &[E820Entry] = unsafe { &*((addr+8) as *mut [E820Entry; 50]) };
        let mut free_region_count: u32 = 0;

        let mut mmu = MMU { free_mem_region: [E820Entry::default(); 50], count };

        assert!(count <= 50);

        for i in 0..count {
            let ii = i as usize;

            if mem_map[ii].typ == 1 {
                //mmu.free_mem_region[free_region_count as usize].addr = mem_map[ii].base;
                //mmu.free_mem_region[free_region_count as usize].len = mem_map[ii].length;
                //_total_mem += mem_map[ii].length;
                //free_region_count += 1;
            }
        }
        mmu
        */
        None
    }
}
