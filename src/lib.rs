// SPDX-License-Identifier: GPL-2.0

pub mod os_usb {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!("libusb1.0_bindgen.rs");

    use std::convert::TryInto;
    use std::mem::transmute;

    fn desc_type_str(dev_type: libusb_descriptor_type) -> &'static str {
        match dev_type {
            libusb_descriptor_type_LIBUSB_DT_DEVICE => "DEVICE",
            libusb_descriptor_type_LIBUSB_DT_CONFIG => "CONFIG",
            libusb_descriptor_type_LIBUSB_DT_STRING => "STRING",
            libusb_descriptor_type_LIBUSB_DT_INTERFACE => "INTERFACE",
            libusb_descriptor_type_LIBUSB_DT_ENDPOINT => "ENDPOINT",
            libusb_descriptor_type_LIBUSB_DT_BOS => "BOS",
            libusb_descriptor_type_LIBUSB_DT_DEVICE_CAPABILITY => "DEVICE_CAPABILITY",
            libusb_descriptor_type_LIBUSB_DT_HID => "HID",
            libusb_descriptor_type_LIBUSB_DT_REPORT => "REPORT",
            libusb_descriptor_type_LIBUSB_DT_PHYSICAL => "PHYSICAL",
            libusb_descriptor_type_LIBUSB_DT_HUB => "HUB",
            libusb_descriptor_type_LIBUSB_DT_SUPERSPEED_HUB => "SUPERSPEED_HUB",
            libusb_descriptor_type_LIBUSB_DT_SS_ENDPOINT_COMPANION => "SS_ENDPOINT_COMPANION",
            _ => "INVALID",
        }
    }

    fn ep_direction_str(ep_addr: u32) -> &'static str {
         match ep_addr & LIBUSB_ENDPOINT_DIR_MASK {
             libusb_endpoint_direction_LIBUSB_ENDPOINT_IN => "IN",
             _ => "OUT",
         }
    }

    fn ep_xfer_type_str(ep_attrs: u32) -> &'static str {
        // XXX libusb documentation is confusing:
        //   Bits 0:1 determine the transfer type and correspond to
        //   libusb_transfer_type...
        //   LIBUSB_TRANSFER_TYPE_BULK_STREAM = 4,
        match ep_attrs & LIBUSB_TRANSFER_TYPE_MASK {
            libusb_transfer_type_LIBUSB_TRANSFER_TYPE_CONTROL => "CONTROL",
            libusb_transfer_type_LIBUSB_TRANSFER_TYPE_ISOCHRONOUS => "ISOCHRONOUS",
            libusb_transfer_type_LIBUSB_TRANSFER_TYPE_BULK => "BULK",
            libusb_transfer_type_LIBUSB_TRANSFER_TYPE_INTERRUPT => "INTERRUPT",
            _ => "INVALID", // XXX impossible. needed for compiler?
        }
    }

    fn dev_eps_iterate(eps: *const libusb_endpoint_descriptor,
                       num_eps: u8) {
        for i in 0..num_eps {
            unsafe {
                let ep = eps.offset(i as isize) as *const libusb_endpoint_descriptor;
                println!("---->{}[{}], addr number: {}, direction: {}, xfer type: {}",
                         desc_type_str((*ep).bDescriptorType as u32), i,
                         (*ep).bEndpointAddress & LIBUSB_ENDPOINT_ADDRESS_MASK as u8,
                         ep_direction_str((*ep).bEndpointAddress as u32),
                         ep_xfer_type_str((*ep).bmAttributes as u32));
            }
        }
    }

    fn dev_if_descs_iterate(if_descs: *const libusb_interface_descriptor,
                            num_descs: i32) {
        for i in 0..num_descs {
            unsafe {
                let if_desc = if_descs.offset(i as isize) as *const libusb_interface_descriptor;
                println!("--->{}[{}], number of endpoints: {}",
                         desc_type_str((*if_desc).bDescriptorType as u32), i,
                         (*if_desc).bNumEndpoints);
                dev_eps_iterate((*if_desc).endpoint, (*if_desc).bNumEndpoints);
            }
        }
    }

    fn dev_ifs_iterate(ifs: *const libusb_interface, num_ifs: u8) {
        for i in 0..num_ifs {
            unsafe {
                let interface = ifs.offset(i as isize) as *const libusb_interface;
                println!("-->interface[{}], number of descriptors: {}",
                         i, (*interface).num_altsetting);
                dev_if_descs_iterate((*interface).altsetting, (*interface).num_altsetting);
            }
        }
    }

    fn dev_cfgs_iterate(dev: *mut libusb_device, num_cfgs: u8) {
        for i in 0..num_cfgs {
            let mut cfg_desc: *mut libusb_config_descriptor = std::ptr::null_mut();
            unsafe {
                let ret = libusb_get_config_descriptor(dev, i, &mut cfg_desc);
                if ret < 0 {
                    panic!("libusb_get_config_descriptor failed");
                }
                println!("->{}[{}] Len {}",
                         desc_type_str((*cfg_desc).bDescriptorType as u32),
                         i, (*cfg_desc).bLength);
                dev_ifs_iterate((*cfg_desc).interface, (*cfg_desc).bNumInterfaces);
                libusb_free_config_descriptor(cfg_desc);
            }
        }
    }

    fn dev_list_iterate(dev_list: *const *mut libusb_device, dev_list_len: i64) {
        for i in 0..dev_list_len {
            let dev_desc = libusb_device_descriptor { bLength: 0,
                                                      bDescriptorType: 0,
                                                      bcdUSB: 0,
                                                      bDeviceClass: 0,
                                                      bDeviceSubClass: 0,
                                                      bDeviceProtocol: 0,
                                                      bMaxPacketSize0: 0,
                                                      idVendor: 0,
                                                      idProduct: 0,
                                                      bcdDevice: 0,
                                                      iManufacturer: 0,
                                                      iProduct: 0,
                                                      iSerialNumber: 0,
                                                      bNumConfigurations: 0 };
            let off: isize = i.try_into().unwrap();
            unsafe {
                let dev = *dev_list.offset(off) as *mut libusb_device;
                let bus = libusb_get_bus_number(dev);
                let addr = libusb_get_device_address(dev);
                let ret = libusb_get_device_descriptor(dev, transmute(&dev_desc));
                if ret < 0 {
                    panic!("libusb_get_device_descriptor failed");
                }
                println!("{}[{}] Bus {:03} Device {:03} ID {:04x}:{:04x}",
                         desc_type_str(dev_desc.bDescriptorType as u32),
                         i, bus, addr, dev_desc.idVendor, dev_desc.idProduct);
                dev_cfgs_iterate(dev, dev_desc.bNumConfigurations);
            }
        }
    }

    pub fn devs_iterate() -> Option<i64> {
        let mut libusb_ctx: *mut libusb_context = std::ptr::null_mut();
        let ret;
        unsafe { ret = libusb_init(&mut libusb_ctx); }
        if ret != 0 {
            println!("failed to initialize libusb: {}", ret);
            return None;
        }

        let mut dev_list: *mut *mut libusb_device = std::ptr::null_mut();
        let dev_list_len = unsafe { libusb_get_device_list(libusb_ctx, &mut dev_list) };
        if dev_list_len <= 0 {
            unsafe { libusb_exit(libusb_ctx); }
            return None;
        }

        dev_list_iterate(dev_list, dev_list_len);

        unsafe {
            libusb_free_device_list(dev_list, 1);
            libusb_exit(libusb_ctx);
        }

        return Some(dev_list_len);
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn libusb_init_exit() {
            let mut libusb_ctx: *mut libusb_context = std::ptr::null_mut();
            unsafe {
                assert_eq!(libusb_init(&mut libusb_ctx), 0);
                libusb_exit(libusb_ctx);
            }
        }

        #[test]
        fn libusb_dev_list() {
            let mut libusb_ctx: *mut libusb_context = std::ptr::null_mut();
            let mut dev_list: *mut *mut libusb_device = std::ptr::null_mut();
            unsafe {
                assert_eq!(libusb_init(&mut libusb_ctx), 0);
                assert_eq!(libusb_get_device_list(libusb_ctx, &mut dev_list) >= 0, true);
                libusb_free_device_list(dev_list, 1);
                libusb_exit(libusb_ctx);
            }
        }
    }
}
