#![no_main]
#![no_std]
#![feature(abi_efiapi)]

extern crate rlibc;
extern crate alloc;

use uefi::prelude::*;
use uefi::ResultExt;

mod proto;