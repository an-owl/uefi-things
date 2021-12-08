#![no_main]
#![no_std]
#![feature(abi_efiapi)]


extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate log;
extern crate uefi;


use uefi::prelude::*;
use alloc::vec::Vec;
use uefi::proto::console::text::Output;
use core::fmt::Write;

#[entry]
fn main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap().unwrap(); //ur fucked if this fails anyway
    let mut tests = Vec::new();



    test_runner::test_runner(tests, image, &st);
    Status::SUCCESS
}