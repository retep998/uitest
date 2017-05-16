use std::ptr::{null, null_mut};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::*;

use Error;
use wide::ToWide;


pub struct Menu(HMENU);
pub enum MenuItem<'a> {
    String(&'a str),
    Separator,
}
pub enum MenuStatus {
    Enabled,
    Disabled,
    Grayed,
}
pub enum MenuCheck {
    Checked,
    Unchecked,
}
impl Menu {
    pub fn new() -> Result<Menu, Error> {
        let menu = unsafe { CreatePopupMenu() };
        if menu.is_null() {
            return Err(Error::get_last_error());
        }
        Ok(Menu(menu))
    }
    pub fn append<'a>(
        &self, item: MenuItem<'a>, id: u16, status: MenuStatus, check: MenuCheck,
    ) -> Result<(), Error> {
        let mut flags = 0;
        let id = id as usize;
        match status {
            MenuStatus::Enabled => flags |= MF_ENABLED,
            MenuStatus::Disabled => flags |= MF_DISABLED,
            MenuStatus::Grayed => flags |= MF_GRAYED,
        }
        match check {
            MenuCheck::Checked => flags |= MF_CHECKED,
            MenuCheck::Unchecked => flags |= MF_UNCHECKED,
        }
        if unsafe { match item {
            MenuItem::String(string) => AppendMenuW(
                self.0, flags | MF_STRING, id, string.to_wide_null().as_ptr(),
            ),
            MenuItem::Separator => AppendMenuW(
                self.0, flags | MF_SEPARATOR, id, null(),
            ),
        }} == 0 {
            return Err(Error::get_last_error());
        }
        Ok(())
    }
    pub fn display(&self, hwnd: HWND, x: i32, y: i32) -> Result<u16, Error> {
        unsafe {
            let ret = SetForegroundWindow(hwnd);
            if ret == 0 {
                return Err(Error::get_last_error());
            }
            let ret = TrackPopupMenuEx(self.0, TPM_RETURNCMD, x, y, hwnd, null_mut());
            if ret == 0 && GetLastError() != 0 {
                return Err(Error::get_last_error());
            }
            Ok(ret as u16)
        }
    }
}
impl Drop for Menu {
    fn drop(&mut self) {
        if unsafe { DestroyMenu(self.0) } == 0 {
            Error::get_last_error().die("Failed to destroy menu");
        }
    }
}
