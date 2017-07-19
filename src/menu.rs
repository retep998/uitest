
use std::mem::forget;
use std::ops::{Deref, DerefMut};
use std::ptr::{null, null_mut};
use winapi::shared::windef::{HMENU};
use winapi::um::winuser::*;

use Error;
use wide::ToWide;
use window::Window;

pub enum MenuStatus {
    Enabled,
    Disabled,
    Grayed,
}
pub enum MenuCheck {
    Checked,
    Unchecked,
}
pub enum MenuAction {
    Id(u16),
    ChildMenu(PopupMenu),
}
pub struct Menu {
    handle: HMENU,
}
impl Menu {
    pub unsafe fn from_raw(handle: HMENU) -> Menu {
        Menu {
            handle: handle,
        }
    }
    pub fn as_raw(&self) -> HMENU {
        self.handle
    }
    pub fn into_raw(self) -> HMENU {
        let handle = self.handle;
        forget(self);
        handle
    }
    pub fn append_string<'a>(
        &self, string: &str, action: MenuAction, status: MenuStatus, check: MenuCheck,
    ) -> Result<(), Error> {
        let mut flags = 0;
        let action = match action {
            MenuAction::Id(n) => n as usize,
            MenuAction::ChildMenu(menu) => {
                flags |= MF_POPUP;
                menu.into_inner().into_raw() as usize
            },
        };
        match status {
            MenuStatus::Enabled => flags |= MF_ENABLED,
            MenuStatus::Disabled => flags |= MF_DISABLED,
            MenuStatus::Grayed => flags |= MF_GRAYED,
        }
        match check {
            MenuCheck::Checked => flags |= MF_CHECKED,
            MenuCheck::Unchecked => flags |= MF_UNCHECKED,
        }
        if unsafe { AppendMenuW(
            self.handle, flags | MF_STRING, action, string.to_wide_null().as_ptr(),
        )} == 0 {
            return Err(Error::get_last_error());
        }
        Ok(())
    }
    pub fn append_separator(&self)-> Result<(), Error> {
        if unsafe { AppendMenuW(
            self.handle, MF_SEPARATOR, 0, null(),
        )} == 0 {
            return Err(Error::get_last_error());
        }
        Ok(())
    }
}
impl Drop for Menu {
    fn drop(&mut self) {
        if unsafe { DestroyMenu(self.handle) } == 0 {
            Error::get_last_error().die("Failed to destroy menu");
        }
    }
}
pub struct PopupMenu(Menu);
impl PopupMenu {
    pub fn new() -> Result<PopupMenu, Error> {
        let menu = unsafe { CreatePopupMenu() };
        if menu.is_null() {
            return Err(Error::get_last_error());
        }
        Ok(PopupMenu(unsafe { Menu::from_raw(menu) }))
    }
    pub fn display<T>(
        &self, window: &Window, x: i32, y: i32, func: T,
    ) -> Result<(), Error> where T: FnMut(u16, &Window) + 'static {
        unsafe {
            if SetForegroundWindow(window.as_raw()) == 0 {
                return Err(Error::get_last_error());
            }
            window.set_menu_handler(Box::new(func));
            if TrackPopupMenuEx(self.handle, 0, x, y, window.as_raw(), null_mut()) == 0 {
                return Err(Error::get_last_error());
            }
            Ok(())
        }
    }
    fn into_inner(self) -> Menu {
        self.0
    }
}
impl Deref for PopupMenu {
    type Target = Menu;
    fn deref(&self) -> &Menu {
        &self.0
    }
}
impl DerefMut for PopupMenu {
    fn deref_mut(&mut self) -> &mut Menu {
        &mut self.0
    }
}
pub struct MenuBar(Menu);
impl Deref for MenuBar {
    type Target = Menu;
    fn deref(&self) -> &Menu {
        &self.0
    }
}
impl DerefMut for MenuBar {
    fn deref_mut(&mut self) -> &mut Menu {
        &mut self.0
    }
}
