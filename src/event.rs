
use winapi::shared::minwindef::{DWORD, LOWORD, LPARAM, UINT, WPARAM};
use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};
use winapi::um::winuser::{WM_APP, WM_CONTEXTMENU, WM_DESTROY};

const WM_APP_NOTIFICATION_ICON: u32 = WM_APP + 1;

pub enum Event {
    Destroy,
    NotifyIcon(NotifyIconEvent),
    Unknown(UINT, WPARAM, LPARAM),
}
impl Event {
    pub unsafe fn from_raw(msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Event {
        match msg {
            WM_APP_NOTIFICATION_ICON => Event::NotifyIcon(NotifyIconEvent::from_raw(wparam, lparam)),
            WM_DESTROY => Event::Destroy,
            _ => Event::Unknown(msg, wparam, lparam),
        }
    }
}
pub enum NotifyIconEvent {
    ContextMenu(i32, i32),
    Unknown(WPARAM, LPARAM),
}
impl NotifyIconEvent {
    unsafe fn from_raw(wparam: WPARAM, lparam: LPARAM) -> NotifyIconEvent {
        let x = GET_X_LPARAM(wparam as LPARAM);
        let y = GET_Y_LPARAM(wparam as LPARAM);
        let msg = LOWORD(lparam as DWORD) as UINT;
        match msg {
            WM_CONTEXTMENU => NotifyIconEvent::ContextMenu(x, y),
            _ => NotifyIconEvent::Unknown(wparam, lparam),
        }
    }
}
