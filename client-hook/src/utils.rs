use std::ops::{Add, Sub};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
struct PtrOffset(isize);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct IntPtr(usize);

impl IntPtr {
    pub fn is_not_null(self) -> bool {
        self.0 != 0
    }
    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
    pub unsafe fn read<T>(self) -> T {
        self.as_ptr::<T>().read()
    }
    pub fn offset(self, next: Self) -> PtrOffset {
        PtrOffset(next.0 as isize - self.0 as isize)
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

impl Sub for IntPtr {
    type Output = IntPtr;

    fn sub(self, rhs: Self) -> Self::Output {
        IntPtr(self.0 - rhs.0)
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
        self
            .inner
            .next()
            .map(|ptr| unsafe { ptr.read() })
            .filter(|next| (self.valid)(next))
    }
}

pub enum Error {
    BadPeFormat
}