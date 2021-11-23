use uefi::proto::Protocol;
use uefi::prelude::BootServices;
use uefi::{Status, Completion};



/// fetches and returns given protocol
/// Err contains the uefi::status returned
/// Ok returns &mut protocol
pub fn get_proto<T: Protocol>(bs: &BootServices) -> uefi::Result<&'static mut T>
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