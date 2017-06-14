
use std::mem::zeroed;
use std::ptr::null_mut;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{DefWindowProcW, DispatchMessageW, GetMessageW, MSG, TranslateMessage};

use Error;
use event::Event;
use window::Window;

pub unsafe extern "system" fn wndproc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    //let window = Window::from_raw(hwnd);
    let event = Event::from_raw(msg, wparam, lparam);
    if let Some(code) = ::handler(hwnd, event) {
        return code
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
pub fn message_loop() {
    let mut msg: MSG = unsafe { zeroed() };
    loop {
        let ret = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
        if ret == 0 {
            break;
        } else if ret == -1 {
            Error::get_last_error().die("Failed to get message");
        }
        unsafe { TranslateMessage(&msg) };
        unsafe { DispatchMessageW(&msg) };
    }
}
