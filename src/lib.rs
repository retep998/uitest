//#![windows_subsystem = "windows"]
#![deny(unreachable_patterns)]
extern crate winapi;
pub mod brush;
pub mod class;
pub mod event;
pub mod icon;
mod wndproc;
pub mod menu;
pub mod notifyicon;
mod wide;
pub mod window;

use std::ptr::null_mut;
use winapi::shared::winerror::{FACILITY_WIN32};
use winapi::um::errhandlingapi::{FatalAppExitW, GetLastError, SetLastError};
use winapi::um::winuser::{MessageBoxW};

use wide::ToWide;

#[derive(Clone, Copy, Debug)]
pub struct Error(u32);
impl Error {
    fn from_raw(code: u32) -> Error {
        Error(code)
    }
    pub fn as_raw(&self) -> u32 {
        self.0
    }
    fn get_last_error() -> Error {
        Error::from_raw(unsafe { GetLastError() })
    }
    fn clear() {
        unsafe { SetLastError(0) }
    }
    pub fn into_hresult(&self) -> i32 {
        let code = self.0 as i32;
        if code < 0 { code }
        else { (code & 0xFFFF) | (FACILITY_WIN32 << 16) | (0x80000000u32 as i32) }
    }
    fn die(&self, s: &str) -> ! {
        let msg = format!("{}: {}", s, self.0);
        unsafe { FatalAppExitW(0, msg.to_wide_null().as_ptr()); }
        unreachable!()
    }
}

pub fn message_box(text: &str, caption: &str, flags: u32) -> Result<i32, Error> {
    let ret = unsafe {
        MessageBoxW(
            null_mut(),
            text.to_wide_null().as_ptr(),
            caption.to_wide_null().as_ptr(),
            flags,
        )
    };
    if ret == 0 { Err(Error::get_last_error()) }
    else { Ok(ret) }
}
