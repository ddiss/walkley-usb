// SPDX-License-Identifier: GPL-2.0

use std::env;

fn main() {
    let lkl_dir = env::var("LKL_DIR").unwrap();

    // XXX assume that lkl has already been compiled
    println!("cargo:rustc-link-search=native={}/tools/lkl/", lkl_dir);
    println!("cargo:rustc-link-lib=static=lkl");
    println!("cargo:rerun-if-changed={}/tools/lkl/liblkl.a", lkl_dir);

    // link against system libusb
    println!("cargo:rustc-link-lib=dylib=usb-1.0");
}
