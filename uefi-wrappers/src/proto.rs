//! Contains convenience functions for protocols

use uefi::proto;
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

            //TODO return the actual status instead of just Success
            Ok(Completion::new(Status::SUCCESS,protocol))
        }
        Err(i) => {
            Err(i)
        }
    };
}