mod hook_old;

use std::ffi::c_void;
use std::iter::once;
use std::ptr::{null, null_mut};
use anyhow::ensure;
use detour3::RawDetour;
use once_cell::sync::OnceCell;
use windows_sys::{s, w};
use windows_sys::Win32::Foundation::{BOOL, FALSE, HMODULE, HWND, TRUE};
use windows_sys::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::SystemServices::*;
use windows_sys::Win32::System::Threading::CreateThread;
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW, SET_WINDOW_POS_FLAGS};

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

type WinPosSig = extern "system" fn(HWND, HWND, i32, i32, i32, i32, SET_WINDOW_POS_FLAGS) -> BOOL;
static HOOK: OnceCell<RawDetour> = OnceCell::new();

#[allow(non_snake_case)]
unsafe extern "system" fn SetWindowPos(hwnd: HWND, _hwndinsertafter: HWND, x: i32, y: i32, cx: i32, cy: i32, uflags: SET_WINDOW_POS_FLAGS) -> BOOL {
    let original: WinPosSig = std::mem::transmute(HOOK.get_unchecked().trampoline());
    original(hwnd, 0, x, y, cx, cy, uflags)
}

unsafe fn hook() -> anyhow::Result<()> {
    let user32_lib = GetModuleHandleW(w!("user32.dll"));
    ensure!(user32_lib != 0);

    let set_window_pos = GetProcAddress(user32_lib, s!("SetWindowPos"));
    ensure!(set_window_pos.is_some());
    let set_window_pos = set_window_pos.unwrap();

    let offset = (((SetWindowPos as *mut isize) as isize - (set_window_pos as *mut isize) as isize) - 5) as usize;
    println!("{}", offset);

    HOOK.get_or_try_init(|| RawDetour::new(set_window_pos as *const (), SetWindowPos as *const ()))?.enable()?;
    Ok(())
}

unsafe extern "system" fn attachment_thread(_lpthreadparameter: *mut c_void) -> u32 {
    let result = format!("{:?}", hook())
        .encode_utf16()
        .chain(once(0u16))
        .collect::<Vec<u16>>();
    MessageBoxW(0, result.as_ptr(), w!("Hook result"), MB_OK);
    0
}

/*
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use anyhow::ensure;
use detour3::{RawDetour};
use once_cell::sync::OnceCell;
use windows_sys::{s, w};
use windows_sys::Win32::Foundation::{BOOL, HMODULE, HWND};
use windows_sys::Win32::System::LibraryLoader::{DisableThreadLibraryCalls, GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows_sys::Win32::System::Threading::CreateThread;
use windows_sys::Win32::UI::WindowsAndMessaging::{SET_WINDOW_POS_FLAGS, SWP_NOZORDER};

#[no_mangle]
pub extern "stdcall" fn DllMain(hinst_dll: HMODULE, fdw_reason: u32, _lpv_reserved:  *mut c_void) -> i32 {
    match fdw_reason {
        // The .dll has been loaded
        DLL_PROCESS_ATTACH => {
            unsafe {
                DisableThreadLibraryCalls(hinst_dll);
                // Create a thread that executes our dll_attach function
                CreateThread(
                    null(),
                    0,
                    Some(dll_process_attach_event),
                    hinst_dll as _,
                    0,
                    null_mut(),
                );
            }
            return 1i32;
        }
        // ignore for now
        _ => 1i32,
    }
}

type WinPosSig = extern "system" fn(HWND, HWND, i32, i32, i32, i32, SET_WINDOW_POS_FLAGS) -> BOOL;
static HOOK: OnceCell<RawDetour> = OnceCell::new();

unsafe extern "system" fn SetWindowPos(hwnd: HWND, hwndinsertafter: HWND, x: i32, y: i32, cx: i32, cy: i32, uflags: SET_WINDOW_POS_FLAGS) -> BOOL {
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

    HOOK.get_or_try_init(|| RawDetour::new(set_window_pos as *const (), SetWindowPos as *const ()))?.enable()?;
    Ok(())
}

 */