
use winapi::shared::minwindef::{DWORD, HIWORD, LOWORD, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windowsx::{GET_X_LPARAM, GET_Y_LPARAM};
use winapi::um::winuser::{CREATESTRUCTW, MINMAXINFO, WM_APP};

pub(crate) const WM_APP_NOTIFICATION_ICON: u32 = WM_APP + 1;

#[derive(Debug)]
pub enum Event {
    MenuCommand(u16),
    Create(*const CREATESTRUCTW),
    Destroy,
    GetMinMaxInfo(*mut MINMAXINFO),
    #[doc(hidden)] Unknown(UINT, WPARAM, LPARAM),
    #[doc(hidden)] NotifyIcon(u16, NotifyIconEvent),
}
impl Event {
    pub unsafe fn from_raw(msg: UINT, wparam: WPARAM, lparam: LPARAM) -> Event {
        use winapi::um::winuser as wu;
        match msg {
            wu::WM_COMMAND if HIWORD(wparam as u32) == 0 => Event::MenuCommand(LOWORD(wparam as u32)),
            wu::WM_CREATE => Event::Create(lparam as *const CREATESTRUCTW),
            wu::WM_DESTROY => Event::Destroy,
            wu::WM_GETMINMAXINFO => Event::GetMinMaxInfo(lparam as *mut MINMAXINFO),
            WM_APP_NOTIFICATION_ICON => Event::NotifyIcon(
                HIWORD(lparam as DWORD),
                NotifyIconEvent::from_raw(wparam, lparam),
            ),
            _ => Event::Unknown(msg, wparam, lparam),
        }
    }
}
#[derive(Debug)]
pub enum NotifyIconEvent {
    ContextMenu(i32, i32),
    MouseMove(i32, i32),
    Select(i32, i32),
    Unknown(u32, i32, i32),
}
impl NotifyIconEvent {
    unsafe fn from_raw(wparam: WPARAM, lparam: LPARAM) -> NotifyIconEvent {
        use winapi::um::winuser as wu;
        use winapi::um::shellapi as sa;
        let x = GET_X_LPARAM(wparam as LPARAM);
        let y = GET_Y_LPARAM(wparam as LPARAM);
        let msg = LOWORD(lparam as DWORD) as UINT;
        match msg {
            sa::NIN_SELECT => NotifyIconEvent::Select(x, y),
            wu::WM_CONTEXTMENU => NotifyIconEvent::ContextMenu(x, y),
            wu::WM_MOUSEMOVE => NotifyIconEvent::MouseMove(x, y),
            _ => NotifyIconEvent::Unknown(msg, x, y),
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
