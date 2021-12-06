#![no_main]
#![no_std]
#![feature(abi_efiapi)]



extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate log;


use uefi::prelude::*;
use uefi::proto::pi::mp::MpServices;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::media::fs;
use uefi::proto::console;

mod test_things;

#[entry]
fn main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    use crate::l_tests::*;
    uefi_services::init(&mut st).unwrap().unwrap(); //ur fucked if this fails anyway


    let _gop = uefi_wrappers::proto::get_proto::<GraphicsOutput>(st.boot_services()).unwrap().unwrap();
    let _mp = uefi_wrappers::proto::get_proto::<MpServices>(st.boot_services()).unwrap().unwrap();
    let fs = uefi_wrappers::proto::get_proto::<fs::SimpleFileSystem>(st.boot_services()).unwrap().unwrap();
    let _o = uefi_wrappers::proto::get_proto::<console::text::Output>(st.boot_services()).unwrap().unwrap();
    let _log = unsafe{uefi::logger::Logger::new(st.stderr())};

    info!("successfully initialized");

    get_file(fs,"/imp.nsh");
    get_file(fs,"/tools/shutdown.efi");



    Status::SUCCESS
}

pub mod l_tests{
    use uefi::proto::media::fs;
    use uefi::proto::media::file;
    use uefi_wrappers::fs as fsw;
    use log::*;
    use crate::test_things::TestResult;

    pub fn get_file(fs: &mut fs::SimpleFileSystem, f_name: &str) -> TestResult{
        return match fsw::get_file_from_path(fs, f_name, file::FileMode::Read, file::FileAttribute::empty()){
            fsw::GetFileStatus::Found(_) => {
                info!("found {}",f_name);
                TestResult::Pass
            }
            fsw::GetFileStatus::NotFound(e) => {
                error!("cant find {} in {}" , e, f_name);
                TestResult::Fail
            }
            fsw::GetFileStatus::Err(e) => {
                error!("Error got {:?}", e);
                TestResult::Fail
            }
        }
    }



}
