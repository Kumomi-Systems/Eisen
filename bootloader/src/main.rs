#![no_main]
#![no_std]

use uefi::prelude::*;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let bs = system_table.boot_services();

    system_table.stdout().clear();
    uefi_services::println!("Hallo Eisen!");
    system_table.boot_services().stall(10_000_000);
    
    Status::SUCCESS
}