#![windows_subsystem = "windows"]
extern crate winapi;
pub mod brush;
pub mod class;
pub mod menu;
mod wide;

use std::mem::{size_of_val, zeroed};
use std::ptr::null_mut;
use winapi::shared::minwindef::{ATOM, DWORD, HIWORD, LOWORD, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HWND};
use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};
use winapi::shared::winerror::{ERROR_INVALID_WINDOW_HANDLE, FACILITY_WIN32};
use winapi::um::errhandlingapi::{FatalAppExitW, GetLastError};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::shellapi::{NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICON_VERSION_4, Shell_NotifyIconW};
use winapi::um::winuser::*;
use winapi::um::winuser::{MB_ICONINFORMATION, MB_OK, MessageBoxW};
use winapi::um::winnt::{LPCWSTR};

use brush::Brush;
use class::ClassBuilder;
use menu::{PopupMenu, MenuAction, MenuCheck, MenuStatus};
use wide::ToWide;

const WM_APP_NOTIFICATION_ICON: u32 = WM_APP + 1;

unsafe extern "system" fn wndproc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
        },
        WM_APP_NOTIFICATION_ICON => {
            let x = GET_X_LPARAM(wparam as LPARAM);
            let y = GET_Y_LPARAM(wparam as LPARAM);
            let event = LOWORD(lparam as DWORD);
            let _id = HIWORD(lparam as DWORD);
            match event as UINT {
                WM_MOUSEMOVE => (),
                WM_CONTEXTMENU => {
                    let menu = PopupMenu::new().unwrap();
                    let child = PopupMenu::new().unwrap();
                    menu.append_string("IP 192.168.1.1", MenuAction::Id(0), MenuStatus::Disabled, MenuCheck::Unchecked).unwrap();
                    child.append_string("Slow Response", MenuAction::Id(3), MenuStatus::Enabled, MenuCheck::Unchecked).unwrap();
                    child.append_string("Application Error", MenuAction::Id(4), MenuStatus::Enabled, MenuCheck::Unchecked).unwrap();
                    child.append_string("Bad Experience", MenuAction::Id(5), MenuStatus::Enabled, MenuCheck::Unchecked).unwrap();
                    menu.append_string("Report", MenuAction::ChildMenu(child), MenuStatus::Enabled, MenuCheck::Unchecked).unwrap();
                    menu.append_string("Login", MenuAction::Id(1), MenuStatus::Enabled, MenuCheck::Unchecked).unwrap();
                    menu.append_separator().unwrap();
                    menu.append_string("Exit", MenuAction::Id(2), MenuStatus::Enabled, MenuCheck::Unchecked).unwrap();
                    let code = menu.display(hwnd, x, y).unwrap();
                    match code {
                        0 => {},
                        1 => {
                            message_box(
                                "Login system is unimplemented.",
                                "Login", MB_OK | MB_ICONWARNING,
                            ).unwrap();
                        },
                        2 => {
                            let ret = DestroyWindow(hwnd);
                            if ret == 0 {
                                let err = GetLastError();
                                die(&format!("Failed to destroy window: {}", err));
                            }
                        },
                        3 | 4 | 5 => {
                            message_box(
                                "Your complaint has been noted.",
                                "Network complaint", MB_OK | MB_ICONINFORMATION,
                            ).unwrap();
                        },
                        _ => unreachable!(),
                    }
                },
                _ => {},
            }
            return 0
        },
        _ => (),
    };
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
fn die(s: &str) -> ! {
    unsafe { FatalAppExitW(0, s.to_wide_null().as_ptr()); }
    unreachable!()
}
fn main() {
    unsafe {
        let brush = Brush::solid_rgb(0x44, 0x77, 0xFF).unwrap();
        let atom = ClassBuilder::new().name("LEPORIDAE")
            .background(brush).icon().register().unwrap();
        let hwnd = CreateWindowExW(
            WS_EX_OVERLAPPEDWINDOW, atom.as_raw() as LPCWSTR,
            "I'm a window!".to_wide_null().as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT, CW_USEDEFAULT,
            400, 300,
            null_mut(), null_mut(), null_mut(), null_mut(),
        );
        if hwnd.is_null() {
            let err = GetLastError();
            die(&format!("Failed to create window: {}", err));
        }
        let mut nid: NOTIFYICONDATAW = zeroed();
        nid.cbSize = size_of_val(&nid) as DWORD;
        nid.hWnd = hwnd;
        nid.uID = 273;
        nid.hIcon = LoadIconW(GetModuleHandleW(null_mut()), MAKEINTRESOURCEW(2));
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_SHOWTIP | NIF_TIP;
        nid.uCallbackMessage = WM_APP_NOTIFICATION_ICON;
        *nid.uVersion_mut() = NOTIFYICON_VERSION_4;
        let tooltip = "I'm a notification icon!".to_wide_null();
        nid.szTip[..tooltip.len()].copy_from_slice(&tooltip);

        let err = Shell_NotifyIconW(NIM_ADD, &mut nid);
        if err == 0 {
            die(&format!("Failed to create notification icon"));
        }
        let err = Shell_NotifyIconW(NIM_SETVERSION, &mut nid);
        if err == 0 {
            die(&format!("Failed to set version for notification icon"));
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
            if msg.message == WM_QUIT {
                break;
            }
            // println!("message: {}", msg.message);
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let err = Shell_NotifyIconW(NIM_DELETE, &mut nid);
        if err == 0 {
            die(&format!("Failed to destroy notification icon"));
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error(u32);
impl Error {
    fn from_raw(code: u32) -> Error {
        Error(code)
    }
    fn as_raw(&self) -> u32 {
        self.0
    }
    fn get_last_error() -> Error {
        Error::from_raw(unsafe { GetLastError() })
    }
    fn into_hresult(&self) -> i32 {
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
struct MessageLoop {

}
impl MessageLoop {
}
struct Window;
struct WindowBuilder;

fn message_box(text: &str, caption: &str, flags: u32) -> Result<i32, Error> {
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
