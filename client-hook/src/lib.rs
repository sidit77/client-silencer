#![allow(clippy::missing_safety_doc)]

mod import;
mod utils;

use std::ffi::c_void;
use std::ptr::{null, null_mut};
use once_cell::sync::OnceCell;
use windows_sys::Win32::Foundation::{BOOL, FALSE, HMODULE, HWND, TRUE};
use windows_sys::Win32::System::LibraryLoader::{DisableThreadLibraryCalls};
use windows_sys::Win32::System::SystemServices::*;
use windows_sys::Win32::System::Threading::CreateThread;
use windows_sys::Win32::UI::WindowsAndMessaging::{TIMERPROC};
use crate::import::{find_function_iat, write_protected};
use crate::utils::Error;

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

//#[allow(non_snake_case)]
//unsafe extern "system" fn SetWindowPos(hwnd: HWND, _hwndinsertafter: HWND, x: i32, y: i32, cx: i32, cy: i32, uflags: SET_WINDOW_POS_FLAGS) -> BOOL {
//    //let original: WinPosSig = std::mem::transmute(HOOK.get_unchecked().trampoline());
//    //original(hwnd, 0, x, y, cx, cy, uflags)
//    //MessageBoxW(0, w!("Test"), w!("Title"), MB_OK);
//    println!("Test");
//    TRUE
//}

//#[allow(non_snake_case)]
//pub unsafe extern "system" fn SetTimer(hwnd: HWND, nidevent: usize, uelapse: u32, lptimerfunc: TIMERPROC) -> usize {
//    println!("Test");
//    12
//}

//unsafe fn hook() -> anyhow::Result<()> {
//    let result = find_function_iat(b"user32.dll", b"SetTimer")?;
//    //let user32_lib = GetModuleHandleW(w!("user32.dll"));
//    //ensure!(user32_lib != 0);
////
//    //let set_window_pos = GetProcAddress(user32_lib, s!("SetTimer"));
//    //ensure!(set_window_pos.is_some());
//    //let set_window_pos = set_window_pos.unwrap();
////
//    ////HOOK.get_or_try_init(|| RawDetour::new(set_window_pos as *const (), SetWindowPos as *const ()))?.enable()?;
//    //install_hook(set_window_pos as *const c_void, SetTimer as *const c_void);
//    Ok(())
//}

type TimerProto = extern "system" fn(HWND, usize, u32, TIMERPROC) -> usize;
static HOOK: OnceCell<TimerProto> = OnceCell::new();
#[allow(non_snake_case)]
pub unsafe extern "system" fn SetTimer(hwnd: HWND, nidevent: usize, _uelapse: u32, lptimerfunc: TIMERPROC) -> usize {
    println!("Test");
    HOOK.get_unchecked()(hwnd, nidevent, 1000, lptimerfunc)
}

unsafe fn hook() -> Result<(), Error> {
    let func_ptr = find_function_iat(b"user32.dll", b"SetTimer")?;
    let old: TimerProto = func_ptr.read();
    HOOK.get_or_init(|| old);
    write_protected(func_ptr.as_ptr(), SetTimer as usize);
    Ok(())
}

unsafe extern "system" fn attachment_thread(_lpthreadparameter: *mut c_void) -> u32 {
    //let result = format!("{:?}", hook())
    //    .encode_utf16()
    //    .chain(once(0u16))
    //    .collect::<Vec<u16>>();
    hook().unwrap_or_else(|err| println!("Error: {:?}", err));
    //MessageBoxW(0, result.as_ptr(), w!("Hook result"), MB_OK);
    0
}
