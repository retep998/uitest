
use std::mem::{size_of};
use std::ptr::null_mut;
use std::sync::Arc;
use winapi::shared::minwindef::{ATOM};
use winapi::shared::windef::HICON;
use winapi::um::winnt::LPCWSTR;
use winapi::um::winuser::{RegisterClassExW, WNDCLASSEXW, UnregisterClassW};

use Error;
use brush::Brush;
use wide::ToWide;
use wndproc::wndproc;

pub struct ClassBuilder {
    name: Vec<u16>,
    background: Option<Brush>,
    icon: Option<HICON>,
}
impl ClassBuilder {
    pub fn new() -> ClassBuilder {
        ClassBuilder {
            name: Vec::new(),
            background: None,
            icon: None,
        }
    }
    pub fn name<T>(mut self, name: &T) -> ClassBuilder where T: ToWide + ?Sized {
        self.name = name.to_wide_null();
        self
    }
    pub fn background(mut self, background: Brush) -> ClassBuilder {
        self.background = Some(background);
        self
    }
    pub fn icon(mut self, icon: HICON) -> ClassBuilder {
        self.icon = Some(icon);
        self
    }
    pub fn register(self) -> Result<Class, Error> {
        assert!(!self.name.is_empty());
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: size_of::<usize>() as i32,
            hInstance: null_mut(),
            hIcon: self.icon.unwrap_or(null_mut()),
            hCursor: null_mut(),
            hbrBackground: self.background.map(|b| b.into_raw()).unwrap_or(null_mut()),
            lpszMenuName: null_mut(),
            lpszClassName: self.name.as_ptr(),
            hIconSm: null_mut(),
        };
        let atom = unsafe { RegisterClassExW(&class) };
        if atom == 0 {
            return Err(Error::get_last_error());
        }
        let class = Class(Arc::new(atom));
        Ok(class)
    }
}
#[derive(Clone)]
pub struct Class(Arc<ATOM>);
impl Class {
    pub fn as_raw(&self) -> ATOM {
        *self.0
    }
    pub fn as_wstr(&self) -> LPCWSTR {
        self.as_raw() as usize as LPCWSTR
    }
}
impl Drop for Class {
    fn drop(&mut self) {
        if unsafe { UnregisterClassW(self.as_raw() as LPCWSTR, null_mut()) } == 0 {
            Error::get_last_error().die("Failed to unregister class");
        }
    }
}
