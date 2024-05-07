#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

extern crate bitflags;
extern crate libc;
use self::bitflags::bitflags;

use self::libc::c_char;
use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;

include!("./bindings.rs");

bitflags! {
    #[derive(PartialEq)]
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

pub struct sbio_channel_handle {
    channel_handle: *mut gre_io_t,
}

unsafe impl Send for sbio_channel_handle {}

pub struct sbio_serialized_data {
    buffer: *mut gre_io_serialized_data_t,
    pub size: i32,
}

/// Open a SBIO channel using a named connection
pub fn open(channel_name: &str, flags: SBIO_FLAGS) -> Result<sbio_channel_handle, &'static str> {
    let name_cstr = CString::new(channel_name).unwrap().into_raw();
    let handle: *mut gre_io_t;
    unsafe {
        handle = gre_io_open(name_cstr, flags.as_u32() as i32);
        drop(CString::from_raw(name_cstr));
    }

    if handle.is_null() {
        Err("Couldn't open SBIO channel")
    } else {
        Ok(sbio_channel_handle {
            channel_handle: handle,
        })
    }
}

/// Close a SBIO channel
pub fn close(channel_handle: &sbio_channel_handle) {
    if channel_handle.channel_handle.is_null() {
        return;
    }

    unsafe {
        gre_io_close(channel_handle.channel_handle);
    }
}

/// Serialize SBIO event
pub fn serialize<T>(
    target: &str,
    name: &str,
    format: &str,
    data: T,
    size: u32,
) -> Result<sbio_serialized_data, &'static str> {
    let buffer: *mut gre_io_serialized_data_t;

    unsafe {
        let target_ptr: *mut c_char = CString::new(target).unwrap().into_raw();
        let name_ptr: *mut c_char = CString::new(name).unwrap().into_raw();
        let format_ptr: *mut c_char = CString::new(format).unwrap().into_raw();
        let ptr: *const T = &data;
        let data_in = ptr as *const c_void;
        //let data_in = &data as *const _ as *const c_void;

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

    if buffer.is_null() {
        Err("Couldn't serialize event data")
    } else {
        Ok(sbio_serialized_data {
            buffer,
            size: size as i32,
        })
    }
}

/// Unserialize SBIO event
pub fn unserialize<'a, T>(buffer: &sbio_serialized_data) -> (&'a str, &'a str, &'a str, &T, i32) {
    let target: &str;
    let name: &str;
    let format: &str;
    let data: &T;
    let size;

    unsafe {
        let mut target_ptr: *mut c_char = std::ptr::null_mut();
        let mut name_ptr: *mut c_char = std::ptr::null_mut();
        let mut format_ptr: *mut c_char = std::ptr::null_mut();
        let mut data_ptr: *mut c_void = std::ptr::null_mut();

        size = gre_io_unserialize(
            buffer.buffer,
            &mut target_ptr as *mut *mut c_char,
            &mut name_ptr as *mut *mut c_char,
            &mut format_ptr as *mut *mut c_char,
            &mut data_ptr as *mut *mut c_void,
        );

        target = CStr::from_ptr(target_ptr).to_str().unwrap();
        name = CStr::from_ptr(name_ptr).to_str().unwrap();
        format = CStr::from_ptr(format_ptr).to_str().unwrap();
        let ptr = data_ptr as *const T;
        data = &*ptr;
    }

    (target, name, format, data, size)
}

pub fn unserialize_event_name(buffer: &sbio_serialized_data) -> &str {
    let _target: &str;
    let name: &str;
    let _format: &str;
    let _size;

    unsafe {
        let mut target_ptr: *mut c_char = std::ptr::null_mut();
        let mut name_ptr: *mut c_char = std::ptr::null_mut();
        let mut format_ptr: *mut c_char = std::ptr::null_mut();
        let mut data_ptr: *mut c_void = std::ptr::null_mut();

        _size = gre_io_unserialize(
            buffer.buffer,
            &mut target_ptr as *mut *mut c_char,
            &mut name_ptr as *mut *mut c_char,
            &mut format_ptr as *mut *mut c_char,
            &mut data_ptr as *mut *mut c_void,
        );

        _target = CStr::from_ptr(target_ptr).to_str().unwrap();
        name = CStr::from_ptr(name_ptr).to_str().unwrap();
        _format = CStr::from_ptr(format_ptr).to_str().unwrap();
    }

    name
}

/// Free serialized data
pub fn free_buffer(buffer: &sbio_serialized_data) {
    unsafe {
        gre_io_free_buffer(buffer.buffer);
    }
}

/// Send an event
pub fn send(
    channel_handle: &sbio_channel_handle,
    event: &sbio_serialized_data,
) -> Result<i32, &'static str> {
    let ret: i32;
    unsafe { ret = gre_io_send(channel_handle.channel_handle, event.buffer) }

    if ret == -1 {
        Err("Couldn't send event")
    } else {
        Ok(ret)
    }
}

/// Receive an event
pub fn receive(channel_handle: &sbio_channel_handle) -> Result<sbio_serialized_data, &'static str> {
    let mut buffer: *mut gre_io_serialized_data_t = std::ptr::null_mut();
    let ret: i32;

    unsafe {
        ret = gre_io_receive(
            channel_handle.channel_handle,
            &mut buffer as *mut *mut gre_io_serialized_data_t,
        );
    }

    if ret == -1 {
        Err("Couldn't receive event")
    } else {
        Ok(sbio_serialized_data { buffer, size: ret })
    }
}

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
        let result = open("sbio1", SBIO_FLAGS::RDONLY);
        assert!(result.is_ok());
        let handle = result.unwrap();
        close(&handle);
    }

    #[test]
    fn serialize_test() {
        let target_in = "target";
        let name_in = "event1";
        let format_in = "4s1 var1 2u1 var2 2u1 var2";
        let data_in = TestData {
            var1: 100,
            var2: 10,
            var3: 5,
        };
        let size_in: u32 = 8;

        let result = serialize(target_in, name_in, format_in, data_in, size_in);
        assert!(result.is_ok());
        let buffer = result.unwrap();

        let target_out: &str;
        let name_out: &str;
        let format_out: &str;
        let ptr: &TestData;
        let size_out: i32;
        (target_out, name_out, format_out, ptr, size_out) = unserialize(&buffer);
        assert_eq!(size_out, size_in as i32);
        assert_eq!(target_out, target_out);
        assert_eq!(name_out, name_in);
        assert_eq!(format_out, format_in);
        assert_eq!(ptr.var1, data_in.var1);
        assert_eq!(ptr.var2, data_in.var2);
        assert_eq!(ptr.var3, data_in.var3);

        free_buffer(&buffer);
    }

    #[test]
    fn send_receive_test() {
        let target_in = "target";
        let name_in = "event1";
        let format_in = "4s1 var1 2u1 var2 2u1 var2";
        let data_in = TestData {
            var1: 100,
            var2: 10,
            var3: 5,
        };
        let size_in: u32 = 8;

        let result = serialize(target_in, name_in, format_in, data_in, size_in);
        assert!(result.is_ok());
        let buffer = result.unwrap();

        let result = open("sbio1", SBIO_FLAGS::RDONLY);
        let recv_handle = result.unwrap();

        let result = open("sbio1", SBIO_FLAGS::WRONLY);
        let send_handle = result.unwrap();
        let result = send(&send_handle, &buffer);
        assert!(result.is_ok());

        let result = receive(&recv_handle);
        assert!(result.is_ok());

        close(&recv_handle);
        close(&send_handle);
    }
}
