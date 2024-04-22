#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate bitflags;
extern crate libc;
use bitflags::bitflags;

mod observer;
use observer::*;

include!("./bindings.rs");

pub struct sbio_channel_handle {}

bitflags! {
    struct SBIO_FLAGS: u32 {
        const RDONLY = GRE_IO_TYPE_RDONLY;
        const XRDONLY = GRE_IO_TYPE_XRDONLY;
        const WRONLY = GRE_IO_TYPE_WRONLY;
        const NONBLOCK = GRE_IO_FLAG_NONBLOCK;
    }
}

pub fn create_send_channel(_channel_name: String, _flags: u32) -> sbio_channel_handle {
    sbio_channel_handle {}
}

pub fn create_receive_channel(_channel_name: String, _flags: u32) -> sbio_channel_handle {
    sbio_channel_handle {}
}

pub fn send_event<T>(
    _handle: sbio_channel_handle,
    _name: String,
    _format: String,
    _data: &[T],
) -> bool {
    true
}

pub fn add_event_callback<T>(
    _handle: sbio_channel_handle,
    _name: String,
    _callback_function: fn(name: String, format: String, data: &[T]),
) -> bool {
    let mut _subject: Subject<T> = Subject::new();
    true
}

pub fn remove_event_callback<T>(
    _handle: sbio_channel_handle,
    _callback_function: fn(name: String, format: String, data: &[T]),
) -> bool {
    true
}

pub fn destroy_channel(_handle: sbio_channel_handle) {}

#[cfg(test)]
mod tests {
    use libc::c_char;
    use std::ffi::c_void;
    use std::ffi::CStr;
    use std::ffi::CString;

    use crate::{
        gre_io_free_buffer, gre_io_serialize, gre_io_serialized_data_t, gre_io_unserialize,
    };

    #[derive(PartialEq, Debug)]
    struct TestData {
        var1: u32,
        var2: u16,
        var3: u16,
    }

    #[test]
    fn serialize_test() {
        let target = "target";
        let name = "event1";
        let format = "4s1 var1 2u1 var2 2u1 var2";
        let mut data = TestData {
            var1: 100,
            var2: 10,
            var3: 5,
        };
        let ptr: *mut TestData = &mut data;

        let target_in = CString::new(target).unwrap();
        let name_in = CString::new(name).unwrap();
        let format_in = CString::new(format).unwrap();
        let data_in = ptr as *mut c_void;
        let size_in = 8;

        let target_out: &CStr;
        let name_out: &CStr;
        let format_out: &CStr;
        let data_out: *mut TestData;

        let mut buffer: *mut gre_io_serialized_data_t = std::ptr::null_mut();
        unsafe {
            let mut target_ptr: *mut c_char = target_in.into_raw();
            let mut name_ptr: *mut c_char = name_in.into_raw();
            let mut format_ptr: *mut c_char = format_in.into_raw();
            let mut data_ptr: *mut c_void = std::ptr::null_mut();

            // first serialize the data
            buffer = gre_io_serialize(buffer, target_ptr, name_ptr, format_ptr, data_in, size_in);
            assert_ne!(buffer, std::ptr::null_mut());

            drop(CString::from_raw(target_ptr));
            drop(CString::from_raw(name_ptr));
            drop(CString::from_raw(format_ptr));

            // then unserialize it into new pointers
            let size = gre_io_unserialize(
                buffer,
                &mut target_ptr as *mut *mut c_char,
                &mut name_ptr as *mut *mut c_char,
                &mut format_ptr as *mut *mut c_char,
                &mut data_ptr as *mut *mut c_void,
            );
            assert_eq!(size, size_in);

            target_out = CStr::from_ptr(target_ptr);
            name_out = CStr::from_ptr(name_ptr);
            format_out = CStr::from_ptr(format_ptr);
            data_out = data_ptr as *mut TestData;
        }

        assert_eq!(target_out.to_str().unwrap(), target);
        assert_eq!(name_out.to_str().unwrap(), name);
        assert_eq!(format_out.to_str().unwrap(), format);
        unsafe {
            assert_eq!((*data_out).var1, (*ptr).var1);
            assert_eq!((*data_out).var2, (*ptr).var2);
            assert_eq!((*data_out).var3, (*ptr).var3);
        }

        unsafe {
            gre_io_free_buffer(buffer);
        }
    }
}
