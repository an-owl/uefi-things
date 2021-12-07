//! Contains medium level graphics functions
//!
//! See [uefi::proto::console::gop] for detailed info


use uefi::proto::console::gop;
use uefi::proto::pi::mp;


use uefi::proto::console::gop::{BltPixel, BltOp, Mode, GraphicsOutput};
use alloc::vec::Vec;
use core::ops::Deref;
use uefi::Completion;
use uefi::proto::pi::mp::MpServices;


mod lib_2d;
//mod lib_3d;


/// Medium level graphics interface
///
/// ## buffers
///
/// Internally these are implemented as Vec<[Sprite]> but are always the same resolution as the screen
///
/// These are intended to be used as multiple screens that you may want saved,
/// Like a background that you want to keep clean and render sprites onto
///
/// If you create a new buffer fro each frame you may run out of memory quickly

pub struct GraphicsHandle<'boot>
{
    gop: &'boot mut gop::GraphicsOutput<'boot>, //public for direct usage
    //TODO replace _mp with CPU multiprocessing table that handles allocating process anc configuration
    _mp:  MpStatus<'boot>, //for future use, to use MP acceleration due to the lack of hardware graphics acceleration
    height: usize,
    width: usize,
    buffers: Vec<Sprite> //TODO make all functions that create new buffers return its index
}

