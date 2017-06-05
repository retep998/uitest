
use std::mem::{forget, size_of_val, zeroed};
use std::ptr::null_mut;
use winapi::shared::minwindef::{ATOM, DWORD};
use winapi::shared::windef::HICON;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winnt::LPCWSTR;
use winapi::um::winuser::{LoadIconW, MAKEINTRESOURCEW, RegisterClassExW, WNDCLASSEXW, UnregisterClassW};

use Error;
use brush::Brush;
use wide::ToWide;

pub struct Class(ATOM);
impl Class {
    pub fn as_raw(&self) -> ATOM {
        self.0
    }
}
impl Drop for Class {
    fn drop(&mut self) {
        if unsafe { UnregisterClassW(self.0 as LPCWSTR, null_mut()) } == 0 {
            Error::get_last_error().die("Failed to unregister class");
        }
    }
}
pub struct ClassBuilder {
    name: Option<Vec<u16>>,
    brush: Option<Brush>,
    icon: Option<HICON>,
}
impl ClassBuilder {
    pub fn new() -> ClassBuilder {
        let mut class: WNDCLASSEXW = unsafe { zeroed() };
        ClassBuilder {
            name: None,
            brush: None,
            icon: None,
        }
    }
    pub fn name(mut self, name: &str) -> ClassBuilder {
        let name = name.to_wide_null();
        self.name = Some(name);
        self
    }
    // TODO Add Icon type
    pub fn icon(mut self) -> ClassBuilder {
        let icon = unsafe { LoadIconW(GetModuleHandleW(null_mut()), MAKEINTRESOURCEW(2)) };
        if icon.is_null() {
            Error::get_last_error().die("Failed to load icon");
        }
        self.icon = Some(icon);
        self
    }
    pub fn background(mut self, brush: Brush) -> ClassBuilder {
        self.brush = Some(brush);
        self
    }
    pub fn register(self) -> Result<Class, Error> {
        let mut class: WNDCLASSEXW = unsafe { zeroed() };
        class.cbSize = size_of_val(&class) as DWORD;
        class.lpfnWndProc = Some(::wndproc);
        if let Some(ref name) = self.name {
            class.lpszClassName = name.as_ptr();
        }
        if let Some(ref brush) = self.brush {
            class.hbrBackground = brush.as_raw();
        }
        let atom = unsafe { RegisterClassExW(&class) };
        if atom == 0 {
            return Err(Error::get_last_error());
        }
        forget(self.brush);
        Ok(Class(atom))
    }
}
