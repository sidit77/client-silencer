use std::env::current_exe;
use std::mem::{size_of_val, transmute};
use std::ptr::{null, null_mut};
use widestring::{U16CString};
use windows_sys::{s, w};
use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::Threading::*;


fn main() {
    unsafe {
        let pid = 1234;//sysinfo::System::new_all().processes_by_name("mpc-be64.exe").next().unwrap().pid().as_u32();

        println!("pid: {}", pid);

        let process_handle = OpenProcess(
            PROCESS_CREATE_THREAD |
                PROCESS_QUERY_INFORMATION |
                PROCESS_VM_OPERATION |
                PROCESS_VM_WRITE |
                PROCESS_VM_READ,
            FALSE, pid
        );
        assert_ne!(process_handle, 0, "Failed to open process");

        let kernel32 = GetModuleHandleW(w!("kernel32.dll"));
        assert_ne!(kernel32, 0, "Failed to get handle of kernel32.dll");
        let load_library = GetProcAddress(kernel32, s!("LoadLibraryW"));
        assert!(load_library.is_some(), "Failed to get address of LoadLibraryW");
        let mut dll_path = current_exe().unwrap();
        dll_path.pop();
        dll_path.push("client_hook.dll");
        assert!(dll_path.exists());
        let dll_path =  U16CString::from_os_str(dll_path.into_os_string()).unwrap();
        println!("{} ({})", dll_path.display(), dll_path.len());
        let dll_path = dll_path.as_slice_with_nul();
        println!("{}", size_of_val(dll_path));

        let virtual_alloc_ptr = VirtualAllocEx(
            process_handle,
            null(),
            size_of_val(dll_path),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE);
        println!("{:?}", virtual_alloc_ptr);
        assert!(!virtual_alloc_ptr.is_null(), "Failed to allocate memory");


        assert_ne!(WriteProcessMemory(
            process_handle,
            virtual_alloc_ptr,
            dll_path.as_ptr() as _,
            size_of_val(dll_path),
            null_mut()
        ), FALSE, "Failed to write memory");

        assert_ne!(CreateRemoteThread(
            process_handle,
            null(),
            0,
            transmute(load_library),
            virtual_alloc_ptr,
            0,
            null_mut()
        ), 0, "Failed to create thread in target process");

    }
}
