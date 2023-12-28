#![no_main]
#![no_std]

#![allow(unused_parens)]
#![feature(array_chunks)]

extern crate arrayvec;

use bootloader_lib::*;

use uefi::prelude::*;
use uefi::table::boot::MemoryType;
use uefi_services::println;

use core::arch::asm;

pub mod graphics;
pub mod kernel;
pub mod memory;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
	uefi_services::init(&mut system_table).expect("Failed to initialise UEFI services");
	system_table.stdout().clear().expect("Failed to clear screen");

	println!("Eisen UEFI Bootloader...");

	let bs: &BootServices = system_table.boot_services();

	let mut kernel_entry_addr: usize = 0;
	kernel::find_kernel(bs, &mut kernel_entry_addr).expect("Failed to find kernel");
	
	let _mmap						= memory::init_memory(bs).expect("Failed to read UEFI memory map");
	let graphics_info				= graphics::init_graphics(bs).expect("Failed to initialise graphics");
	let (runtime_systable, mmap)	= system_table.exit_boot_services(MemoryType::LOADER_DATA);

	let bi: BootInfo = BootInfo {
		graphics_info:		graphics_info,
		mmap:				mmap.entries().copied().collect(),
		runtime_systable:	runtime_systable
	};

	unsafe { asm!("call {}", in(reg) kernel_entry_addr, in("rdi") &bi); }

	loop {}

	#[allow(unreachable_code)]
	Status::SUCCESS
}