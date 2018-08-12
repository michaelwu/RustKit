extern crate rustkit;

use rustkit::NSObject;

#[test]
fn nsobject_new() {
    let _obj = NSObject::new().unwrap();
}
