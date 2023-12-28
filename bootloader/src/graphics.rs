use bootloader_lib::GraphicsInfo;

use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat, PixelBitmask};
use uefi::Result;
use uefi::table::boot::{OpenProtocolParams, OpenProtocolAttributes};
use uefi_services::system_table;

const MAX_RESOLUTION: usize = 6000 * 4000;

pub fn init_graphics(bs: &BootServices) -> Result<GraphicsInfo, ()> {
	let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>()?;
	let mut gop;
	unsafe {
		gop = bs.open_protocol::<GraphicsOutput>(
			OpenProtocolParams{
				handle: gop_handle,
				agent: bs.image_handle(),
				controller: None
			}, 
			OpenProtocolAttributes::GetProtocol
		)?;
	}

	let mut graphics_info: GraphicsInfo = GraphicsInfo {
		mode: 0,
		res_h: 1,
		res_v: 1,
		base: 0 as *mut u8,
		size: 0,
		format: PixelFormat::Rgb,
		bitmask: PixelBitmask { red: 0, green: 0, blue: 0, reserved: 0 }
	};

	for (idx, mode) in gop.modes(bs).enumerate() {
		let res_h = mode.info().resolution().0;
		let res_v = mode.info().resolution().1;
		let mode_resolution = res_h * res_v;
		if (
			mode_resolution <= MAX_RESOLUTION &&
			mode_resolution > graphics_info.res_h * graphics_info.res_v &&
			(res_h << 7) / res_v >= (graphics_info.res_h << 7) / graphics_info.res_v
		) {
			graphics_info.mode = idx as u32;
			graphics_info.res_h = res_h;
			graphics_info.res_v = res_v;
			graphics_info.format = mode.info().pixel_format();
			
			if graphics_info.format == PixelFormat::Bitmask {
				graphics_info.bitmask = mode.info().pixel_bitmask().unwrap(); // Will panic if bitmask is none
			}
		}
	}

	let best_mode = gop.query_mode(graphics_info.mode, bs)?;
	gop.set_mode(&best_mode)?;
	graphics_info.base = gop.frame_buffer().as_mut_ptr();
	graphics_info.size = gop.frame_buffer().size();

	system_table().stdout().clear()?;

	Ok(graphics_info)
}