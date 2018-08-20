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

#[test]
fn nsobject_description() {
    let desc = NSObject::description();
    assert_eq!(desc.is_some(), true);

    let desc = desc.unwrap();
    let desclen = desc.length();
    let ruststr: String =
        (0..desclen).map(|i|
                         std::char::from_u32(desc.characterAtIndex_(i) as u32).
                         unwrap()).collect();
    assert_eq!(&ruststr, "NSObject");
}
