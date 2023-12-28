#![no_main]
#![no_std]

use bootloader_lib::*;

use core::panic::PanicInfo;

pub extern "C" fn _start(_bootinfo: &'static BootInfo) -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}