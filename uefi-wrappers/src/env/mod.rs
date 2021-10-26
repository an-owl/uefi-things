use uefi::proto::loaded_image;
use alloc::vec::Vec;
use alloc::vec;
use uefi::proto::loaded_image::LoadOptionsError;
use alloc::string::{String, ToString};

/// This functions similarly to std::env::opts
/// this will return a result containing Ok(iter) which will contain an iterator over the options
/// Err will only be returned if the options cannot be converted into valid utf-8, Err will contain a Vector containing the unconverted args
pub fn args(img: loaded_image::LoadedImage) -> Result<vec::IntoIter<String>,Vec<u8>> {


    let mut buff_size = 1024;
    let mut buff = Vec::new();

    loop {
        buff.resize(buff_size, 0);
        match img.load_options(&mut *buff) {
            Ok(opts) => {
                return Ok(parse_opts(&opts.to_string()));

            }

            Err(LoadOptionsError::NotValidUtf8) => return Err(buff),

            Err(LoadOptionsError::BufferTooSmall) => buff_size *= 2,
        }
    }
}



fn parse_opts(s: &String) -> vec::IntoIter<String> {

    let mut args: Vec<String> = Vec::new();


    let mut i = 0;
    while i < s.len()
    //this is a c like for loop
    {
        if s.chars().nth(i).unwrap() == ' '{
            i += 1; //skip char if its ' '
        } else {
            //push argv to args
            let end = i + get_argv(&s[i..s.len()]);
            args.push(s[i..end].to_string());
            i = end;
        }
        i += 1;
    }

    let mut ret: Vec<String> = Vec::new();
    for mut i in args {
        //TODO clean the string better don't just purge it
        i.retain(|c| (c != '\'') && (c !='\"'));
        ret.push(i)
    }


    return ret.into_iter()
}

fn get_argv(s: &str) -> usize {
    //gets the last index of the argument of the given string and index


    let findable= ['"','\'']; //characters used to parse raw sections of string
    let mut find = '\0'; //while find == '\0' a quote has not been ended, leaving a quote open should be handled by a shell nd is therefore undefined behaviour
    let mut index: usize = 0;
    for i in s.chars(){
        'quotes: for ch in findable{
            if (i == ch) && (find == '\0')
            //this branch finds the beginning of the quote
            {
                find = ch;
                break 'quotes;

            } else if (i == ch) && (find == ch)
            //this branch finds the end of the quote
            {
                find = '\0';
                break 'quotes;
            }
        }
        if (i == ' ') && (find == '\0'){
            break
        }
        index += 1;

    };
    return index;
}
