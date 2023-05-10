use std::sync::atomic::{AtomicU32, Ordering};
use widestring::{U16String};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE, WPARAM};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::*;
use num_enum::FromPrimitive;


unsafe fn get_ex_style(hwnd: HWND) -> WINDOW_EX_STYLE {
    WINDOW_EX_STYLE(GetWindowLongW(hwnd, GWL_EXSTYLE) as u32)
}

unsafe fn get_style(hwnd: HWND) -> WINDOW_STYLE {
    WINDOW_STYLE(GetWindowLongW(hwnd, GWL_STYLE) as u32)
}


//fn is_topmost(style: WINDOW_EX_STYLE) -> bool {
//    style & WS_EX_TOPMOST == WS_EX_TOPMOST
//}

unsafe fn get_title(hwnd: HWND) -> U16String {
    let len = GetWindowTextLengthW(hwnd) + 1;
    let mut string =  vec![0u16; len as usize];
    GetWindowTextW(hwnd, string.as_mut_slice());
    string.pop();
    U16String::from_vec(string)
}

unsafe extern "system" fn hook_proc(_: HWINEVENTHOOK, event: u32, hwnd: HWND, _idobject: i32, _idchild: i32, _ideventthread: u32, _dwmseventtime: u32) {
    //println!("event: {}", event);
    if hwnd.0 != 0 {
        println!("Window {:?} ({:X}) ({:X}): {:?}", get_title(hwnd), get_style(hwnd).0, get_ex_style(hwnd).0, Event::from(event));
    }

}

static MAIN_THREAD: AtomicU32 = AtomicU32::new(0);
unsafe extern "system" fn ctrl_handler(_: u32) -> BOOL {
    let thread = MAIN_THREAD.load(Ordering::Acquire);
    if thread != 0 {
        PostThreadMessageW(thread, WM_QUIT, WPARAM::default(), LPARAM::default());
    }
    TRUE
}

fn main() {
    unsafe {
        println!("Hello, world!");

        MAIN_THREAD.store(GetCurrentThreadId(), Ordering::Release);

        let hook = SetWinEventHook(EVENT_MIN, EVENT_MAX, None, Some(hook_proc), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS);
        assert_ne!(hook.0, 0);
        SetConsoleCtrlHandler(Some(ctrl_handler), TRUE).expect("Could not set console handler");

        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        println!("Stopping");

        UnhookWinEvent(hook);


    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
#[repr(u32)]
enum Event {
    #[num_enum(default)]
    Unknown = 0,
    ConsoleCaret = 16385u32,
    ConsoleEnd = 16639u32,
    ConsoleEndApplication = 16391u32,
    ConsoleLayout = 16389u32,
    ConsoleStartApplication = 16390u32,
    ConsoleUpdateRegion = 16386u32,
    ConsoleUpdateScroll = 16388u32,
    ConsoleUpdateSimple = 16387u32,
    ObjectAcceleratorchange = 32786u32,
    ObjectCloaked = 32791u32,
    ObjectContentscrolled = 32789u32,
    ObjectCreate = 32768u32,
    ObjectDefactionchange = 32785u32,
    ObjectDescriptionchange = 32781u32,
    ObjectDestroy = 32769u32,
    ObjectDragcancel = 32802u32,
    ObjectDragcomplete = 32803u32,
    ObjectDragdropped = 32806u32,
    ObjectDragenter = 32804u32,
    ObjectDragleave = 32805u32,
    ObjectDragstart = 32801u32,
    ObjectEnd = 33023u32,
    ObjectFocus = 32773u32,
    ObjectHelpchange = 32784u32,
    ObjectHide = 32771u32,
    ObjectHostedobjectsinvalidated = 32800u32,
    ObjectImeChange = 32809u32,
    ObjectImeHide = 32808u32,
    ObjectImeShow = 32807u32,
    ObjectInvoked = 32787u32,
    ObjectLiveregionchanged = 32793u32,
    ObjectLocationchange = 32779u32,
    ObjectNamechange = 32780u32,
    ObjectParentchange = 32783u32,
    ObjectReorder = 32772u32,
    ObjectSelection = 32774u32,
    ObjectSelectionadd = 32775u32,
    ObjectSelectionremove = 32776u32,
    ObjectSelectionwithin = 32777u32,
    ObjectShow = 32770u32,
    ObjectStatechange = 32778u32,
    ObjectTexteditConversiontargetchanged = 32816u32,
    ObjectTextselectionchanged = 32788u32,
    ObjectUncloaked = 32792u32,
    ObjectValuechange = 32782u32,
    SystemAlert = 2u32,
    SystemArrangmentpreview = 32790u32,
    SystemCaptureend = 9u32,
    SystemCapturestart = 8u32,
    SystemContexthelpend = 13u32,
    SystemContexthelpstart = 12u32,
    SystemDesktopswitch = 32u32,
    SystemDialogend = 17u32,
    SystemDialogstart = 16u32,
    SystemDragdropend = 15u32,
    SystemDragdropstart = 14u32,
    SystemEnd = 255u32,
    SystemForeground = 3u32,
    SystemImeKeyNotification = 41u32,
    SystemMenuend = 5u32,
    SystemMenupopupend = 7u32,
    SystemMenupopupstart = 6u32,
    SystemMenustart = 4u32,
    SystemMinimizeend = 23u32,
    SystemMinimizestart = 22u32,
    SystemMovesizeend = 11u32,
    SystemMovesizestart = 10u32,
    SystemScrollingend = 19u32,
    SystemScrollingstart = 18u32,
    SystemSound = 1u32,
    SystemSwitchend = 21u32,
    SystemSwitcherAppdropped = 38u32,
    SystemSwitcherAppgrabbed = 36u32,
    SystemSwitcherAppovertarget = 37u32,
    SystemSwitcherCancelled = 39u32,
    SystemSwitchstart = 20u32,
}