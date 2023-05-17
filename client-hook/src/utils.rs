use core::ffi::c_void;
use core::mem::size_of;
use core::ops::{Add, Sub};

use windows_sys::w;
use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Memory::{VirtualProtect, PAGE_READWRITE};

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $result:expr) => {
        if !($cond) {
            return Err($result);
        }
    };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IntPtr(usize);

impl IntPtr {
    pub const fn null() -> Self {
        Self(0)
    }
    pub fn is_not_null(self) -> bool {
        self.0 != 0
    }
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }
    pub unsafe fn read<T>(self) -> T {
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

impl<T> From<*const T> for IntPtr {
    fn from(value: *const T) -> Self {
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

#[derive(Copy, Clone)]
pub struct RawIterPtr<T> {
    ptr: *const T
}

impl<T> RawIterPtr<T> {
    pub unsafe fn new(ptr: *const T) -> Self {
        Self { ptr }
    }
}

impl<T> Iterator for RawIterPtr<T> {
    type Item = *const T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.ptr;
        self.ptr = unsafe { self.ptr.offset(1) };
        Some(next)
    }
}

#[derive(Copy, Clone)]
pub struct IterPtr<T> {
    inner: RawIterPtr<T>,
    valid: fn(&T) -> bool
}

impl<T> IterPtr<T> {
    pub unsafe fn until(ptr: *const T, valid: fn(&T) -> bool) -> Self {
        Self {
            inner: RawIterPtr::new(ptr),
            valid
        }
    }
}

impl<T> Iterator for IterPtr<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|ptr| unsafe { ptr.read() })
            .filter(|next| (self.valid)(next))
    }
}

pub unsafe fn write_protected<T>(src: *const c_void, data: T) -> Result<(), Error> {
    let mut protection = 0;
    ensure!(VirtualProtect(
        src,
        size_of::<T>(),
        PAGE_READWRITE,
        &mut protection
    ) != FALSE, Error::WindowsFailure);
    let target = src as *mut T;

    target.write(data);

    ensure!(VirtualProtect(
        src,
        size_of::<T>(),
        protection,
        &mut protection
    ) != FALSE, Error::WindowsFailure);
    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    BadPeFormat,
    ModuleNotFound,
    FunctionNotFound,
    WindowsFailure
}

impl Error {
    pub fn msg(self) -> *const u16 {
        match self {
            Error::BadPeFormat => w!("Failed to read executable"),
            Error::ModuleNotFound => w!("Could not find the specified module"),
            Error::FunctionNotFound => w!("Could not find the specified function"),
            Error::WindowsFailure => w!("A Windows function returned unsuccessfully")
        }
    }
}
