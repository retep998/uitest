use std::ptr::null_mut;
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, Ordering};
use winapi::shared::minwindef::{LPARAM, UINT, WPARAM};
use winapi::shared::windef::{HWND, HWND__};
use winapi::shared::winerror::ERROR_INVALID_WINDOW_HANDLE;
use winapi::um::winnt::{LPCWSTR};
use winapi::um::winuser::{CW_USEDEFAULT, CreateWindowExW, PostMessageW};

use Error;
use class::Class;
use wide::ToWide;

#[derive(Clone)]
pub struct RemoteWindow {
    hwnd: Arc<AtomicPtr<HWND__>>,
}
impl RemoteWindow {
    fn as_raw(&self) -> Result<HWND, Error> {
        let hwnd = self.hwnd.load(Ordering::SeqCst);
        if hwnd.is_null() {
            Err(Error::from_raw(ERROR_INVALID_WINDOW_HANDLE))
        } else {
            Ok(hwnd)
        }
    }
    unsafe fn post_message(&self, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Result<(), Error> {
        if PostMessageW(self.as_raw()?, msg, wparam, lparam) == 0 {
            return Err(Error::get_last_error())
        }
        Ok(())
    }
}
pub struct Window {
    hwnd: Arc<AtomicPtr<HWND__>>,
    #[allow(dead_code)] class: Option<Class>,
}
impl Window {
    pub fn remote(&self) -> RemoteWindow {
        RemoteWindow {
            hwnd: self.hwnd.clone(),
        }
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        self.hwnd.store(null_mut(), Ordering::SeqCst);
    }
}
pub struct WindowBuilder {
    class: Option<Class>,
}
impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        unimplemented!()
    }
    pub fn class(mut self, class: Class) -> WindowBuilder {
        self.class = Some(class);
        self
    }
    pub fn create_child(self, _window: RemoteWindow) -> Result<Window, Error> {
        unimplemented!()
    }
    pub fn create_message(self) -> Result<Window, Error> {
        let atom = self.class.as_ref().map(|x| x.as_raw()).unwrap_or(0);
        let hwnd = unsafe { CreateWindowExW(
            0, atom as LPCWSTR,
            "I'm a window!".to_wide_null().as_ptr(),
            0,
            CW_USEDEFAULT, CW_USEDEFAULT,
            0, 0,
            null_mut(), null_mut(), null_mut(), null_mut(),
        )};
        if hwnd.is_null() {
            return Err(Error::get_last_error())
        }
        Ok(Window {
            hwnd: Arc::new(AtomicPtr::new(hwnd)),
            class: self.class,
        })
    }
}
