#![feature(windows_subsystem)]
#![windows_subsystem = "windows"]
extern crate winapi;
mod wide;
use std::io::Error;
use std::mem::{size_of_val, zeroed};
use std::ptr::null_mut;
use wide::ToWide;
use winapi::shared::minwindef::{DWORD, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HWND};
use winapi::shared::winerror::ERROR_INVALID_WINDOW_HANDLE;
use winapi::um::errhandlingapi::{FatalAppExitW, GetLastError};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::shellapi::{NIF_ICON, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, NOTIFYICON_VERSION_4, Shell_NotifyIconW};
use winapi::um::wingdi::CreateSolidBrush;
use winapi::um::winuser::*;
use winapi::um::winuser::{MB_ICONINFORMATION, MB_OK, MessageBoxW};
use winapi::um::winnt::{LPCWSTR};
unsafe extern "system" fn wndproc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        msg => {
            //println!("Unknown message: 0x{:x}", msg);
            DefWindowProcW(hwnd, msg, wparam, lparam)
        },
    }
}
fn die(s: &str) -> ! {
    unsafe { FatalAppExitW(0, s.to_wide_null().as_ptr()); }
    unreachable!()
}
fn main() {
    unsafe {
        let mut wx: WNDCLASSEXW = zeroed();
        wx.cbSize = size_of_val(&wx) as DWORD;
        wx.lpfnWndProc = Some(wndproc);
        let class_name = "LEPORIDAE".to_wide_null();
        wx.lpszClassName = class_name.as_ptr();
        let brush = CreateSolidBrush(0xFF7744);
        if brush.is_null() {
            let err = GetLastError();
            die(&format!("Failed to create brush: {}", err));
        }
        wx.hbrBackground = brush;
        let icon = LoadIconW(GetModuleHandleW(null_mut()), MAKEINTRESOURCEW(2));
        if icon.is_null() {
            let err = GetLastError();
            die(&format!("Failed to create icon: {}", err));
        }
        wx.hIcon = icon;
        let class = RegisterClassExW(&wx);
        if class == 0 {
            let err = GetLastError();
            die(&format!("Failed to register class: {}", err));
        }
        let hwnd = CreateWindowExW(
            WS_EX_OVERLAPPEDWINDOW, class as LPCWSTR,
            "I'm a window!".to_wide_null().as_ptr(), WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT, CW_USEDEFAULT,
            400, 300,
            null_mut(), null_mut(), null_mut(), null_mut(),
        );
        if hwnd.is_null() {
            let err = GetLastError();
            die(&format!("Failed to create window: {}", err));
        }
        ShowWindow(hwnd, SW_SHOWDEFAULT);
        let mut nid: NOTIFYICONDATAW = zeroed();
        nid.cbSize = size_of_val(&nid) as DWORD;
        nid.hWnd = hwnd;
        nid.uID = 273;
        nid.hIcon = icon;
        nid.uFlags = NIF_ICON | NIF_SHOWTIP | NIF_TIP;
        *nid.uVersion_mut() = NOTIFYICON_VERSION_4;
        let tooltip = "I'm a notification icon!".to_wide_null();
        nid.szTip[..tooltip.len()].copy_from_slice(&tooltip);

        let err = Shell_NotifyIconW(NIM_ADD, &mut nid);
        if err == 0 {
            die(&format!("Failed to create notification icon"));
        }
        let mut msg: MSG = zeroed();
        loop {
            let ret = GetMessageW(&mut msg, hwnd, 0, 0);
            if ret == 0 {
                break;
            } else if ret == -1 {
                let err = GetLastError();
                if err == ERROR_INVALID_WINDOW_HANDLE {
                    break;
                }
                die(&format!("Failed to get message: {}", err));
            }
            //println!("message: {}", msg.message);
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let err = Shell_NotifyIconW(NIM_DELETE, &mut nid);
        if err == 0 {
            die(&format!("Failed to destroy notification icon"));
        }
    }
    message_box("Your computer has been invaded by rabbits.", "Rabbit Alert", MB_OK | MB_ICONINFORMATION).unwrap();
}

fn message_box(text: &str, caption: &str, flags: u32) -> Result<i32, Error> {
    let ret = unsafe {
        MessageBoxW(
            null_mut(),
            text.to_wide_null().as_ptr(),
            caption.to_wide_null().as_ptr(),
            flags,
        )
    };
    if ret == 0 { Err(Error::last_os_error()) }
    else { Ok(ret) }
}
