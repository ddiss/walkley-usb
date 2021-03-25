// SPDX-License-Identifier: GPL-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("lkl_next_5.10_host_h_bindgen.rs");
extern "C" { fn dbg_entrance(); }

use std::ffi::CString;
use walkley_usb::os_usb;

fn main() {
    let ret;
    let kernel_cmdline = CString::new("mem=16M loglevel=8").unwrap();
    unsafe {
        let raw_lkl_host_ops = &mut lkl_host_ops as *mut lkl_host_operations;
        ret = lkl_start_kernel(raw_lkl_host_ops, kernel_cmdline.as_ptr());
    }
    if ret < 0 {
        panic!("lkl_start_kernel() failed: {}", ret);
    }

    // TODO filter based on vendor and product ids
    match os_usb::devs_iterate() {
        None => panic!("failed to iterate USB devices"),
        Some(stat) => println!("processed {} USB devices", stat),
    }

    unsafe {
        dbg_entrance();
        lkl_sys_halt();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lkl_start_stop() {
        let kernel_cmdline = CString::new("mem=16M loglevel=8").unwrap();
        unsafe {
            let raw_lkl_host_ops = &mut lkl_host_ops as *mut lkl_host_operations;
            assert_eq!(lkl_start_kernel(raw_lkl_host_ops, kernel_cmdline.as_ptr()), 0);
            assert_eq!(lkl_is_running(), 1);
            assert_eq!(lkl_sys_halt(), 0);
            assert_eq!(lkl_is_running(), 0);
        }
    }
}
