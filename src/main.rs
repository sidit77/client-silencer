use std::env::current_exe;
use std::mem::size_of_val;
use sysinfo::{PidExt, ProcessExt, SystemExt};
use widestring::{U16CString};
use windows::imp::GetProcAddress;
use windows::{s, w};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx};
use windows::Win32::System::Threading::*;

fn main() {
    unsafe {
        let pid = sysinfo::System::new_all().processes_by_name("hotkey-timer.exe").next().unwrap().pid().as_u32();

        println!("pid: {}", pid);

        let process_handle = OpenProcess(
            PROCESS_CREATE_THREAD |
                PROCESS_QUERY_INFORMATION |
                PROCESS_VM_OPERATION |
                PROCESS_VM_WRITE |
                PROCESS_VM_READ,
            false, pid
        ).expect("Failed to open process");

        let kernel32 = GetModuleHandleW(w!("kernel32.dll")).unwrap();
        let load_library = GetProcAddress(kernel32.0, s!("LoadLibraryW"));
        println!("{:?}", load_library);
        assert!(!load_library.is_null());
        let mut dll_path = current_exe().unwrap();
        dll_path.pop();
        dll_path.push("client_hook.dll");
        assert!(dll_path.exists());
        let dll_path =  U16CString::from_os_str(&dll_path.into_os_string()).unwrap();
        println!("{} ({})", dll_path.display(), dll_path.len());
        let dll_path = dll_path.as_slice_with_nul();
        println!("{}", size_of_val(dll_path));

        let virtual_alloc_ptr = VirtualAllocEx(
            process_handle,
            None,
            size_of_val(dll_path),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE);
        println!("{:?}", virtual_alloc_ptr);
        assert!(!virtual_alloc_ptr.is_null());


        WriteProcessMemory(
            process_handle,
            virtual_alloc_ptr,
            dll_path.as_ptr() as _,
            size_of_val(dll_path),
            None
        ).unwrap();

        CreateRemoteThread(
            process_handle,
            None,
            0,
            Some(std::mem::transmute(load_library)),
            Some(virtual_alloc_ptr),
            0,
            None
        ).unwrap();

    }
}
