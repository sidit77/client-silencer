use std::env::{current_exe, set_current_dir};
use std::iter::once;
use std::mem::{size_of, transmute, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE, INVALID_HANDLE_VALUE, TRUE};
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::Threading::*;
use windows_sys::{s, w};

fn main() {
    let process_name = "LeagueClientUx.exe";

    current_exe()
        .map(|mut path| {
            path.pop();
            path
        })
        .and_then(set_current_dir)
        .unwrap_or_else(|err| println!("Failed to move working dir: {}", err));

    let dll_path = PathBuf::from("client_hook.dll")
        .canonicalize()
        .expect("Can not find dll file");

    println!("Attempting to inject dll file: {}", dll_path.display());
    print!("Searching for process \"{}\"...", process_name);

    let pid = ProcessIter::default()
        .find(|(_, name)| name == process_name)
        .map(|(pid, _)| pid)
        .expect("Could not find process");

    println!("  Success (pid: {})", pid);

    print!("Opening process...");
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ,
            FALSE,
            pid
        )
    };
    assert_ne!(process_handle, 0, "Failed to open process");
    println!("  Success (handle: 0x{:x})", process_handle);

    print!("Writing dll path to memory...");
    let memory_ptr = unsafe {
        let path: Vec<u16> = dll_path
            .into_os_string()
            .encode_wide()
            .chain(once(0u16))
            .collect();

        let virtual_alloc_ptr = VirtualAllocEx(
            process_handle,
            null(),
            path.len() * size_of::<u16>(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE
        );
        assert!(!virtual_alloc_ptr.is_null(), "Failed to allocate memory");

        assert_ne!(
            WriteProcessMemory(
                process_handle,
                virtual_alloc_ptr,
                path.as_ptr() as _,
                path.len() * size_of::<u16>(),
                null_mut()
            ),
            FALSE,
            "Failed to write memory"
        );

        virtual_alloc_ptr
    };
    println!("  Success (ptr: {:?})", memory_ptr);

    print!("Creating remote thread...");
    let remote_thread = unsafe {
        let kernel32 = GetModuleHandleW(w!("kernel32.dll"));
        assert_ne!(kernel32, 0, "Failed to get handle of kernel32.dll");
        let load_library = GetProcAddress(kernel32, s!("LoadLibraryW"));
        assert!(load_library.is_some(), "Failed to get address of LoadLibraryW");

        let mut thread_id = 0;
        assert_ne!(
            CreateRemoteThread(
                process_handle,
                null(),
                0,
                transmute(load_library),
                memory_ptr,
                0,
                &mut thread_id
            ),
            0,
            "Failed to create thread in target process"
        );
        thread_id
    };
    println!("  Success (tid: 0x{:x})", remote_thread);
    println!("Done");
}

pub struct ProcessIter {
    entry: PROCESSENTRY32W,
    snapshot: HANDLE
}

impl Default for ProcessIter {
    fn default() -> Self {
        let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        assert_ne!(snapshot, INVALID_HANDLE_VALUE, "Failed to take snapshot of current process list");

        Self { entry, snapshot }
    }
}

impl Drop for ProcessIter {
    fn drop(&mut self) {
        assert_ne!(unsafe { CloseHandle(self.snapshot) }, 0, "Failed to close process snapshot");
    }
}

fn trim(data: &[u16]) -> String {
    let len =  data
        .iter()
        .take_while(|c| **c != 0)
        .count();
    String::from_utf16_lossy(&data[..len])
}

impl Iterator for ProcessIter {
    type Item = (u32, String);

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { Process32NextW(self.snapshot, &mut self.entry) } {
            TRUE => {
                let pid = self.entry.th32ProcessID;
                let name = trim(&self.entry.szExeFile);
                Some((pid, name))
            }
            _ => None
        }
    }
}
