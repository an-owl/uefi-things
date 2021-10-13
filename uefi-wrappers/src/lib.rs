#![no_main]
#![no_std]
#![feature(abi_efiapi)]

extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate log;

use uefi::prelude::*;

pub mod proto;
pub mod com;
pub mod fs;

fn handle_not_success(s: Status){todo!()} //i don't really know what to do with this