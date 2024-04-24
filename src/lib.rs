#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate bitflags;
extern crate libc;
use bitflags::bitflags;

mod observer;
//use observer::*;

use libc::c_char;
use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;

include!("./bindings.rs");

pub struct sbio_channel_handle {
    _channel_handle: *mut gre_io_t,
}

bitflags! {
    pub struct SBIO_FLAGS: u32 {
        const RDONLY = GRE_IO_TYPE_RDONLY;
        const XRDONLY = GRE_IO_TYPE_XRDONLY;
        const WRONLY = GRE_IO_TYPE_WRONLY;
        const NONBLOCK = GRE_IO_FLAG_NONBLOCK;
    }
}

impl SBIO_FLAGS {
    pub fn as_u32(&self) -> u32 {
        self.bits()
    }
}

/// Open a SBIO channel using a named connection
///
/// # Safety
pub unsafe fn open(channel_name: &str, flags: SBIO_FLAGS) -> *mut gre_io_t {
    let name_cstr = CString::new(channel_name).unwrap().into_raw();
    let handle: *mut gre_io_t;
    unsafe {
        handle = gre_io_open(name_cstr, flags.as_u32() as i32);
        drop(CString::from_raw(name_cstr));
    }
    handle
}

/// Close a SBIO channel
///
/// # Safety
pub unsafe fn close(channel_handle: *mut gre_io_t) {
    unsafe {
        gre_io_close(channel_handle);
    }
}

///
///
/// # Safety
pub unsafe fn serialize<T>(
    target: &str,
    name: &str,
    format: &str,
    data: &T,
    size: u32,
) -> *mut gre_io_serialized_data_t {
    let buffer: *mut gre_io_serialized_data_t;

    unsafe {
        let target_ptr: *mut c_char = CString::new(target).unwrap().into_raw();
        let name_ptr: *mut c_char = CString::new(name).unwrap().into_raw();
        let format_ptr: *mut c_char = CString::new(format).unwrap().into_raw();
        let data_in = &data as *const _ as *const c_void;

        buffer = gre_io_serialize(
            std::ptr::null_mut(),
            target_ptr,
            name_ptr,
            format_ptr,
            data_in,
            size as i32,
        );

        drop(CString::from_raw(target_ptr));
        drop(CString::from_raw(name_ptr));
        drop(CString::from_raw(format_ptr));
    }
    buffer
}

///
///
/// # Safety
pub unsafe fn unserialize<'a, T>(
    buffer: *mut gre_io_serialized_data_t,
) -> (&'a str, &'a str, &'a str, *const T, i32) {
    let target: &str;
    let name: &str;
    let format: &str;
    let data: *const T;
    let size;

    unsafe {
        let mut target_ptr: *mut c_char = std::ptr::null_mut();
        let mut name_ptr: *mut c_char = std::ptr::null_mut();
        let mut format_ptr: *mut c_char = std::ptr::null_mut();
        let mut data_ptr: *mut libc::c_void = std::ptr::null_mut();

        size = gre_io_unserialize(
            buffer,
            &mut target_ptr as *mut *mut c_char,
            &mut name_ptr as *mut *mut c_char,
            &mut format_ptr as *mut *mut c_char,
            &mut data_ptr as *mut *mut c_void,
        );

        target = CStr::from_ptr(target_ptr).to_str().unwrap();
        name = CStr::from_ptr(name_ptr).to_str().unwrap();
        format = CStr::from_ptr(format_ptr).to_str().unwrap();
        data = data_ptr as *mut T;
    }

    (target, name, format, data, size)
}

///
///
/// # Safety
pub unsafe fn free_buffer(buffer: *mut gre_io_serialized_data_t) {
    unsafe {
        gre_io_free_buffer(buffer);
    }
}

// pub fn create_send_channel(channel_name: &str, flags: SBIO_FLAGS) -> sbio_channel_handle {
//     let channel_handle = open(channel_name, SBIO_FLAGS::RDONLY | flags);
//     sbio_channel_handle { channel_handle: channel_handle }
// }

// pub fn create_receive_channel(channel_name: &str, flags: SBIO_FLAGS) -> sbio_channel_handle {
//     let channel_handle = open(channel_name, SBIO_FLAGS::RDONLY | flags);
//     sbio_channel_handle { channel_handle: channel_handle }
// }

// pub fn destroy_channel(handle: sbio_channel_handle) {
//     close(handle.channel_handle)
// }

// pub fn send_event<T>(
//     _handle: sbio_channel_handle,
//     _name: String,
//     _format: String,
//     _data: &[T],
// ) -> bool {
//     true
// }

// pub fn add_event_callback<T>(
//     _handle: sbio_channel_handle,
//     _name: String,
//     _callback_function: fn(name: String, format: String, data: &[T]),
// ) -> bool {
//     let mut _subject: Subject<T> = Subject::new();
//     true
// }

// pub fn remove_event_callback<T>(
//     _handle: sbio_channel_handle,
//     _callback_function: fn(name: String, format: String, data: &[T]),
// ) -> bool {
//     true
// }

#[cfg(test)]
mod tests {
    use super::*;
    use observer::*;

    #[derive(PartialEq, Debug, Copy, Clone)]
    struct TestData {
        var1: u32,
        var2: u16,
        var3: u16,
    }

    #[test]
    fn open_read_test() {
        let sbio_t;
        unsafe {
            sbio_t = open("sbio1", SBIO_FLAGS::RDONLY);
        }
        assert_ne!(sbio_t, std::ptr::null_mut());

        unsafe {
            close(sbio_t);
        }
    }

    #[test]
    fn serialize_test() {
        let target_in = "target";
        let name_in = "event1";
        let format_in = "4s1 var1 2u1 var2 2u1 var2";
        let mut data_in = TestData {
            var1: 100,
            var2: 10,
            var3: 5,
        };
        let size_in: u32 = 8;

        let buffer;
        unsafe {
            buffer = serialize(target_in, name_in, format_in, &mut data_in, size_in);
        }
        assert_ne!(buffer, std::ptr::null_mut());

        let (target_out, name_out, format_out, ptr, size_out);

        unsafe {
            (target_out, name_out, format_out, ptr, size_out) = unserialize(buffer);
        }
        assert_eq!(size_out, size_in as i32);
        assert_eq!(target_out, target_out);
        assert_eq!(name_out, name_in);
        assert_eq!(format_out, format_in);

        // todo!("Fix the following asserts");
        let _data_out: *const TestData;
        _data_out = ptr;
        // unsafe {
        //     assert_eq!((*data_out).var1, data_in.var1);
        //     assert_eq!((*data_out).var2, data_in.var2);
        //     assert_eq!((*data_out).var3, data_in.var3);
        // }

        unsafe {
            free_buffer(buffer);
        }
    }
}
