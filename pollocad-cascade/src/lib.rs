use std::ffi::{CStr, c_void};
use std::ptr;
use raw_window_handle as rwh;
use raw_window_handle::{HasRawWindowHandle as _, HasRawDisplayHandle as _};

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
mod bind {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[derive(Clone, Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CGAL error: {}", self.0)
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

fn protect<R>(f: impl FnOnce(*mut bind::Error) -> R) -> Result<R> {
    let mut err: bind::Error = ptr::null_mut();
    let r = f(&mut err as *mut bind::Error);
    if !err.is_null() {
        unsafe {
            let err_str = String::from_utf8_lossy(CStr::from_ptr(err).to_bytes()).to_string();
            bind::error_free2(err);
            return Err(Error(err_str));
        }
    }

    return Ok(r);
}

/*#[repr(u32)]
pub enum BooleanOp {
    Union = bind::BooleanOp_BOOLEAN_OP_UNION as u32,
    Difference = bind::BooleanOp_BOOLEAN_OP_DIFFERENCE as u32,
    Intersection = bind::BooleanOp_BOOLEAN_OP_INTERSECTION as u32,
}*/

pub struct CascadePreview(bind::CascadePreview);

pub mod MouseFlags {
    use super::bind;
    pub const BUTTON_LEFT: u32 = bind::MouseFlags_MOUSE_BUTTON_LEFT;
    pub const BUTTON_MIDDLE: u32 = bind::MouseFlags_MOUSE_BUTTON_MIDDLE;
    pub const BUTTON_RIGHT: u32 = bind::MouseFlags_MOUSE_BUTTON_RIGHT;
    pub const BUTTON_CHANGE: u32 = bind::MouseFlags_MOUSE_FLAGS_BUTTON_CHANGE;
}

unsafe impl Send for CascadePreview {}

impl CascadePreview {
    pub fn new(window: &(impl rwh::HasRawWindowHandle + rwh::HasRawDisplayHandle)) -> Result<CascadePreview> {
        let (window, display) = match (window.raw_window_handle(), window.raw_display_handle()) {
            (rwh::RawWindowHandle::Xlib(rwh::XlibWindowHandle { window, .. }), rwh::RawDisplayHandle::Xlib(rwh::XlibDisplayHandle { display, .. })) => (window as *mut c_void, display),
            _ => (ptr::null_mut(), ptr::null_mut()),
        };

        unsafe { protect(|err| bind::cascade_preview_new(display, window, err) ).map(CascadePreview) }
    }

    pub fn paint(&mut self, x: u32, y: u32, width: u32, height: u32, angle: f32) -> Result<()> {
        unsafe { protect(|err| bind::cascade_preview_paint(self.0, x, y, width, height, angle, err)) }
    }

    pub fn mouse_event(&mut self, x: i32, y: i32, wheel: i32, flags: u32) -> Result<()> {
        unsafe { protect(|err| bind::cascade_preview_mouse_event(self.0, x, y, wheel, flags, err)) }
    }
}

impl Drop for CascadePreview {
    fn drop(&mut self) {
        unsafe { bind::cascade_preview_free(self.0); }
    }
}
