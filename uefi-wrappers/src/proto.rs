use uefi::proto::Protocol;
use uefi::prelude::BootServices;
use uefi::Status;

pub fn get_proto<T: Protocol>(bs: BootServices) -> Result<&'static mut T,Status>
{
    let protocol;
    return match bs.locate_protocol::<T>() {
        Ok(proto) => {
            let proto = proto.log();
            protocol = unsafe { &mut *proto.get() };
            Ok(protocol)
        }
        Err(i) => {
            Err(i.status())
        }
    };
}