
use std::cell::RefCell;
use std::mem::{size_of, zeroed};
use std::rc::Rc;
use winapi::um::shellapi::{
    NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
    NOTIFYICONDATAW, NOTIFYICON_VERSION_4, Shell_NotifyIconW,
};

use Error;
use event::{EventResponse, NotifyIconEvent, WM_APP_NOTIFICATION_ICON};
use icon::Icon;
use wide::ToWide;
use window::Window;

struct NotifyIconInternal {
    nid: RefCell<NOTIFYICONDATAW>,
    handler: Box<Fn(NotifyIconEvent, &Window) -> Option<EventResponse>>,
}
impl Drop for NotifyIconInternal {
    fn drop(&mut self) {
        let err = unsafe { Shell_NotifyIconW(NIM_DELETE, &mut *self.nid.borrow_mut()) };
        if err == 0 {
            Error::get_last_error().die("Failed to delete notification icon");
        }
    }
}
#[derive(Clone)]
pub struct NotifyIcon(Rc<NotifyIconInternal>);
impl NotifyIcon {
    pub fn handle_event(&self, e: NotifyIconEvent, w: &Window) -> Option<EventResponse> {
        (self.0.handler)(e, w)
    }
    pub fn id(&self) -> u16 {
        self.0.nid.borrow().uID as u16
    }
    fn set_version(&self) -> Result<(), Error> {
        let err = unsafe { Shell_NotifyIconW(NIM_SETVERSION, &mut *self.0.nid.borrow_mut()) };
        if err == 0 {
            return Err(Error::get_last_error());
        }
        Ok(())
    }
    fn enable_messages(&self) -> Result<(), Error> {
        self.0.nid.borrow_mut().uFlags |= NIF_MESSAGE;
        self.modify()
    }
    fn modify(&self) -> Result<(), Error> {
        let err = unsafe { Shell_NotifyIconW(NIM_MODIFY, &mut *self.0.nid.borrow_mut()) };
        if err == 0 {
            return Err(Error::get_last_error());
        }
        Ok(())
    }
    pub fn set_tooltip(&self, msg: &str) -> Result<(), Error> {
        let msg = msg.to_wide();
        {
            let mut nid = self.0.nid.borrow_mut();
            nid.szTip = [0; 128];
            nid.szTip[..msg.len()].copy_from_slice(&msg);
            nid.uFlags |= NIF_TIP | NIF_SHOWTIP;
        }
        self.modify()
    }
}
pub struct NotifyIconBuilder {
    icon: Option<Icon>,
    id: Option<u16>,
    handler: Option<Box<Fn(NotifyIconEvent, &Window) -> Option<EventResponse>>>,
}
impl NotifyIconBuilder {
    pub fn new() -> NotifyIconBuilder {
        NotifyIconBuilder {
            icon: None,
            id: None,
            handler: None,
        }
    }
    pub fn icon(mut self, icon: Icon) -> NotifyIconBuilder {
        self.icon = Some(icon);
        self
    }
    pub fn id(mut self, id: u16) -> NotifyIconBuilder {
        assert!(id != 0, "Notification Icon ID must be non-zero");
        self.id = Some(id);
        self
    }
    pub fn handler<T>(
        mut self, handler: T
    ) -> NotifyIconBuilder where T: Fn(NotifyIconEvent, &Window) -> Option<EventResponse> + 'static {
        self.handler = Some(Box::new(handler));
        self
    }
    pub fn create(self, window: &Window) -> Result<NotifyIcon, Error> {
        unsafe {
            let mut nid: NOTIFYICONDATAW = zeroed();
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = window.as_raw();
            nid.uCallbackMessage = WM_APP_NOTIFICATION_ICON;
            nid.uID = self.id.unwrap() as u32;
            if let Some(icon) = self.icon {
                nid.uFlags |= NIF_ICON;
                nid.hIcon = icon.into_raw();
            }
            *nid.uVersion_mut() = NOTIFYICON_VERSION_4;
            Error::clear();
            let err = Shell_NotifyIconW(NIM_ADD, &mut nid);
            if err == 0 {
                return Err(Error::get_last_error());
            }
            let ni = NotifyIcon(Rc::new(NotifyIconInternal {
                nid: RefCell::new(nid),
                handler: self.handler.unwrap_or_else(|| Box::new(|_, _| None)),
            }));
            ni.set_version()?;
            ni.enable_messages()?;
            window.add_nicon(ni.clone(), ni.id());
            Ok(ni)
        }
    }
}
