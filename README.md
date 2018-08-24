# RustKit
Fast and ergonomic Rust bindings for ObjC APIs

RustKit is currently under development. Please try it if you want to contribute or provide feedback on the generated bindings.

## Prerequisites
Clang 8.0 (currently trunk) with a [patch](https://reviews.llvm.org/D50318) is currently required. Build clang and set the `LIBCLANG_PATH` environmental variable to the directory that `libclang.dylib` is in, which should be in the `lib` directory of your clang/llvm build directory.

## Example

```
extern crate rustkit;

use rustkit::NSObject;

fn main() {
    let obj = NSObject::new();
    
    let desc = NSObject::description();
    let desc = desc.unwrap();
    let desclen = desc.length();
    let ruststr: String =
        (0..desclen).map(|i|
                         std::char::from_u32(desc.characterAtIndex_(i) as u32).
                         unwrap()).collect();
    println!("NSObject::description(): {}", ruststr);
}
```
