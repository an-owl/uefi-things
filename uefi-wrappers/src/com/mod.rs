//!contains components for handling generic serial communications

use uefi::proto::console::serial::Serial;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use alloc::boxed::Box;

const BUFF_SIZE: usize = 1024;

pub fn read(sp: &mut Serial) -> Vec<u8>{
    let mut cache= VecDeque::new();
    loop {
        let mut buff: Box<[u8; BUFF_SIZE]> = Box::new([0;BUFF_SIZE]);
        match sp.read(&mut *buff){
            //

            Ok(r) => {
                //this arm will drop the buff into the cache to be sorted are the transmission is ended
                if r.status().is_success(){
                    cache.push_back(buff)
                } else {
                    r.log();
                    cache.push_back(buff)
                }
            }

            Err(r) => {
                //this arm will sort the cache append the last buffer and return from the loop
                if r.status().is_success() == false {
                    // TODO clean this up
                }

                let mut ret = clean_cache(cache);
                let mut end = buff.to_vec();
                end.truncate(r.data().clone());
                ret.append(&mut end);

                return ret;
            }
        }

    }
}

fn clean_cache(cache: VecDeque<Box<[u8;BUFF_SIZE]>>) -> Vec<u8>{
    let mut ret= Vec::new();
    for i in cache{
        //iterates from 0.. and appends the array to ret
        ret.append(&mut i.to_vec())
    }
    return ret;
}