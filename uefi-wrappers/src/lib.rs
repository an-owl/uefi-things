#![no_main]
#![no_std]
#![feature(abi_efiapi)]

extern crate rlibc;
extern crate alloc;

use uefi::prelude::*;

mod proto;
mod com;

fn handle_not_success(s: Status){todo!()} //i don't really know what to do with this