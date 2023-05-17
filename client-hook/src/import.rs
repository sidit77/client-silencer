use core::ffi::CStr;
use core::ptr::{addr_of, null};

use windows_sys::Win32::System::Diagnostics::Debug::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::*;
use windows_sys::Win32::System::WindowsProgramming::*;

use crate::ensure;
use crate::utils::{Error, IntPtr, IterPtr, RawIterPtr};

//use std::io::Write;
//use std::fs::File;

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

pub unsafe fn find_function_iat(module: &[u8], name: &[u8]) -> Result<IntPtr, Error> {
    //let mut file = File::create("F:\\log.txt").unwrap();

    let base: IntPtr = GetModuleHandleW(null()).into();
    ensure!(base.is_not_null(), Error::WindowsFailure);
    let dos_header: IMAGE_DOS_HEADER = base.read();
    ensure!(dos_header.e_magic == IMAGE_DOS_SIGNATURE, Error::BadPeFormat);
    let pe_header: IMAGE_NT_HEADERS = (base + dos_header.e_lfanew.into()).read();
    ensure!(pe_header.Signature == IMAGE_NT_SIGNATURE, Error::BadPeFormat);
    let optional_header = pe_header.OptionalHeader;
    ensure!(optional_header.Magic == IMAGE_NT_OPTIONAL_HDR_MAGIC, Error::BadPeFormat);

    let import_dir = optional_header.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT as usize];

    for import_descriptor in IterPtr::<IMAGE_IMPORT_DESCRIPTOR>::until(
        (base + import_dir.VirtualAddress.into()).as_ptr(),
        |desc| desc.Anonymous.Characteristics != 0
    ) {
        let module_name = CStr::from_ptr((base + import_descriptor.Name.into()).as_ptr());
        //writeln!(file, "{:?}", module_name);
        if module_name.to_bytes() == module {
            let thunk_ilt: IntPtr = import_descriptor.Anonymous.OriginalFirstThunk.into();
            let thunk_iat: IntPtr = import_descriptor.FirstThunk.into();
            ensure!(thunk_ilt.is_not_null(), Error::BadPeFormat);
            ensure!(thunk_iat.is_not_null(), Error::BadPeFormat);

            let ilt_iter = IterPtr::<IMAGE_THUNK_DATA>::until((base + thunk_ilt).as_ptr(), |ilt| ilt.u1.AddressOfData != 0);
            let iat_iter = RawIterPtr::<IMAGE_THUNK_DATA>::new((base + thunk_iat).as_ptr());
            for (ilt, iat) in ilt_iter.zip(iat_iter) {
                if ilt.u1.Ordinal & IMAGE_ORDINAL_FLAG == 0 {
                    let import: *const IMAGE_IMPORT_BY_NAME = (base + ilt.u1.AddressOfData.into()).as_ptr();
                    let func_name = CStr::from_ptr((*import).Name.as_ptr() as _);
                    //writeln!(file, "    {:?}", func_name);
                    if func_name.to_bytes() == name {
                        return Ok(addr_of!((*iat).u1.Function).into());
                    }
                }
            }
            return Err(Error::FunctionNotFound);
        }
    }
    Err(Error::ModuleNotFound)
}
