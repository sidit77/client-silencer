use std::env::current_exe;
use std::mem::size_of_val;
use std::ptr::{null, null_mut};
use sysinfo::{PidExt, ProcessExt, SystemExt};
use widestring::{U16CString};
use windows_sys::{s, w};
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx};
use windows_sys::Win32::System::Threading::*;

fn main() {
    unsafe {
        let pid = sysinfo::System::new_all().processes_by_name("mpc-be64.exe").next().unwrap().pid().as_u32();

        println!("pid: {}", pid);

        let process_handle = OpenProcess(
            PROCESS_CREATE_THREAD |
                PROCESS_QUERY_INFORMATION |
                PROCESS_VM_OPERATION |
                PROCESS_VM_WRITE |
                PROCESS_VM_READ,
            0, pid
        );
        assert_ne!(process_handle, 0, "Failed to open process");

        let kernel32 = GetModuleHandleW(w!("kernel32.dll"));
        assert_ne!(kernel32, 0, "Failed to get handle for kernel32.dll");
        let load_library = GetProcAddress(kernel32, s!("LoadLibraryW"))
            .expect("Failed to the address of LoadLibraryW");
        let mut dll_path = current_exe()
            .expect("Failed to get exe path");
        dll_path.pop();
        dll_path.push("client_hook.dll");
        println!("Hook path: {}", dll_path.display());
        assert!(dll_path.exists(), "Path the the hook dll is wrong");
        let dll_path =  U16CString::from_os_str(&dll_path.into_os_string())
            .expect("Can not convert path into c string");
        let dll_path = dll_path.as_slice_with_nul();

        let virtual_alloc_ptr = VirtualAllocEx(
            process_handle,
            null(),
            size_of_val(dll_path),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE);
        assert!(!virtual_alloc_ptr.is_null(), "Failed to allocate memory in the target process");


        assert_ne!(WriteProcessMemory(
            process_handle,
            virtual_alloc_ptr,
            dll_path.as_ptr() as _,
            size_of_val(dll_path),
            null_mut()
        ), 0, "Failed to write path to memory");

        assert_ne!(CreateRemoteThread(
            process_handle,
            null(),
            0,
            Some(std::mem::transmute(load_library)),
            virtual_alloc_ptr,
            0,
            null_mut()
        ), 0, "Failed to insert hook");

        println!("Hook installed!");
    }
}
