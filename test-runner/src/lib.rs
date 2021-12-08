#![no_std]
#![feature(abi_efiapi)]

extern crate rlibc;
extern crate alloc;

use uefi::proto::console::text::Output;
use uefi::prelude::*;

use alloc::vec::Vec;
use core::fmt::Write;


#[derive(Clone,Copy,PartialEq)]
pub enum TestResult{
    NotTested,
    Pass,
    Fail(uefi::Status,&'static str),
    Unknown(uefi::Status,&'static str),
}


pub struct Test{
    pub name: &'static str,
    test_fn: fn(Handle, &SystemTable<Boot>) -> TestResult,
    pub test_state: TestResult,
}

impl Test{
    pub fn new(name: &'static str, test_fn: fn(Handle, &SystemTable<Boot>) -> TestResult) -> Self{
        return Self{name, test_fn,test_state: TestResult::NotTested}
    }
    pub fn run(&mut self, handle: Handle, st: &SystemTable<Boot>) -> TestResult{
        self.test_state = (self.test_fn)(handle.clone(), st);
        return self.test_state
    }
}

pub fn test_runner(tests: Vec<Test>, table: Handle ,st: &SystemTable<Boot> ){
    use uefi::proto::console::text::Color::*;
    use uefi_wrappers::proto::get_proto;
    let mut test_comp = Vec::new();

    let o = get_proto::<Output>(&st.boot_services()).unwrap().unwrap();
    //run tests
    for mut test in tests{

        writeln!(o, "running test {}",test.name).unwrap();
        match test.run(table, &st){
            TestResult::NotTested => panic!("nani?"),
            TestResult::Pass => {
                o.set_color(Green,Black).unwrap().unwrap();
                writeln!(o,"ok").unwrap();
                o.set_color(White,Black).unwrap().unwrap();
            }
            TestResult::Fail(_, _) => {
                o.set_color(Red,Black).unwrap().unwrap();
                writeln!(o,"FAILED").unwrap();
                o.set_color(White,Black).unwrap().unwrap();
            }
            TestResult::Unknown(s, _) => {
                o.set_color(Yellow,Black).unwrap().unwrap();
                writeln!(o,"Unknown").unwrap();
                o.set_color(White,Black).unwrap().unwrap();
                writeln!(o,"Status: {:?}    Test may need to be manually verified",s).unwrap();
            }
        }
        test_comp.push(test)
    }

    writeln!(o, "").unwrap();
    //print fails / reasons
    for test in &test_comp{
        if test.test_state == TestResult::Pass{
            continue
        }

        let status;
        let message;
        write!(o,"Test {}",test.name).unwrap();
        if let TestResult::Fail(s,msg) = test.test_state.clone(){
            status = s;
            message = msg;
            o.set_color(Red,Black).unwrap().unwrap();
            write!(o," FAILED ").unwrap()
        } else if let TestResult::Unknown(s,msg) = test.test_state.clone() {
            status = s;
            message = msg;
            o.set_color(Yellow,Black).unwrap().unwrap();
            write!(o," Unknown ").unwrap()
        } else {
            panic!()
        }
        o.set_color(White,Black).unwrap().unwrap();
        write!(o,"with status {:?}." ,status).unwrap();
        if message.len() > 0{
            write!(o, "Gave reason {}",message).unwrap();
        }
        writeln!(o,"").unwrap();
    }

    let mut num_pass = 0;
    let mut num_fail = 0;
    let mut num_unkn = 0;
    //collect data summary
    for test in &test_comp{
        match test.test_state{
            TestResult::NotTested => panic!(),
            TestResult::Pass => num_pass += 1,
            TestResult::Fail(_, _) => num_fail += 1,
            TestResult::Unknown(_, _) => num_unkn += 1,
        }
    }

    writeln!(o,"Tests complete").unwrap();

    write!(o,"  {} tests run", test_comp.len()).unwrap();
    write!(o, " {} tests", num_pass).unwrap();
    o.set_color(Green,Black).unwrap().unwrap();
    write!(o, " PASSED ").unwrap();
    o.set_color(White,Black).unwrap().unwrap();

    write!(o, "{} tests", num_fail).unwrap();
    o.set_color(Red,Black).unwrap().unwrap();
    write!(o, " FAILED ").unwrap();
    o.set_color(White,Black).unwrap().unwrap();

    write!(o, "{} tests", num_unkn).unwrap();
    o.set_color(Yellow,Black).unwrap().unwrap();
    write!(o, " UNKNOWN ").unwrap();
    o.set_color(White,Black).unwrap().unwrap();
    writeln!(o,"").unwrap();
}