use std::ffi::{CStr, c_void};
use std::ptr;
use raw_window_handle as rwh;

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
            bind::cascade_error_free(err);
            return Err(Error(err_str));
        }
    }

    return Ok(r);
}

#[repr(u32)]
pub enum BooleanOp {
    Union = bind::BooleanOp_BOOLEAN_OP_UNION as u32,
    Difference = bind::BooleanOp_BOOLEAN_OP_DIFFERENCE as u32,
    Intersection = bind::BooleanOp_BOOLEAN_OP_INTERSECTION as u32,
}

pub struct CascadePreview(bind::CascadePreview);

bitflags::bitflags! {
    pub struct MouseFlags: u32 {
        const BUTTON_LEFT = bind::MouseFlags_MOUSE_FLAG_BUTTON_LEFT;
        const BUTTON_MIDDLE = bind::MouseFlags_MOUSE_FLAG_BUTTON_MIDDLE;
        const BUTTON_RIGHT = bind::MouseFlags_MOUSE_FLAG_BUTTON_RIGHT;
        const BUTTON_CHANGE = bind::MouseFlags_MOUSE_FLAG_BUTTON_CHANGE;
        const MODIFIER_CTRL =bind::MouseFlags_MOUSE_FLAG_MODIFIER_CTRL;
        const MODIFIER_SHIFT = bind::MouseFlags_MOUSE_FLAG_MODIFIER_SHIFT;
        const MODIFIER_ALT = bind::MouseFlags_MOUSE_FLAG_MODIFIER_ALT;
    }
}

unsafe impl Send for CascadePreview {}

impl CascadePreview {
    pub fn new(window: &(impl rwh::HasRawDisplayHandle + rwh::HasRawWindowHandle)) -> Result<CascadePreview> {
        let display = match window.raw_display_handle() {
            rwh::RawDisplayHandle::Xlib(rwh::XlibDisplayHandle { display, .. }) => display,
            _ => ptr::null_mut()
        };

        let window = match window.raw_window_handle() {
            rwh::RawWindowHandle::Xlib(rwh::XlibWindowHandle { window, .. }) => window as *mut c_void,
            rwh::RawWindowHandle::Win32(rwh::Win32WindowHandle { hwnd, .. }) => hwnd as *mut c_void,
            _ => ptr::null_mut(),
        };

        unsafe { protect(|err| bind::cascade_preview_new(display, window, err) ).map(CascadePreview) }
    }

    pub fn paint(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<()> {
        unsafe { protect(|err| bind::cascade_preview_paint(self.0, x, y, width, height, err)) }
    }

    pub fn mouse_event(&mut self, x: i32, y: i32, wheel: i32, flags: MouseFlags) -> Result<()> {
        unsafe { protect(|err| bind::cascade_preview_mouse_event(self.0, x, y, wheel, flags.bits(), err)) }
    }

    pub fn set_shape(&mut self, shape: &Shape) -> Result<()> {
        unsafe { protect(|err| bind::cascade_preview_set_shape(self.0, shape.0, err)) }
    }

    pub fn has_animation(&self) -> Result<bool> {
        unsafe { protect(|err| bind::cascade_preview_has_animation(self.0, err)).map(|v| v != 0) }
    }
}

impl Drop for CascadePreview {
    fn drop(&mut self) {
        unsafe { bind::cascade_preview_free(self.0); }
    }
}

pub struct Shape(bind::CascadeShape);

unsafe impl Send for Shape {}

impl Shape {
    pub fn new_cube(x: f64, y: f64, z: f64) -> Result<Shape> {
        unsafe { protect(|err| bind::cascade_shape_new_box(x, y, z, err)).map(Shape) }
    }

    pub fn new_cylinder(r: f64, h: f64) -> Result<Shape> {
        unsafe { protect(|err| bind::cascade_shape_new_cylinder(r, h, err)).map(Shape) }
    }

    pub fn transform(&self, matrix: &[f64; 16]) -> Result<Shape> {
        unsafe { protect(|err| bind::cascade_shape_transform(self.0, matrix.as_ptr(), err)).map(Shape) }
    }

    pub fn boolean_op(&self, other: &Shape, op: BooleanOp) -> Result<Shape> {
        unsafe {
            protect(|err| {
                bind::cascade_shape_boolean_op(self.0, other.0, op as u32, err)
            }).map(Shape)
        }
    }
}

impl Drop for Shape {
    fn drop(&mut self) {
        unsafe {
            bind::cascade_shape_free(self.0);
        }
    }
}

impl Clone for Shape {
    fn clone(&self) -> Self {
        unsafe { Shape(protect(|err| bind::cascade_shape_clone(self.0, err)).expect("cascade_shape_clone failed")) }
    }
}