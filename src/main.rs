#![no_std]
#![no_main]
#![feature(asm)]
#![feature(const_slice_len)]
#![feature(slice_patterns)]

use uefi::{prelude::*, table::runtime::ResetType};

mod chip8;

use log::*;

#[no_mangle]
pub extern "C" fn efi_main(_image: uefi::Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");

    st.stdout()
        .reset(false)
        .expect_success("Failed to reset stdout");

    chip8::run(&st);

    info!("Shutting down in 3 seconds...");
    st.boot_services().stall(3_000_000);

    let rt = st.runtime_services();
    rt.reset(ResetType::Shutdown, Status::SUCCESS, None);
}
