extern crate alloc;

use alloc::slice;
use uefi::{prelude::*, Error};
use uefi::proto::media::file::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::Result;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams, AllocateType, MemoryType, PAGE_SIZE};
use uefi_services::println;

use xmas_elf::{header::*, program};
use xmas_elf::P64;
use xmas_elf::program::ProgramHeader64;

use alloc::{vec, vec::Vec};
use core::mem::{size_of, transmute};

pub fn find_kernel(bs: &BootServices, kernel_entry_addr: &mut usize) -> Result {
	let sfs_handle = bs.get_handle_for_protocol::<SimpleFileSystem>()?;
	let mut sfs;
	unsafe {
		sfs = bs.open_protocol::<SimpleFileSystem>(
			OpenProtocolParams{
				handle: sfs_handle,
				agent: bs.image_handle(),
				controller: None
			},
			OpenProtocolAttributes::GetProtocol
		)?;
	}

	// Locate kernel file and guarantee it is a file
	let mut rootdir: Directory = sfs.open_volume()?;
	let kernel_handle_uncertain: FileHandle = rootdir.open(
		cstr16!("sys\\kernel\\kernel"),
		FileMode::Read,
		FileAttribute::empty()
	)?;
	if !kernel_handle_uncertain.is_regular_file()? {
		return Err(uefi::Error::new(Status::ABORTED, ()))
	}
	let mut kernel_handle: RegularFile = kernel_handle_uncertain.into_regular_file().unwrap();

	// Read ELF header of kernel
	let mut kernel_elf_header_pt1_buf = [0; size_of::<HeaderPt1>()];
	let mut kernel_elf_header_pt2_buf = [0; size_of::<HeaderPt2_<P64>>()];
	let kernel_elf_header_ident: HeaderPt1;
	let kernel_elf_header_extra: HeaderPt2_<P64>;
	kernel_handle.set_position(0)?;
	kernel_handle.read(kernel_elf_header_pt1_buf.as_mut())?;
	kernel_handle.set_position(size_of::<HeaderPt1>() as u64)?;
	kernel_handle.read(kernel_elf_header_pt2_buf.as_mut())?;
	unsafe {
		kernel_elf_header_ident = transmute::<[u8; size_of::<HeaderPt1>()], HeaderPt1>(kernel_elf_header_pt1_buf);
		kernel_elf_header_extra = transmute::<[u8; size_of::<HeaderPt2_<P64>>()], HeaderPt2_<P64>>(kernel_elf_header_pt2_buf);
	}

	// Validate ELF header of kernel
	if(
		kernel_elf_header_ident.magic					!= [0x7F, 0x45, 0x4C, 0x46]	||
		kernel_elf_header_ident.class.as_class()		!= Class::SixtyFour			||
		kernel_elf_header_ident.data.as_data()			!= Data::LittleEndian		||
		kernel_elf_header_extra.type_.as_type()			!= Type::Executable			||
		kernel_elf_header_extra.machine.as_machine()	!= Machine::X86_64			||
		kernel_elf_header_extra.version					!= 1
	) {
		return Err(Error::new(Status::ABORTED, ()));
	}

	// Load the kernel
	let mut kernel_program_headers_buf = vec![0 as u8; (kernel_elf_header_extra.ph_entry_size * kernel_elf_header_extra.ph_count) as usize];
	kernel_handle.set_position(kernel_elf_header_extra.ph_offset)?;
	kernel_handle.read(kernel_program_headers_buf.as_mut_slice())?;
	let mut kernel_program_headers: Vec<ProgramHeader64> = Vec::with_capacity(kernel_elf_header_extra.ph_count as usize);
	unsafe {
		const PROGRAM_HEADER_64_SIZE: usize = size_of::<ProgramHeader64>();
		for kernel_program_header in kernel_program_headers_buf.array_chunks::<PROGRAM_HEADER_64_SIZE>() {
			kernel_program_headers.push(
				transmute::<[u8; PROGRAM_HEADER_64_SIZE], ProgramHeader64>(*kernel_program_header)
			);
		}
	}
	for phdr in kernel_program_headers {
		if phdr.get_type().unwrap() == program::Type::Load {
			bs.allocate_pages(
				AllocateType::Address(phdr.physical_addr),
				MemoryType::LOADER_DATA,
				(phdr.mem_size as usize + PAGE_SIZE - 1) / PAGE_SIZE
			)?;
			kernel_handle.set_position(phdr.offset)?;
			unsafe {
				kernel_handle.read(
					slice::from_raw_parts_mut(
						phdr.physical_addr as *mut u8,
						((phdr.mem_size as usize + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE
					)
				)?;
			}
			break;
		}
	}

	*kernel_entry_addr = kernel_elf_header_extra.entry_point as usize;

	println!("Successfully loaded kernel.");
	
	Ok(())
}