// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ptr::NonNull;

/* We use a macro instead of a struct so the user can't try to move
 * or drop the AutoreleasePool and screw up the order of the pops.
 * We use a struct inside the macro anyway to make sure the user
 * can't avoid the pop by returning.
 * Passing a closure to AutoreleasePool is another option, but is
 * less ergonomic than this macro.
 */
#[macro_export]
macro_rules! autoreleasepool {
    ( $b:block ) => {{
        extern "C" {
            fn objc_autoreleasePoolPush() -> *mut u8;
            fn objc_autoreleasePoolPop(c: *mut u8);
        }
        struct AutoreleasePool {
            c: *mut u8,
        }
        impl Drop for AutoreleasePool {
            fn drop(&mut self) {
                unsafe { objc_autoreleasePoolPop(self.c) }
            }
        }
        {
            let pool =
                AutoreleasePool { c: unsafe { objc_autoreleasePoolPush() } };
            $b
        }
    }}
}

#[repr(C)]
pub struct ObjCImageInfo {
    pub version: u32,
    pub flags: u32,
}

#[cfg(target_pointer_width = "32")]
pub type Mask = u16;
#[cfg(target_pointer_width = "64")]
pub type Mask = u32;

// XXX placeholder
pub type Bucket = u8;

#[repr(C)]
pub struct Cache {
    pub buckets: *mut Bucket,
    pub mask: Mask,
    pub occupied: Mask,
}

#[repr(C)]
pub struct ClassDataBits {
    pub bits: usize,
}

#[repr(C)]
pub struct Class {
    pub isa: *const Class,
    pub superclass: *const Class,
    pub cache: Cache,
    pub bits: ClassDataBits,
}

#[repr(C)]
pub struct Protocol {
    pub isa: *const Class,
    pub mangled_name: *const u8,
    pub protocols: *const (),
    pub instance_methods: *const (),
    pub class_methods: *const (),
    pub optional_instance_methods: *const (),
    pub optional_class_methods: *const (),
    pub instance_properties: *const (),
    pub size: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SelectorRef(pub *const u8);
unsafe impl Sync for SelectorRef {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ClassRef(pub *const Class);
unsafe impl Sync for ClassRef {}

#[repr(C)]
pub struct Object {
    pub isa: *const Class,
}

#[repr(C)]
pub struct Super {
    pub receiver: Object,
    pub superclass: *const Class,
}

pub trait ObjCClass: Sized {
    fn classref() -> ClassRef;
}

pub struct Arc<T> {
    ptr: NonNull<T>,
}

impl<T> Arc<T> {
    pub unsafe fn new_unchecked(p: *mut T) -> Arc<T> {
        Arc {
            ptr: NonNull::new_unchecked(p),
        }
    }

    pub unsafe fn new(p: *mut T) -> Option<Arc<T>> {
        if !p.is_null() {
            Some(Arc {
                ptr: NonNull::new_unchecked(p),
            })
        } else {
            None
        }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Arc<T> {
        unsafe {
            objc_retain(self.ptr.as_ptr() as *mut Object);
            Arc::new_unchecked(self.ptr.as_ptr())
        }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        unsafe { objc_release(self.ptr.as_ptr() as *mut Object) }
    }
}

impl<T> std::ops::Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

#[link(name = "objc")]
extern "C" {
    pub fn objc_msgSend(o: *mut Object, op: SelectorRef, ...) -> *mut Object;
    pub fn objc_msgSendSuper2(o: Super, op: SelectorRef, ...) -> *mut Object;
    pub fn objc_msgSend_stret(o: *mut Object, op: SelectorRef, ...);
    pub fn objc_msgSendSuper2_stret(o: Super, op: SelectorRef, ...);
    pub fn objc_msgSend_fpret(o: *mut Object, op: SelectorRef, ...) -> f32;
    pub fn objc_msgSend_fp2ret(o: *mut Object, op: SelectorRef, ...);

    pub fn objc_retain(o: *mut Object) -> *mut Object;
    pub fn objc_release(o: *mut Object);
    // this is some magic.
    pub fn objc_retainAutoreleasedReturnValue(o: *mut Object);

    pub fn objc_allocWithZone(o: ClassRef) -> *mut Object;
}
