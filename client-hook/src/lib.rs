#![no_std]
#![allow(clippy::missing_safety_doc)]

#[link(name = "vcruntime")]
#[link(name = "ucrt")]
#[link(name = "msvcrt")]
extern "C" {}

mod import;
mod utils;

use core::ffi::c_void;
use core::panic;
use core::ptr::{null, null_mut};

use crossbeam_utils::atomic::AtomicCell;
use windows_sys::w;
use windows_sys::Win32::Foundation::{BOOL, FALSE, HMODULE, HWND, TRUE};
use windows_sys::Win32::System::LibraryLoader::DisableThreadLibraryCalls;
use windows_sys::Win32::System::SystemServices::*;
use windows_sys::Win32::System::Threading::{CreateThread, ExitProcess};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, SET_WINDOW_POS_FLAGS, SWP_NOZORDER};

use crate::import::find_function_iat;
use crate::utils::{write_protected, Error, IntPtr};

#[panic_handler]
fn panic(_info: &panic::PanicInfo) -> ! {
    unsafe { ExitProcess(1) }
}

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
        DLL_PROCESS_DETACH => unhook(),
        _ => {}
    }
    TRUE
}

unsafe extern "system" fn attachment_thread(_lpthreadparameter: *mut c_void) -> u32 {
    if let Err(err) = hook() {
        MessageBoxW(0, err.msg(), w!("Hook Failed"), MB_OK);
    }
    0
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
static POS_FUNC_PTR: AtomicCell<IntPtr> = AtomicCell::new(IntPtr::null());
static OLD_POS_FUNC: AtomicCell<Option<SetPosProto>> = AtomicCell::new(None);
#[allow(non_snake_case)]
pub unsafe extern "system" fn SetWindowPos(hwnd: HWND, hwndinsertafter: HWND, x: i32, y: i32, cx: i32, cy: i32, uflags: SET_WINDOW_POS_FLAGS) -> BOOL {
    OLD_POS_FUNC
        .load()
        .map(|func| func(hwnd, hwndinsertafter, x, y, cx, cy, uflags | SWP_NOZORDER))
        .unwrap_or(FALSE)
}

unsafe fn hook() -> Result<(), Error> {
    let func_ptr = find_function_iat(b"user32.dll", b"SetWindowPos")
        .or_else(|_| find_function_iat(b"USER32.dll", b"SetWindowPos"))?;
    POS_FUNC_PTR.store(func_ptr);
    OLD_POS_FUNC.store(func_ptr.read());
    write_protected(func_ptr.as_ptr(), SetWindowPos as usize)?;
    Ok(())
}

unsafe fn unhook() {
    if let Some(func) = OLD_POS_FUNC.load() {
        let ptr = POS_FUNC_PTR.load();
        if ptr.is_not_null() {
            let _ = write_protected(ptr.as_ptr(), func as usize);
        }
    }
}