//implemented before i forgot. Disabled should only be needed for debugging acts the same way as none
enum MpStatus<'mp> {
    None,
    Enabled (&'mp mut MpServices),
    Disabled(&'mp mut MpServices),
}

/// Contains graphical data
///
/// When created within a [GraphicsHandle] is is assumed to be the full screen

pub struct Sprite {
    height: usize,
    width: usize,
    pub data: Vec<BltPixel>,
}

impl<'boot> GraphicsHandle<'boot>{

    /// Generates a new graphics handle
    ///
    /// [MpServices][uefi::proto::pi::mp::MpServices] will be implemented in lib_3dmp in the future
    pub fn new(gop: &'boot mut gop::GraphicsOutput<'boot>, mp: Option<&'boot mut mp::MpServices>,) -> GraphicsHandle<'boot>{

        let (height,width) = gop.current_mode_info().resolution();
        return match mp{
            None => GraphicsHandle {gop, _mp: MpStatus::None,height,width,buffers: Vec::new()},
            Some(mps) => GraphicsHandle {gop, _mp: MpStatus::Enabled(mps),height,width,buffers: Vec::new()}
        };


    }


    /// Copies buffer in buff_num to video
    ///
    /// # Panics
    /// - This function will panic if buff_num is smaller than buffers.len()
    pub fn draw(&mut self, buff_num: usize) -> uefi::Result {

        if buff_num > self.buffers[buff_num].len() {
            panic!();
        }
        self.gop.blt(BltOp::BufferToVideo { buffer: &self.buffers[buff_num], src: gop::BltRegion::Full, dest: (0, 0), dims: (self.height, self.width) })
    }

    /// Copies sprite into framebuffer using receive_sprite()
    /// Location format is (x,y)
    ///
    /// # Panics
    /// - This function will panic if buff_num is not smaller than self.buffers.len()
    pub fn draw_to_buff(&mut self, s: &Sprite, buff_num: usize, location: (usize,usize)) {
        assert!(buff_num < self.buffers.len());

        self.buffers[buff_num].receive_sprite(s,location);
    }

    /// Pushes a new frame buffer into self
    pub fn new_buff(&mut self){
         self.buffers.push(Sprite::new(self.height,self.width));
    }

    /// Attempts to insert sprite into buffers,
    /// Dimensions *must* be the same as the current screen resolution
    pub fn insert_buff(&mut self, s: Sprite) -> Result<(),()>{
        if (s.width == self.width) & (s.height == self.height){
            self.buffers.push(s);
        }

        return Err(());
    }

    /// Removes and returns buffer at `index`
    pub fn remove_buff(&mut self, index: usize) -> Sprite{
        self.buffers.remove(index)
    }

    /// Gets current screen resolution as (width,height)
    pub fn get_resolution(&self) -> (usize,usize){
        (self.width,self.height)
    }
    /// Returns array of available graphics [modes][uefi::proto::console::gop::Mode]
    pub fn get_modes(&self) -> impl ExactSizeIterator<Item = Completion<Mode>> + '_{
        return self.gop.modes()
    }

    /// sets graphics mode
    ///
    /// Clears all buffers because they will be the incorrect resolution
    pub fn set_mode(&mut self, mode: gop::Mode) -> uefi::Result{
        self.buffers.clear();
        return self.gop.set_mode(&mode)
    }
}

impl Sprite {
    /// Creates new partial graphical Sprite with given dimensions
    pub fn new(width: usize, height: usize) -> Sprite {

        let data = alloc::vec![BltPixel::new(0,0,0);height*width];
        return Sprite {height, width, data}

    }

    pub fn to_ppm(&self) -> Vec<u8>{
        use alloc::string::String;
        use core::fmt::Write;

        let mut head = String::new(); //its a bit hacky but its easy
        write!(head,"P3 {} {} 255 ",self.width,self.height).unwrap(); //colour depth always 8 bit I'm not dealing with anything else
        let mut out = Vec::from(head.as_bytes());


        for pix in &self.data{
            //there's probably a better way to do this but with black magic asm fuckery i don't know if i can do it
            //depends on the compiler i guess
            out.push(pix.red);
            out.push(pix.green);
            out.push(pix.blue);
        }

        return out;
    }

    /// Takes ppm file and moves it into frame buffer
    /// Will fit as much data into buffer as it can before exiting, does not check dimensions
    pub fn read_ppm(&mut self, ppm_data: &[u8]) -> Result<(),&str>{

        const MAGIC_NUMBER: [u8;2] = [0x50,0x53]; //ASCII for P3
        let mut data = Vec::from(ppm_data);

        if data.len() < 9 {
            return Err("file too small")
        }
        //P 3
        if !data.starts_with(&MAGIC_NUMBER){
            return Err("Bad Magic");
        }

        data[0] = 0;
        data[1] = 1;
        //removes unhelpful data to simplify following loop


        let find = |find: u8, search: &[u8]| -> usize {
            let mut count: usize = 0;

            for i in search {
                if i == &find {
                    break
                }
                count += 1;
            }
            return count;
        };

        //strip comments
        {
            let mut i = 0;
            while i < data.len(){
                if data[i] == '#' as u8{
                    find('\n' as u8, &data[i..]);

                    for pos in 0..find('\n' as u8, &data[i..]){ //fuck i miss regex
                        data[pos] = 0;
                    }
                }
                i += 1;
            }
        };

        //get head
        let (_head,tail) = {
            let whitespace = " \n\t\0".as_bytes();
            let mut group = 0;
            let mut in_group = false;
            let mut count = 0;
            for i in &data{

                if group == 4{
                    break
                }

                if (whitespace.contains(i)) & (in_group == false){ //triggers on entry to whitespace segment
                    in_group = true;
                    group += 1;

                } else if (!whitespace.contains(i)) & (in_group == true){ //triggers after exit from whitespace segment (    HERE)
                    in_group = false;
                }
                count += 1;
            }

            if data[count..].len() % 3 != 0 {
                // TODO check for extra whitespaces
                return Err("bad_data_len")
            }



            data.split_at(count)

        };

        //parse head maybe?
        //atm assume colours are 255

        {
            let i = 0;
            let buff = &mut self.data;

            while  (i < tail.len() * 3) & (i < tail.len()) {
                let red   = tail[(i * 3) + 0];
                let green = tail[(i * 3) + 1];
                let blue  = tail[(i * 3) + 2];

                buff[i] = BltPixel::new(red,green,blue);
            }

        }
        return Ok(());
    }

    /// Copies one sprite into another
    /// Location format is (x,y)
    /// Sprites that exceed the dimensions of self will be cut off at the furthest possible point from (0,0)
    fn receive_sprite(&mut self, s: &Sprite, location: (usize, usize)){
        let (x,y) = location;
        //std copy obv
        //these can both be done with modified s dimensions to ignore pixels beyond the frame



        //checks if data is out of frame bounds before copying separate for performance reasons
        //TODO may have been made redundant requires testing
        /*let cautious_copy = {
            if (x + s.width) > self.width{
                let altered_x = self.width - (x + s.width);

            }
            for scan in y..(y + s.height){
                if scan > self.height{
                    break
                }

                //contains address offset of first blt in sprites
                let scan_start = scan * self.width;
                let far_scan_start = scan * s.width;

                //is tail past scan line end if yes copy until when?
                //elevate out of loop?



                //copies full scan line from s to self
                self.data[scan_start + x..scan_start + x + s.width] = s.data[far_scan_start..far_scan_start + s.width]
            }
        };*/

        let mut alt_x = s.width;
        let mut alt_y = s.height;

        //if s too wide
        if x + s.width > self.width{
            alt_x = (self.width - (x+s.width)) - s.width;
        }
        if y + s.height > self.height{
            alt_y = (self.width - (y + s.height)) - s.height;
        }

        let mut std_copy = |s_height: usize ,s_width: usize|{

            for scan in y..(y + s_height){

                //contains address offset of first blt in sprites
                let scan_start = scan * self.width;
                let far_scan_start = scan * s_width;

                //copies full scan line from s to self
                //both are of size s_width
                //doesn't work cos it's a bitch
                //self.data[scan_start + x..(scan_start + x + s_width)] = s.data[far_scan_start..far_scan_start + s_width]
                //workaround
                let count = 0;
                for pix in &s.data[far_scan_start..far_scan_start + s_width]{
                    self.data[scan_start+count] = *pix;
                }
            }
        };

        std_copy(alt_x,alt_y)

    }
}

impl Deref for Sprite {
    type Target = [BltPixel];

    fn deref(&self) -> &Self::Target {
        return &*self.data
    }
}