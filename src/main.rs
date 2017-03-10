#![feature(windows_subsystem)]
#![windows_subsystem = "windows"]
extern crate winapi;
use std::ffi::OsStr;
use std::io::Error;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::um::winuser::{MB_OK, MessageBoxW};

fn main() {
    let msg = "Hello, world!";
    let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    let ret = unsafe {
        MessageBoxW(null_mut(), wide.as_ptr(), wide.as_ptr(), MB_OK)
    };
    if ret == 0 {
        println!("Failed: {:?}", Error::last_os_error());
    }
}
