use std::ffi::{c_char, c_void, CStr};
use std::fmt::Debug;
use std::mem::size_of;
use std::ptr::{addr_of, copy_nonoverlapping, null};
use once_cell::sync::OnceCell;
use windows_sys::Win32::Foundation::{HMODULE, HWND};
use windows_sys::Win32::System::Diagnostics::Debug::{IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_NT_HEADERS32, IMAGE_NT_HEADERS64, IMAGE_NT_OPTIONAL_HDR_MAGIC};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::{PAGE_EXECUTE_READWRITE, PAGE_READWRITE, VirtualProtect};
use windows_sys::Win32::System::SystemServices::{IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_DESCRIPTOR, IMAGE_NT_SIGNATURE, IMAGE_ORDINAL_FLAG32, IMAGE_ORDINAL_FLAG64};
use windows_sys::Win32::System::WindowsProgramming::IMAGE_THUNK_DATA64;
use windows_sys::Win32::UI::WindowsAndMessaging::TIMERPROC;

unsafe fn read_at<T>(base: HMODULE) -> T {
    let ptr = base as *const T;
    ptr.read()
}

#[cfg(target_pointer_width = "32")]
type IMAGE_NT_HEADERS = IMAGE_NT_HEADERS32;
#[cfg(target_pointer_width = "64")]
type IMAGE_NT_HEADERS = IMAGE_NT_HEADERS64;

#[cfg(target_pointer_width = "32")]
type IMAGE_THUNK_DATA = IMAGE_THUNK_DATA32;
#[cfg(target_pointer_width = "64")]
type IMAGE_THUNK_DATA = IMAGE_THUNK_DATA64;

#[cfg(target_pointer_width = "32")]
const IMAGE_ORDINAL_FLAG: u32 = IMAGE_ORDINAL_FLAG32;
#[cfg(target_pointer_width = "64")]
const IMAGE_ORDINAL_FLAG: u64 = IMAGE_ORDINAL_FLAG64;

type TimerProto = extern "system" fn(HWND, usize, u32, TIMERPROC) -> usize;
static HOOK: OnceCell<TimerProto> = OnceCell::new();

pub unsafe fn find_iat() {
    let base = GetModuleHandleW(null());
    let dos_header: IMAGE_DOS_HEADER = read_at(base);
    assert_eq!(dos_header.e_magic, IMAGE_DOS_SIGNATURE);
    let pe_header: IMAGE_NT_HEADERS = read_at(base + dos_header.e_lfanew as isize);
    assert_eq!(pe_header.Signature, IMAGE_NT_SIGNATURE);
    let optional_header = pe_header.OptionalHeader;
    assert_eq!(optional_header.Magic, IMAGE_NT_OPTIONAL_HDR_MAGIC);

    let import_dir = optional_header.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize];
    let descriptor_start_rva = import_dir.VirtualAddress;
    let mut import_descriptor_ptr= (base + descriptor_start_rva as isize) as *const IMAGE_IMPORT_DESCRIPTOR;

    loop {
        let import_descriptor = import_descriptor_ptr.read();
        if import_descriptor.Anonymous.Characteristics == 0 {
            break;
        }
        let name = CStr::from_ptr((base + import_descriptor.Name as isize) as *const c_char);
        println!("{:?}", name);

        let mut thunk_ilt = import_descriptor.Anonymous.OriginalFirstThunk as *const IMAGE_THUNK_DATA;
        let mut thunk_iat = import_descriptor.FirstThunk as *mut IMAGE_THUNK_DATA;
        assert!(!thunk_ilt.is_null());
        assert!(!thunk_iat.is_null());

        thunk_ilt = (base + thunk_ilt as isize) as *const IMAGE_THUNK_DATA;
        thunk_iat = (base + thunk_iat as isize) as *mut IMAGE_THUNK_DATA;
        assert!(!thunk_ilt.is_null());
        assert!(!thunk_iat.is_null());

        loop {
            let ilt = thunk_ilt.read();
            if ilt.u1.AddressOfData == 0 {
                break;
            }
            if ilt.u1.Ordinal & IMAGE_ORDINAL_FLAG == 0 {
                let name= (base + ilt.u1.AddressOfData as isize) as *const IMAGE_IMPORT_BY_NAME;
                let func_name = CStr::from_ptr((*name).Name.as_ptr() as _);
                println!("    {:?}", func_name);
                if func_name.to_bytes() == b"SetTimer" {
                    let old: TimerProto = std::mem::transmute((*thunk_iat).u1.Function);
                    HOOK.get_or_init(|| old);
                    let nf = SetTimer as u64;
                    write_protected::<u64>(addr_of!((*thunk_iat).u1.Function) as *const c_void, nf);
                    println!("{:x}", (*thunk_iat).u1.Function);
                }

            }
            thunk_ilt = thunk_ilt.offset(1);
            thunk_iat = thunk_iat.offset(1);
        }

        import_descriptor_ptr = import_descriptor_ptr.offset(1);
    }

}

unsafe fn write_protected<T>(src: *const c_void, data: T) {
    let mut protection = 0;
    VirtualProtect(
        src,
        size_of::<T>(),
        PAGE_READWRITE,
        &mut protection
    );
    let target = src as *mut T;

    target.write(data);

    VirtualProtect(
        src,
        size_of::<T>(),
        protection,
        &mut protection
    );
}

#[allow(non_snake_case)]
pub unsafe extern "system" fn SetTimer(hwnd: HWND, nidevent: usize, uelapse: u32, lptimerfunc: TIMERPROC) -> usize {
    println!("Test");
    HOOK.get_unchecked()(hwnd, nidevent, 1000, lptimerfunc)
}