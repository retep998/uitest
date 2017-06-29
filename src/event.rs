
use winapi::shared::minwindef::{DWORD, LOWORD, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};
use winapi::um::winuser::{WM_APP};

const WM_APP_NOTIFICATION_ICON: u32 = WM_APP + 1;

#[derive(Debug)]
pub enum Event {
    Destroy,
    Unknown(UINT, WPARAM, LPARAM),
    NotifyIcon(NotifyIconEvent),
}
impl Event {
    pub unsafe fn from_raw(msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Event {
        use winapi::um::winuser as wu;
        match msg {
            wu::WM_DESTROY => Event::Destroy,
            WM_APP_NOTIFICATION_ICON => Event::NotifyIcon(
                NotifyIconEvent::from_raw(wparam, lparam)
            ),
            _ => Event::Unknown(msg, wparam, lparam),
        }
    }
}
#[derive(Debug)]
pub enum NotifyIconEvent {
    ContextMenu(i32, i32),
    Unknown(WPARAM, LPARAM),
}
impl NotifyIconEvent {
    unsafe fn from_raw(wparam: WPARAM, lparam: LPARAM) -> NotifyIconEvent {
        use winapi::um::winuser as wu;
        let x = GET_X_LPARAM(wparam as LPARAM);
        let y = GET_Y_LPARAM(wparam as LPARAM);
        let msg = LOWORD(lparam as DWORD) as UINT;
        match msg {
            wu::WM_CONTEXTMENU => NotifyIconEvent::ContextMenu(x, y),
            _ => NotifyIconEvent::Unknown(wparam, lparam),
        }
    }
}
pub struct EventResponse(LRESULT);
impl EventResponse {
    pub unsafe fn from_raw(x: LRESULT) -> EventResponse {
        EventResponse(x)
    }
    pub fn as_raw(&self) -> LRESULT {
        self.0
    }
}
pub type EventHandler = Fn(Event) -> Option<EventResponse> + Send;
