
use std::mem::forget;
use std::ptr::null_mut;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{LoadIconW, MAKEINTRESOURCEW};
use winapi::shared::windef::HICON;

use Error;

pub struct Icon(HICON);

impl Icon {
    pub unsafe fn from_resource(id: u16) -> Result<Icon, Error> {
        let icon = LoadIconW(GetModuleHandleW(null_mut()), MAKEINTRESOURCEW(id));
        if icon.is_null() {
            return Err(Error::get_last_error());
        }
        Ok(Icon(icon))
    }
    pub fn into_raw(self) -> HICON {
        let icon = self.0;
        forget(self);
        icon
    }
}
