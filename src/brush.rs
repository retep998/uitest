
use std::mem::forget;
use winapi::shared::windef::{HBRUSH, HGDIOBJ};
use winapi::um::wingdi::{CreateSolidBrush, DeleteObject};

use Error;

pub struct Brush(HBRUSH);
impl Brush {
    pub fn as_raw(&self) -> HBRUSH {
        self.0
    }
    pub fn into_raw(self) -> HBRUSH {
        let x = self.0;
        forget(self);
        x
    }
    pub fn solid_rgb(r: u8, g: u8, b: u8) -> Result<Brush, Error> {
        let rgb = (r as u32) | ((g as u32) << 8) | ((b as u32) << 16);
        let brush = unsafe { CreateSolidBrush(rgb) };
        if brush.is_null() {
            return Err(Error::get_last_error());
        }
        Ok(Brush(brush))
    }
}
impl Drop for Brush {
    fn drop(&mut self) {
        if unsafe { DeleteObject(self.0 as HGDIOBJ) } == 0 {
            Error::get_last_error().die("Failed to destroy brush");
        }
    }
}
