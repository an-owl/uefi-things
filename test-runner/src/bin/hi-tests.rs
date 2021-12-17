#![no_main]
#![no_std]
#![feature(abi_efiapi)]

//! hi not as in hello but as in human interface

extern crate rlibc;
extern crate alloc;
extern crate log;
extern crate uefi;


use uefi::prelude::*;
use alloc::vec::Vec;

#[entry]
fn main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap().unwrap(); //ur fucked if this fails anyway
    let mut tests = Vec::new();

    tests.push(test_runner::Test::new("test graphics draw", tests::test_graphics_things));
    tests.push(test_runner::Test::new("test drw_to_buff", tests::test_receive_sprite));

    test_runner::test_runner(tests, image, &st);
    Status::SUCCESS
}






pub mod tests {
    use uefi::Handle;
    use uefi::prelude::*;
    use core::fmt::Write;
    use test_runner::TestResult;
    use test_runner::TestResult::*;
    use uefi_wrappers::proto::get_proto;
    use uefi::proto::console::gop::GraphicsOutput;
    use uefi_wrappers::glib::{GraphicsHandle, Sprite};
    use uefi::proto::media::fs::SimpleFileSystem;
    use uefi_wrappers::fs::{read_file, get_file_from_path};
    use uefi::proto::media::file::FileType;
    use uefi::proto::console::text::Output;
    use alloc::string::ToString;
    const SECOND: usize = 1000000;
    pub fn test_graphics_things(_table: Handle, st: &SystemTable<Boot>) -> TestResult{
        const WIDTH: usize = 800;
        const HEIGHT: usize = 600;

        let gop = get_proto::<GraphicsOutput>(st.boot_services()).unwrap().unwrap();
        let fs = get_proto::<SimpleFileSystem>(st.boot_services()).unwrap().unwrap();
        let o = get_proto::<Output>(st.boot_services()).unwrap().unwrap();

        let mut g = GraphicsHandle::new(gop,None);


        if g.get_resolution() != (WIDTH,HEIGHT){
            let (width,height) = g.get_resolution();
            writeln!(o,"current res {},{} expected {}x{}",width,height,WIDTH,HEIGHT).unwrap();
            return Fail(Status::ABORTED, "Only configured for 800x600 resolution");
        }
        let mut bg = Sprite::new(WIDTH, HEIGHT);
        let ppm_dat = match uefi_wrappers::fs::get_file_from_path(fs,
                                                    "/test-img.ppm",
                                                    uefi::proto::media::file::FileMode::Read,
                                                    uefi::proto::media::file::FileAttribute::empty())
            .into_type().unwrap(){
            FileType::Regular(f) => {
                read_file(f)
            }
            FileType::Dir(_) => {

                return Fail(Status::LOAD_ERROR,"Found Directory")
            }
        };



        match bg.read_ppm(&ppm_dat){
            Ok(_) => {

            }
            Err(e) => return Fail(Status::LOAD_ERROR,e)
        }


        if let Err(()) = g.insert_buff(bg){
            let (width, height) = g.get_resolution();
            writeln!(o,"current res {},{} expected {}x{}",width,height,WIDTH,HEIGHT).unwrap();
            return Fail(Status::BAD_BUFFER_SIZE,"Failed to insert buffer into GraphicsHandle")
        }

        g.draw(0).unwrap().unwrap();
        st.boot_services().stall(2000000);

        Unknown(Status::SUCCESS,"Requires Human Verification")
    }

    pub fn test_receive_sprite(_table: Handle, st: &SystemTable<Boot>) -> TestResult{
        let gop = get_proto::<GraphicsOutput>(st.boot_services()).unwrap().unwrap();
        let fs = get_proto::<SimpleFileSystem>(st.boot_services()).unwrap().unwrap();
        let o = get_proto::<Output>(st.boot_services()).unwrap().unwrap();

        let mut g = GraphicsHandle::new(gop,None);

        let bg_file =  match uefi_wrappers::fs::get_file_from_path(fs,
                                                    "/test-img.ppm",
                                                    uefi::proto::media::file::FileMode::Read,
                                                    uefi::proto::media::file::FileAttribute::empty())
            .into_type().unwrap(){
            FileType::Regular(f) => {
                read_file(f)
            }
            FileType::Dir(_) => {

                return Fail(Status::LOAD_ERROR,"Found Directory")
            }
        };

        let mut bg = Sprite::new(g.get_resolution().0,g.get_resolution().1);
        let mut fg = Sprite::new(171,226);
        bg.read_ppm(&bg_file).unwrap();
        drop(bg_file);

        let fg_file =  match uefi_wrappers::fs::get_file_from_path(fs,
                                                    "/smolspite.ppm",
                                                    uefi::proto::media::file::FileMode::Read,
                                                    uefi::proto::media::file::FileAttribute::empty())
            .into_type().unwrap(){
            FileType::Regular(f) => {
                read_file(f)
            }
            FileType::Dir(_) => {

                return Fail(Status::LOAD_ERROR,"Found Directory")
            }
        };

        fg.read_ppm(&fg_file).unwrap();
        drop(fg_file);



        g.insert_buff(bg);
        g.draw(0);
        st.boot_services().stall(1*SECOND);

        g.draw_to_buff(&fg,0,(0,0));
        g.draw(0);
        st.boot_services().stall(1*SECOND);

        let (mut width,mut height) = g.get_resolution(); //sets cords to place in bottom right corner
        width  -= fg.resolution().0;
        height -= fg.resolution().1;
        writeln!(o,"x: {}",width).unwrap();
        writeln!(o,"y: {}",height).unwrap();
        st.boot_services().stall(3*SECOND);
        g.draw_to_buff(&fg,0,(width,height));

        g.draw(0);
        st.boot_services().stall(3*SECOND);

        Unknown(Status::SUCCESS,"Requires human verification")
    }
}