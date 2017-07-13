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
use std::thread::sleep;
use std::time::Duration;
use winapi::shared::winerror::{FACILITY_WIN32};
use winapi::um::errhandlingapi::{FatalAppExitW, GetLastError, SetLastError};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::shellapi::{NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICON_VERSION_4, Shell_NotifyIconW};
use winapi::um::winuser::{LoadIconW, MAKEINTRESOURCEW, MB_ICONINFORMATION, MB_OK, MessageBoxW};

use brush::Brush;
use class::ClassBuilder;
use event::{NotifyIconEvent};
use menu::{PopupMenu, MenuAction, MenuCheck, MenuStatus};
use wide::ToWide;
use window::WindowBuilder;

/*
fn handler(window: HWND, event: Event) -> Option<LRESULT> {
    match event {
        Event::Destroy => unsafe { PostQuitMessage(0) },
        Event::NotifyIcon(event) => match event {
            NotifyIconEvent::ContextMenu(x, y) => {
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
                menu.display(window, x, y).unwrap();
                // TODO Get menu result as event
                /*let code = menu.display(window, x, y).unwrap();
                match code {
                    0 => {},
                    1 => {
                        message_box(
                            "Login system is unimplemented.",
                            "Login", MB_OK | MB_ICONWARNING,
                        ).unwrap();
                    },
                    2 => {
                        if unsafe { DestroyWindow(window) } == 0 {
                            Error::get_last_error().die("Failed to destroy window");
                        }
                    },
                    3 | 4 | 5 => {
                        message_box(
                            "Your complaint has been noted.",
                            "Network complaint", MB_OK | MB_ICONINFORMATION,
                        ).unwrap();
                    },
                    _ => unreachable!(),
                }*/
            },
            _ => (),
        },
        _ => (),
    }
    None
}
*/
fn foo() {
    /*
    let brush = Brush::solid_rgb(0x44, 0x77, 0xFF).unwrap();
    let icon = unsafe { LoadIconW(GetModuleHandleW(null_mut()), MAKEINTRESOURCEW(2)) };
    if icon.is_null() {
        Error::get_last_error().die("Failed to load icon");
    }
    let class = ClassBuilder::new().background(brush).icon(icon).name("WNDBUN").register().expect("Failed to register class");
    let wb = WindowBuilder::new().class(class).handler(|e| {
        println!("{:?}", e);
        None
    });
    let window = wb.create_message().unwrap();
    loop {
        sleep(Duration::from_secs(10));
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
    SetLastError(0);
    let err = Shell_NotifyIconW(NIM_ADD, &mut nid);
    if err == 0 {
        Error::get_last_error().die("Failed to create notification icon");
    }
    let err = Shell_NotifyIconW(NIM_SETVERSION, &mut nid);
    if err == 0 {
        Error::get_last_error().die("Failed to set version for notification icon");
    }
    wndproc::message_loop();
    let err = Shell_NotifyIconW(NIM_DELETE, &mut nid);
    if err == 0 {
        Error::get_last_error().die("Failed to destroy notification icon");
    }
    */
}

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
