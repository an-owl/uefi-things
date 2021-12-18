//! Contains medium level graphics functions
//!
//! See [uefi::proto::console::gop] for detailed info


use uefi::proto::console::gop;
use uefi::proto::pi::mp;


use uefi::proto::console::gop::{BltPixel, BltOp, Mode, GraphicsOutput};
use alloc::vec::Vec;
use core::ops::Deref;
use uefi::{Completion, Status};
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
    pub gop: &'boot mut gop::GraphicsOutput<'boot>, //public for direct usage
    //TODO replace _mp with CPU multiprocessing table that handles allocating process anc configuration
    mp:  MpStatus<'boot>, //for future use, to use MP acceleration due to the lack of hardware graphics acceleration
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

        let (width,height) = gop.current_mode_info().resolution();
        return match mp{
            None => GraphicsHandle {gop, mp: MpStatus::None,height,width,buffers: Vec::new()},
            Some(mps) => GraphicsHandle {gop, mp: MpStatus::Enabled(mps),height,width,buffers: Vec::new()}
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
        self.gop.blt(BltOp::BufferToVideo { buffer: &self.buffers[buff_num], src: gop::BltRegion::Full, dest: (0, 0), dims: (self.width, self.height) })
    }

    /// Copies sprite into framebuffer using render_sprite()
    /// Location format is (x,y)
    ///
    /// # Panics
    /// - This function will panic if buff_num is not smaller than self.buffers.len()
    pub fn draw_to_buff(&mut self, s: &Sprite, buff_num: usize, location: (usize,usize)) {
        assert!(buff_num < self.buffers.len());

        self.buffers[buff_num].render_sprite(s, location);
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
            return Ok(())
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

    /// Enables/Disables multiprocessing when available
    /// noes nothing on erroneous input.
    pub fn change_mp(self,state: bool) -> Self{
        return match self.mp {
            MpStatus::Enabled(mp) if !state => {
                Self{
                    gop: self.gop,
                    mp: MpStatus::Disabled(mp),
                    buffers: self.buffers,
                    height: self.height,
                    width: self.width,
                }
            }
            MpStatus::Disabled(mp) if state => {
                Self{
                    gop: self.gop,
                    mp: MpStatus::Enabled(mp),
                    buffers: self.buffers,
                    height: self.height,
                    width: self.width,
                }
            }
            _ => {
                Self{
                    gop: self.gop,
                    mp: self.mp,
                    buffers: self.buffers,
                    height: self.height,
                    width: self.width,

                }
            }
        }
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

    ///returns sprite resolution as Width x Height
    pub fn resolution(&self) -> (usize,usize){
        (self.width,self.height)
    }

    /// Takes ppm file and moves it into frame buffer
    /// Will fit as much data into buffer as it can before exiting, does not check dimensions
    pub fn read_ppm(&mut self, ppm_data: &[u8]) -> Result<(),&'static str>{

        const MAGIC_NUMBER: [u8;2] = [0x50,0x36]; //ASCII for P6


        let mut data = Vec::from(ppm_data); //TODO change to uefi::Status

        if data.len() < 9 {
            return Err("file too small")
        }
        //P 3
        if !data.starts_with(&MAGIC_NUMBER){
            return Err("Bad Magic");
        }

        let body_start = Sprite::ppm_head(ppm_data);

        let tail = Vec::from(&data[body_start..]);
        //parse head maybe?
        //atm assume colours are 255

        {
            let mut i = 0;
            let buff = &mut self.data;

            while  (i < buff.len()) && (2 + (i * 3) < tail.len()) {
                let red   = tail[(i * 3) + 0];
                let green = tail[(i * 3) + 1];
                let blue  = tail[(i * 3) + 2];

                buff[i] = BltPixel::new(red,green,blue);
                i += 1;
            }

        }
        return Ok(());
    }


    ///finds the end of ppm head, returns 0 on error
    fn ppm_head(data: &[u8]) -> usize{
        //TODO Return header data

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


        let mut i = 2; //skips magic
        let mut in_group = false;
        let mut group = 0;

        while i < data.len(){

            if group == 3 && in_group == false{
                //exit condition
                return i
            }

            if in_group && data[i].is_ascii_whitespace() {
                //end of group
                in_group = false

            } else if data[i] == b'#' {
                //finds and skips comments
                i += find(b'\n', &data[i..]);
                continue

            } else if !in_group && data[i].is_ascii_digit(){
                //start fo group
                in_group = true;
                group += 1;
            }
            i += 1

        }
        0
    }


    /// Copies one sprite into another
    /// Location format is (x,y)
    /// Sprites that exceed the dimensions of self will be cut off at the furthest possible point from (0,0)
    pub fn render_sprite(&mut self, s: &Sprite, location: (usize, usize)){
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

            for scan in 0..s_height{

                //contains address offset of first blt in sprites
                let scan_start = (scan+y) * self.width; //address for pix 0 in current scanline
                let far_scan_start = scan * s_width; //same as above but for s

                info!("scan: {}, addr: {}",scan,scan_start);

                //info!("scan: {}, addr: {}",scan,far_scan_start);

                self.data[scan_start + x..(scan_start + x + s_width)].copy_from_slice(&s.data[far_scan_start..far_scan_start + s_width])
            }
        };

        std_copy(alt_y,alt_x)

    }
}

impl Deref for Sprite {
    type Target = [BltPixel];

    fn deref(&self) -> &Self::Target {
        return &*self.data
    }
}