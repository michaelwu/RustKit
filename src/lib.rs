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

/* This probably won't work for bitcode. Need to use LLVM IR metadata.
 * See llvm/docs/LangRef.rst */
#[allow(dead_code)]
#[no_mangle]
#[link_section = "__DATA,__objc_imageinfo,regular,no_dead_strip"]
pub static IMAGEINFO: objc::ObjCImageInfo = objc::ObjCImageInfo {
    version: 0,
    flags: 0,
};

include!(concat!(env!("OUT_DIR"), "/top.rs"));
