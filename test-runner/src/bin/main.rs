#![no_main]
#![no_std]
#![feature(abi_efiapi)]


extern crate rlibc;
extern crate alloc;
#[macro_use]
extern crate log;


use uefi::prelude::*;
use alloc::vec::Vec;
use uefi::proto::console::text::Output;
use core::fmt::Write;

#[entry]
fn main(_image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap().unwrap(); //ur fucked if this fails anyway
    let mut log = unsafe {uefi::logger::Logger::new(uefi_wrappers::proto::get_proto::<Output>(st.boot_services()).unwrap().unwrap())};
    log.disable();

    info!("successfully initialized");

    let mut tests = Vec::new();


    tests.push(test_runner::Test::new("Get_file_from_path test", tests::test_file_from_path));
    tests.push(test_runner::Test::new("Get_args test", tests::test_get_args));

    writeln!(st.stdout(),"Running {} tests", tests.len()).unwrap();

    test_runner::test_runner(tests, &st);

    Status::SUCCESS
}



pub mod tests{
    use core::fmt::Write;
    use uefi::prelude::*;
    use uefi_wrappers::fs::GetFileStatus;
    use uefi::proto::media::file::{FileMode, FileAttribute};
    use uefi_wrappers::proto::get_proto;
    use test_runner::TestResult;
    use test_runner::TestResult::*;
    use uefi::proto::loaded_image::LoadedImage;
    use alloc::string::String;
    use alloc::vec::Vec;
    use uefi::proto::console::text::Output;


    pub fn test_file_from_path(st: &SystemTable<Boot>) -> TestResult{
        use uefi::proto::media::fs::SimpleFileSystem;
        use uefi_wrappers::fs::get_file_from_path;

        let mut fs = get_proto::<SimpleFileSystem>(st.boot_services()).unwrap().unwrap();
        match get_file_from_path(&mut fs, "/test-img.ppm", FileMode::Read, FileAttribute::empty()){
            GetFileStatus::Found(_) => {}
            GetFileStatus::NotFound(_) => return Fail(Status::NOT_FOUND,"File not found may not be present"),
            GetFileStatus::Err(e) => return Fail(e,""),
        }

        match get_file_from_path(fs, "notafile", FileMode::Read, FileAttribute::empty()) {
            GetFileStatus::Found(_) => return Fail(Status::SUCCESS,"paths must start with '/' anything else should return ABORTED"),
            GetFileStatus::NotFound(_) => return Fail(Status::NOT_FOUND,"paths must start with '/' anything else should return ABORTED"),
            GetFileStatus::Err(Status::ABORTED) => {}
            GetFileStatus::Err(e) => return Fail(e,"Got wrong error should be Status::ABORTED")
        }

        match get_file_from_path(fs, "/tools/shutdown.efi", FileMode::Read, FileAttribute::empty()) {
            GetFileStatus::Found(_) => {}
            GetFileStatus::NotFound(_) => return Fail(Status::NOT_FOUND,"File not found may not be present"),
            GetFileStatus::Err(e) => return Fail(e,""),
        }

        Pass
    }

    pub fn test_get_args(st: &SystemTable<Boot>) -> TestResult{
        let image = get_proto::<LoadedImage>(st.boot_services()).unwrap().unwrap();
        let o = get_proto::<Output>(st.boot_services()).unwrap().unwrap();
        let args: Vec<String> = uefi_wrappers::env::args(image).unwrap().collect();

        for arg in &args {
            writeln!(o,"{}",arg).unwrap()
        }

        writeln!(o, "\n got {} args", args.len()).unwrap();

        return if args.len() == 0 {
            Fail(Status::NOT_FOUND, "No args frond please ensure some were given")
        } else {
            Pass
        }
    }
}
