use std::ffi::c_void;
use anyhow::ensure;
use detour3::{RawDetour};
use once_cell::sync::OnceCell;
use windows_sys::{s, w};
use windows_sys::Win32::Foundation::{BOOL, HMODULE, HWND};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS;

#[no_mangle]
pub extern "stdcall" fn DllMain(_hinst_dll: HMODULE, fdw_reason: u32, _lpv_reserved:  *mut c_void) -> i32 {
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                if let Err(err) = hook() {
                    let _ = std::fs::write("C:\\client-hook.log", format!("{:?}", err));
                }
            }
            return 1i32;
        }
        _ => 1i32,
    }
}

type WinPosSig = extern "system" fn(HWND, HWND, i32, i32, i32, i32, SET_WINDOW_POS_FLAGS) -> BOOL;
static HOOK: OnceCell<RawDetour> = OnceCell::new();

unsafe extern "system" fn set_window_pos_detour(hwnd: HWND, _hwndinsertafter: HWND, x: i32, y: i32, cx: i32, cy: i32, uflags: SET_WINDOW_POS_FLAGS) -> BOOL {
    let original: WinPosSig = std::mem::transmute(HOOK.get_unchecked().trampoline());
    original(hwnd, 0, x, y, cx, cy, uflags)
}

unsafe extern "system" fn dll_process_attach_event(_lpthreadparameter: *mut c_void) -> u32 {
    hook().expect("fdggdf");
    return 0;
}

unsafe fn hook() -> anyhow::Result<()> {
    let user32_lib = GetModuleHandleW(w!("user32.dll"));
    ensure!(user32_lib != 0);

    let set_window_pos = GetProcAddress(user32_lib, s!("SetWindowPos"));
    ensure!(set_window_pos.is_some());
    let set_window_pos = set_window_pos.unwrap();

    HOOK.get_or_try_init(|| RawDetour::new(set_window_pos as *const (), set_window_pos_detour as *const ()))?.enable()?;
    Ok(())
}