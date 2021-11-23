//! This library contains functions used to interface the the [uefi] crate at a higher level than it
//! provides in order to simplify development
//!
//!

#![no_main]
#![no_std]
#![feature(abi_efiapi)]

extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate log;


pub mod proto;
pub mod com;
pub mod fs;
pub mod env;
pub mod glib;