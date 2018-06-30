// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate rust_gen as gen;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::collections::HashSet;

fn bind_system_header(sdk_root: &Path, header: &str, out_dir: &Path, top: &mut File) {
    let mut header_path = sdk_root.to_owned();
    header_path.push("usr/include");
    header_path.push(header);
    gen::bind_file(&sdk_root, &header_path, &out_dir);
    write!(top, "include!(concat!(env!(\"OUT_DIR\"), \"/{}.rs\"));\n", header_path.file_stem().unwrap().to_str().unwrap()).unwrap();
}

fn main () {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let sdk_root = Path::new("/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk");
    let frameworks = vec!["Foundation"];
    let top_path = out_dir.join("top.rs");
    let mut top = File::create(&top_path).unwrap();
    bind_system_header(&sdk_root, "objc/NSObject.h", &out_dir, &mut top);
    bind_system_header(&sdk_root, "MacTypes.h", &out_dir, &mut top);
    bind_system_header(&sdk_root, "sys/acl.h", &out_dir, &mut top);
    bind_system_header(&sdk_root, "hfs/hfs_unistr.h", &out_dir, &mut top);
    bind_system_header(&sdk_root, "mach/message.h", &out_dir, &mut top);
    let mut done: HashSet<String> = HashSet::new();
    let mut deps: Vec<String> = frameworks.iter().map(|s| s.to_string()).collect();
    while let Some(f) = deps.pop() {
        let newdeps = gen::bind_framework(&sdk_root, &f, &out_dir);
        write!(top, "pub mod {};\n", f).unwrap();
        done.insert(f);
        for d in &newdeps {
            if !done.contains(d) && !deps.iter().any(|s| s == d) {
                deps.push(d.clone());
            }
        }
    }
}
