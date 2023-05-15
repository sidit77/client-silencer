use std::ffi::{c_void, CStr};
use std::mem::size_of;
use std::ops::Add;
use std::ptr::{addr_of, null};
use once_cell::sync::OnceCell;
use windows_sys::Win32::Foundation::{HMODULE, HWND};
use windows_sys::Win32::System::Diagnostics::Debug::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Memory::*;
use windows_sys::Win32::System::SystemServices::*;
use windows_sys::Win32::System::WindowsProgramming::*;
use windows_sys::Win32::UI::WindowsAndMessaging::TIMERPROC;

#[cfg(target_pointer_width = "32")]
#[allow(non_camel_case_types)]
type IMAGE_NT_HEADERS = IMAGE_NT_HEADERS32;
#[cfg(target_pointer_width = "64")]
#[allow(non_camel_case_types)]
type IMAGE_NT_HEADERS = IMAGE_NT_HEADERS64;

#[cfg(target_pointer_width = "32")]
#[allow(non_camel_case_types)]
type IMAGE_THUNK_DATA = IMAGE_THUNK_DATA32;
#[cfg(target_pointer_width = "64")]
#[allow(non_camel_case_types)]
type IMAGE_THUNK_DATA = IMAGE_THUNK_DATA64;

#[cfg(target_pointer_width = "32")]
const IMAGE_ORDINAL_FLAG: u32 = IMAGE_ORDINAL_FLAG32;
#[cfg(target_pointer_width = "64")]
const IMAGE_ORDINAL_FLAG: u64 = IMAGE_ORDINAL_FLAG64;

type TimerProto = extern "system" fn(HWND, usize, u32, TIMERPROC) -> usize;
static HOOK: OnceCell<TimerProto> = OnceCell::new();

pub unsafe fn find_iat() {
    let base: IntPtr = GetModuleHandleW(null()).into();
    assert!(base.is_not_null());
    let dos_header: IMAGE_DOS_HEADER = base.read();
    assert_eq!(dos_header.e_magic, IMAGE_DOS_SIGNATURE);
    let pe_header: IMAGE_NT_HEADERS = (base + dos_header.e_lfanew.into()).read();
    assert_eq!(pe_header.Signature, IMAGE_NT_SIGNATURE);
    let optional_header = pe_header.OptionalHeader;
    assert_eq!(optional_header.Magic, IMAGE_NT_OPTIONAL_HDR_MAGIC);

    let import_dir = optional_header.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize];
    let descriptor_start_rva = import_dir.VirtualAddress.into();
    let mut import_descriptor_ptr: *const IMAGE_IMPORT_DESCRIPTOR = (base + descriptor_start_rva).as_ptr();

    loop {
        let import_descriptor = import_descriptor_ptr.read();
        if import_descriptor.Anonymous.Characteristics == 0 {
            break;
        }
        let name = CStr::from_ptr((base + import_descriptor.Name.into()).as_ptr());
        println!("{:?}", name);

        let thunk_ilt: IntPtr = import_descriptor.Anonymous.OriginalFirstThunk.into();
        let thunk_iat: IntPtr = import_descriptor.FirstThunk.into();
        assert!(thunk_ilt.is_not_null());
        assert!(thunk_iat.is_not_null());

        let mut thunk_ilt: *const IMAGE_THUNK_DATA = (base + thunk_ilt).as_ptr();
        let mut thunk_iat: *mut IMAGE_THUNK_DATA = (base + thunk_iat).as_mut_ptr();
        assert!(!thunk_ilt.is_null());
        assert!(!thunk_iat.is_null());

        loop {
            let ilt = thunk_ilt.read();
            if ilt.u1.AddressOfData == 0 {
                break;
            }
            if ilt.u1.Ordinal & IMAGE_ORDINAL_FLAG == 0 {
                let name: *const IMAGE_IMPORT_BY_NAME = (base + ilt.u1.AddressOfData.into()).as_ptr();
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
struct IntPtr(usize);

impl IntPtr {
    fn is_not_null(self) -> bool {
        self.0 != 0
    }
    fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }
    fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
    unsafe fn read<T>(self) -> T {
        self.as_ptr::<T>().read()
    }
}

impl From<usize> for IntPtr {
    fn from(value: usize) -> Self {
        IntPtr(value)
    }
}

impl From<isize> for IntPtr {
    fn from(value: isize) -> Self {
        IntPtr(value as usize)
    }
}

impl From<u32> for IntPtr {
    fn from(value: u32) -> Self {
        IntPtr(value as usize)
    }
}

impl From<i32> for IntPtr {
    fn from(value: i32) -> Self {
        IntPtr(value as usize)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<u64> for IntPtr {
    fn from(value: u64) -> Self {
        IntPtr(value as usize)
    }
}

impl Add for IntPtr {
    type Output = IntPtr;

    fn add(self, rhs: Self) -> Self::Output {
        IntPtr(self.0 + rhs.0)
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
pub unsafe extern "system" fn SetTimer(hwnd: HWND, nidevent: usize, _uelapse: u32, lptimerfunc: TIMERPROC) -> usize {
    println!("Test");
    HOOK.get_unchecked()(hwnd, nidevent, 1000, lptimerfunc)
}