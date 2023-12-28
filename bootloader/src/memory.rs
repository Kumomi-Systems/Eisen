extern crate alloc;

use uefi::prelude::*;
use uefi::Result;
use uefi::table::boot::{MemoryDescriptor, MemoryMap, MemoryMapSize};
use uefi_services::println;

use alloc::boxed::Box;
use alloc::vec;
use arrayvec::ArrayVec;

pub fn init_memory(bs: &BootServices) -> Result<ArrayVec<MemoryDescriptor, 256>, ()> {
	let mmap_size_max : MemoryMapSize = bs.memory_map_size();
	let mmap_storage: &mut [u8] = Box::leak(vec![0; mmap_size_max.map_size + 10 * mmap_size_max.entry_size].into_boxed_slice());
	let mmap: MemoryMap = bs.memory_map(mmap_storage).expect("Failed to get UEFI memory map");

	println!("Read UEFI Memory Map.");
	
	Ok(mmap.entries().copied().collect())
}