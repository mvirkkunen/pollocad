use std::ffi::CStr;

use cpp::{cpp, cpp_class};

mod shape;
pub use shape::*;

mod preview;
pub use preview::*;

mod constants;

#[derive(Clone, Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OCCT error: {}", self.0)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

trait CppResult: Sized {
    type Value;

    fn result(self) -> Result<Self::Value> {
        unsafe {
            let mut err: *mut i8 = std::ptr::null_mut();
            let value = self.get(&mut err);

            if err.is_null() {
                Ok(value)
            } else {
                let err_str = String::from_utf8_lossy(CStr::from_ptr(err).to_bytes()).to_string();
                Err(crate::Error(err_str))
            }
        }
    }

    unsafe fn get(&self, err: *mut *mut i8) -> Self::Value;
}

cpp_class!(unsafe struct VoidResult as "CppResult<void>");

impl CppResult for VoidResult {
    type Value = ();

    unsafe fn get(&self, err: *mut *mut i8) -> Self::Value {
        cpp!([self as "CppResult<void> *", err as "char **"] -> () as "void" {
            self->get(err);
        });
    }
}

cpp_class!(unsafe struct BoolResult as "CppResult<bool>");

impl CppResult for BoolResult {
    type Value = bool;

    unsafe fn get(&self, err: *mut *mut i8) -> Self::Value {
        cpp!([self as "CppResult<bool> *", err as "char **"] -> bool as "bool" {
            return self->get(err);
        })
    }
}
