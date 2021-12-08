//! Contains convenience functions for protocols

use uefi::{proto, Handle};
use uefi::prelude::BootServices;
use uefi::{Status, Completion};



/// Fetches and returns protocol `T`
pub fn get_proto<T: proto::Protocol>(bs: &BootServices) -> uefi::Result<&'static mut T>
{
    let protocol;
    return match bs.locate_protocol::<T>() {
        Ok(proto) => {
            let proto = proto.log();
            protocol = unsafe { &mut *proto.get() };
            
            Ok(Completion::new(Status::SUCCESS,protocol))
        }
        Err(i) => {
            Err(i)
        }
    };
}

pub fn get_proto_handle<T: proto::Protocol>(image: Handle, bs: &BootServices) -> uefi::Result<&'static mut T>
{
    let protocol;
    return match bs.handle_protocol(image) {
        Ok(proto) => {
            let proto = proto.log();
            protocol = unsafe { &mut *proto.get() };

            Ok(Completion::new(Status::SUCCESS,protocol))
        }
        Err(i) => {
            Err(i)
        }
    };
}