
use std::mem::zeroed;
use std::ptr::null_mut;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{DefWindowProcW, DispatchMessageW, GetMessageW, MSG, TranslateMessage};
use Error;
use window::Window;

pub(crate) unsafe extern "system" fn wndproc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    println!("wndproc: 0x{:x}", msg);
    let window = Window::from_raw(hwnd).expect("Failed to get window internals")
        .unwrap_or_else(|| Window::initialize(hwnd).expect("Failed to initialize window"));
    if let Some(response) = window.handle_event(msg, wparam, lparam) {
        return response.as_raw()
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
pub(crate) fn message_loop() {
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
