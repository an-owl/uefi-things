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