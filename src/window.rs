use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::mem::forget;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::thread::spawn;
use winapi::shared::minwindef::{LPARAM, UINT, WPARAM};
use winapi::shared::windef::{HWND};
use winapi::shared::winerror::ERROR_INVALID_WINDOW_HANDLE;
use winapi::um::winuser::{
    CW_USEDEFAULT, CreateWindowExW, GetWindowLongPtrW, HWND_MESSAGE, PostMessageW, SetWindowLongPtrW,
};

use Error;
use class::Class;
use event::{Event, EventResponse};
use notifyicon::NotifyIcon;
use wide::ToWide;
use wndproc::message_loop;
// Because we cannot assign state to the window until after it is created, and the window procedure
// is called during creation, this means that we are unable to assign the user's custom event
// handler to the window from the code creating the window. Instead we store the user event handler
// in a thread local, and when the window procedure is first called it notices the lack of state
// and takes the user handler from the thread local and assigns the state.
thread_local!{
    static WINDOW_HANDLER: Cell<Option<Box<Fn(Event, &Window) -> Option<EventResponse> + Send>>>
        = Cell::new(None);
}
// An HWND can be destroyed from under us at any time, so in order to prevent other threads from
// accessing an invalid HWND, they only have a Weak reference to it. This is still racy, but it's
// fine as user32's HWND allocation strategy ensures the same exact HWND won't be reused for a new
// window any time soon.
#[derive(Clone)]
pub struct WindowRef {
    hwnd: Weak<HWND>,
}
impl WindowRef {
    pub fn as_raw(&self) -> Result<HWND, Error> {
        self.hwnd.upgrade().map(|x| *x).ok_or(Error::from_raw(ERROR_INVALID_WINDOW_HANDLE))
    }
    pub unsafe fn post_message(
        &self, msg: UINT, wparam: WPARAM, lparam: LPARAM,
    ) -> Result<(), Error> {
        if PostMessageW(self.as_raw()?, msg, wparam, lparam) == 0 {
            return Err(Error::get_last_error())
        }
        Ok(())
    }
    pub fn with_window<T, R>() -> Result<R, Error> where T: FnOnce(Window) -> Option<R> + Send {
        unimplemented!()
    }
}
unsafe impl Send for WindowRef {}
struct WindowInternal {
    hwnd: Arc<HWND>,
    handler: Box<Fn(Event, &Window) -> Option<EventResponse> + Send>,
    class: Cell<Option<Class>>,
    nicons: RefCell<HashMap<u16, NotifyIcon>>,
}
impl Drop for WindowInternal {
    fn drop(&mut self) {
        // Might be something in the future
    }
}
pub struct Window(Rc<WindowInternal>);
impl Window {
    pub(crate) fn initialize(hwnd: HWND) -> Result<Window, Error> {
        let handler = WINDOW_HANDLER.with(|x| {
            x.replace(None).expect("Attempted to initialize window without handler")
        });
        let internal = Rc::new(WindowInternal {
            hwnd: Arc::new(hwnd),
            handler: handler,
            class: Cell::new(None),
            nicons: RefCell::new(HashMap::new()),
        });
        let win = Window(internal.clone());
        let rc = Rc::into_raw(internal);
        Error::clear();
        let prev = unsafe { SetWindowLongPtrW(hwnd, 0, rc as isize) };
        let err = Error::get_last_error();
        if prev != 0 {
            unreachable!("Attempted to initialize window long ptr which already had a non-zero value");
        }
        if err.as_raw() != 0 {
            return Err(err);
        }
        Ok(win)
    }
    pub(crate) fn add_nicon(&self, ni: NotifyIcon, id: u16) {
        assert!(!self.0.nicons.borrow().contains_key(&id)); //TODO handling duplicate ids
        self.0.nicons.borrow_mut().insert(id, ni);
    }
    pub fn as_ref(&self) -> WindowRef {
        WindowRef {
            hwnd: Arc::downgrade(&self.0.hwnd),
        }
    }
    pub fn as_raw(&self) -> HWND {
        *self.0.hwnd
    }
    pub(crate) unsafe fn from_raw(hwnd: HWND) -> Result<Option<Window>, Error> {
        Error::clear();
        let raw = GetWindowLongPtrW(hwnd, 0) as *const WindowInternal;
        let err = Error::get_last_error();
        if raw.is_null() {
            if err.as_raw() != 0 {
                return Err(Error::get_last_error());
            }
            return Ok(None);
        }
        let rc = Rc::from_raw(raw);
        forget(rc.clone());
        Ok(Some(Window(rc)))
    }
    pub(crate) fn handle_event(
        &self, msg: UINT, wparam: WPARAM, lparam: LPARAM,
    ) -> Option<EventResponse> {
        let event = unsafe { Event::from_raw(msg, wparam, lparam) };
        match event {
            Event::NotifyIcon(id, e) => {
                self.0.nicons.borrow()[&id].handle_event(e)
            },
            _ => (self.0.handler)(event, self),
        }
    }
}
pub struct WindowBuilder {
    handler: Option<Box<Fn(Event, &Window) -> Option<EventResponse> + Send>>,
    class: Option<Class>,
}
impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            handler: None,
            class: None,
        }
    }
    pub fn handler<T>(
        mut self, handler: T
    ) -> WindowBuilder where T: Fn(Event, &Window) -> Option<EventResponse> + Send + 'static {
        self.handler = Some(Box::new(handler));
        self
    }
    pub fn class(mut self, class: Class) -> WindowBuilder {
        self.class = Some(class);
        self
    }
    pub fn create_child(self, _window: WindowRef) -> Result<WindowRef, Error> {
        unimplemented!()
    }
    pub fn create_message(self) -> Result<WindowRef, Error> {
        let class = self.class.expect("Must specify a class");
        let handler = self.handler.unwrap_or_else(|| Box::new(|_, _| None));
        let pair: Arc<(Mutex<Option<Result<WindowRef, Error>>>, Condvar)>
            = Arc::new((Mutex::new(None), Condvar::new()));
        let rpair = pair.clone();
        spawn(move|| {
            WINDOW_HANDLER.with(|x| x.set(Some(handler)));
            let hwnd = unsafe { CreateWindowExW(
                0, class.as_wstr(),
                "I'm a window!".to_wide_null().as_ptr(),
                0,
                CW_USEDEFAULT, CW_USEDEFAULT,
                0, 0,
                HWND_MESSAGE, null_mut(), null_mut(), null_mut(),
            )};
            if hwnd.is_null() {
                let err = Error::get_last_error();
                *rpair.0.lock().unwrap() = Some(Err(err));
                rpair.1.notify_one();
                return;
            }
            let window = unsafe { Window::from_raw(hwnd).unwrap().unwrap() };
            window.0.class.set(Some(class));
            let remote = window.as_ref();
            *rpair.0.lock().unwrap() = Some(Ok(remote));
            rpair.1.notify_one();
            message_loop();
        });
        let mut result = pair.0.lock().unwrap();
        while result.is_none() {
            result = pair.1.wait(result).unwrap();
        }
        result.take().unwrap()
    }
}
