extern crate rustkit;

use rustkit::NSObject;
use rustkit::NSObjectProto;

#[test]
fn nsobject_new() {
    let obj = NSObject::new();
    assert_eq!(obj.is_some(), true);
    let obj = obj.unwrap();
    assert_eq!(obj.isProxy(), false);
}
