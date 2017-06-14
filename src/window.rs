use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::thread::spawn;
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
struct WindowInternal {
    hwnd: Arc<AtomicPtr<HWND__>>,
    _class: Option<Class>,
}
impl Drop for WindowInternal {
    fn drop(&mut self) {
        self.hwnd.store(null_mut(), Ordering::SeqCst);
    }
}
pub struct Window(Rc<WindowInternal>);
impl Window {
    pub fn remote(&self) -> RemoteWindow {
        RemoteWindow {
            hwnd: self.0.hwnd.clone(),
        }
    }
    pub(crate) unsafe fn from_raw(hwnd: HWND) -> Window {
        unimplemented!()
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
    pub fn create_child(self, _window: RemoteWindow) -> Result<RemoteWindow, Error> {
        unimplemented!()
    }
    pub fn create_message(self) -> Result<RemoteWindow, Error> {
        let pair: Arc<(Mutex<Option<Result<RemoteWindow, Error>>>, Condvar)> = Arc::new((Mutex::new(None), Condvar::new()));
        let rpair = pair.clone();
        spawn(move|| {
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
                let err = Error::get_last_error();
                *rpair.0.lock().unwrap() = Some(Err(err));
                rpair.1.notify_one();
            }
            let window = WindowInternal {
                hwnd: Arc::new(AtomicPtr::new(hwnd)),
                _class: self.class,
            };
        });
        let mut result = pair.0.lock().unwrap();
        while result.is_none() {
            result = pair.1.wait(result).unwrap();
        }
        unimplemented!()
    }
}
