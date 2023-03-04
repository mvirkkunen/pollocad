use std::ptr;
use std::ffi::c_void;

use cpp::{cpp, cpp_class};
use raw_window_handle as rwh;

use crate::*;

cpp! {{
    #include <memory>

    #include "preview.cpp"
}}

bitflags::bitflags! {
    pub struct MouseFlags: i32 {
        const BUTTON_LEFT = crate::constants::MouseFlags::Left;
        const BUTTON_MIDDLE = crate::constants::MouseFlags::Middle;
        const BUTTON_RIGHT = crate::constants::MouseFlags::Right;
        const BUTTON_CHANGE = crate::constants::MouseFlags::ButtonChange;
        const MODIFIER_CTRL = crate::constants::MouseFlags::Ctrl;
        const MODIFIER_SHIFT = crate::constants::MouseFlags::Shift;
        const MODIFIER_ALT = crate::constants::MouseFlags::Alt;
    }
}

cpp_class!(pub unsafe struct CascadePreview as "std::unique_ptr<CascadePreview>");

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

        let preview = cpp!(unsafe [] -> CascadePreview as "std::unique_ptr<CascadePreview>" {
            return std::make_unique<CascadePreview>();
        });

        cpp!(unsafe [preview as "std::unique_ptr<CascadePreview>", display as "void *", window as "void *"] -> VoidResult as "CppResult<void>" {
            return protect<void>([&] { preview->init(display, window); });
        }).result()?;

        Ok(preview)
    }

    pub fn paint(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<()> {
        cpp!(unsafe [self as "std::unique_ptr<CascadePreview> *", x as "uint32_t", y as "uint32_t", width as "uint32_t", height as "uint32_t"] -> VoidResult as "CppResult<void>" {
            return protect<void>([&]{ (*self)->paint(x, y, width, height); });
        }).result()
    }

    pub fn mouse_event(&mut self, x: i32, y: i32, wheel: i32, flags: MouseFlags) -> Result<()> {
        let flags = flags.bits() as u32;

        cpp!(unsafe [self as "std::unique_ptr<CascadePreview> *", x as "int32_t", y as "int32_t", wheel as "int32_t", flags as "MouseFlags"] -> VoidResult as "CppResult<void>" {
            return protect<void>([&]{ (*self)->mouse_event(x, y, wheel, flags); });
        }).result()
    }

    pub fn set_shape(&mut self, shape: &crate::Shape) -> Result<()> {
        cpp!(unsafe [self as "std::unique_ptr<CascadePreview> *", shape as "TopoDS_Shape *"] -> VoidResult as "CppResult<void>" {
            return protect<void>([&]{ (*self)->set_shape(*shape); });
        }).result()
    }

    pub fn has_animation(&self) -> Result<bool> {
        cpp!(unsafe [self as "std::unique_ptr<CascadePreview> *"] -> BoolResult as "CppResult<bool>" {
            return protect<bool>([&]{ return (*self)->has_animation(); });
        }).result()
    }
}
