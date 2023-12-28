#![no_std]

extern crate arrayvec;

use uefi::prelude::*;
use uefi::proto::console::gop::*;
use uefi::table::boot::*;
use uefi::table::Runtime;

use arrayvec::ArrayVec;

pub struct GraphicsInfo {
	pub mode: u32,
	pub res_h: usize,
	pub res_v: usize,
	pub base: *mut u8,
	pub size: usize,
	pub format: PixelFormat,
	pub bitmask: PixelBitmask
}

pub struct BootInfo {
	pub graphics_info: GraphicsInfo,
	pub mmap: ArrayVec<MemoryDescriptor, 256>,
	pub runtime_systable: SystemTable<Runtime>
}
