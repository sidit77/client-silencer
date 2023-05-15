#![allow(clippy::missing_safety_doc)]

mod import;
mod utils;

use std::ffi::c_void;
use std::iter::once;
use std::ptr::{null, null_mut};
use once_cell::sync::OnceCell;
use windows_sys::w;
use windows_sys::Win32::Foundation::{BOOL, FALSE, HMODULE, HWND, TRUE};
use windows_sys::Win32::System::LibraryLoader::{DisableThreadLibraryCalls};
use windows_sys::Win32::System::SystemServices::*;
use windows_sys::Win32::System::Threading::CreateThread;
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW, SET_WINDOW_POS_FLAGS, SWP_NOZORDER};
use crate::import::find_function_iat;
use crate::utils::{Error, write_protected};

#[no_mangle]
pub unsafe extern "stdcall" fn DllMain(hmodule: HMODULE, reason: u32, _: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hmodule);
            let result = CreateThread(
                null(),
                0,
                Some(attachment_thread),
                null(),
                0,
                null_mut()
            );
            if result == 0 {
                return FALSE;
            }
        },
        DLL_PROCESS_DETACH => { },
        _ => { }
    }
    TRUE
}

//type WinPosSig = extern "system" fn(HWND, HWND, i32, i32, i32, i32, SET_WINDOW_POS_FLAGS) -> BOOL;

//type TimerProto = extern "system" fn(HWND, usize, u32, TIMERPROC) -> usize;
//static OLD_TIMER_FUNC: OnceCell<TimerProto> = OnceCell::new();
//#[allow(non_snake_case)]
//pub unsafe extern "system" fn SetTimer(hwnd: HWND, nidevent: usize, _uelapse: u32, lptimerfunc: TIMERPROC) -> usize {
//    println!("Test");
//    OLD_TIMER_FUNC
//        .get()
//        .map(|func| func(hwnd, nidevent, 1000, lptimerfunc))
//        .unwrap_or(0)
//}

//unsafe fn hook() -> Result<(), Error> {
//    let func_ptr = find_function_iat(b"user32.dll", b"SetTimer")?;
//    OLD_TIMER_FUNC.get_or_init(|| unsafe { func_ptr.read() });
//    write_protected(func_ptr.as_ptr(), SetTimer as usize)?;
//    Ok(())
//}

type SetPosProto = extern "system" fn(HWND, HWND, i32, i32, i32, i32, SET_WINDOW_POS_FLAGS) -> BOOL;
static OLD_POS_FUNC: OnceCell<SetPosProto> = OnceCell::new();
#[allow(non_snake_case)]
pub unsafe extern "system" fn SetWindowPos(hwnd: HWND, hwndinsertafter: HWND, x: i32, y: i32, cx: i32, cy: i32, uflags: SET_WINDOW_POS_FLAGS) -> BOOL {
    OLD_POS_FUNC
        .get()
        .map(|func| func(hwnd, hwndinsertafter, x, y, cx, cy, uflags | SWP_NOZORDER))
        .unwrap_or(FALSE)
}

unsafe fn hook() -> Result<(), Error> {
    let func_ptr = find_function_iat(b"user32.dll", b"SetWindowPos")
        .or_else(|_| find_function_iat(b"USER32.dll", b"SetWindowPos"))?;
    OLD_POS_FUNC.get_or_init(|| unsafe { func_ptr.read() });
    write_protected(func_ptr.as_ptr(), SetWindowPos as usize)?;
    Ok(())
}



unsafe extern "system" fn attachment_thread(_lpthreadparameter: *mut c_void) -> u32 {
    if let Err(err) = hook() {
        let result = format!("{:?}", err)
            .encode_utf16()
            .chain(once(0u16))
            .collect::<Vec<u16>>();
        MessageBoxW(0, result.as_ptr(), w!("Hook Failed"), MB_OK);
    }
    0
}
