extern crate objc_rustime as objc;
#[macro_use]
extern crate bitflags;

use std::mem;
use std::ptr;
#[repr(C)]
pub struct FILE {
    opaque: [u8; 0]
}
#[repr(C)]
pub struct c_void {
    opaque: [u8; 0]
}

include!(concat!(env!("OUT_DIR"), "/top.rs"));
