use gtk::EventControllerKey;
use gtk::GestureClick;
use gtk::glib;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::graphene::Point;
use std::cell::RefCell;
use gtk::subclass::prelude::*;
use gdk4_x11::{X11Surface, X11Display};
use pollocad_occt::{CascadePreview, MouseFlags};

struct Handles(*mut std::ffi::c_void, *mut std::ffi::c_void);   

unsafe impl raw_window_handle::HasRawWindowHandle for Handles {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        let mut handle = raw_window_handle::XlibWindowHandle::empty();
        handle.window = self.1 as u64;
        raw_window_handle::RawWindowHandle::Xlib(handle)
    }
}

unsafe impl raw_window_handle::HasRawDisplayHandle for Handles {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        let mut handle = raw_window_handle::XlibDisplayHandle::empty();
        handle.display = self.0;
        raw_window_handle::RawDisplayHandle::Xlib(handle)
    }
}

#[derive(Default)]
pub struct OcctGLArea {
    preview: RefCell<Option<CascadePreview>>,
    tick_callback: RefCell<Option<gtk::TickCallbackId>>,
}

impl OcctGLArea {
    pub fn set_code(&self, code: &str) {
        println!("code: {}", code);

        if self.preview.borrow().is_some() {
            match crate::parser::parse_source(code) {
                Ok((_, body)) => match crate::runtime::exec(body.as_ref()) {
                    Ok(crate::runtime::Value::Solid(geo)) => {
                        self.preview.borrow_mut().as_mut().unwrap().set_shape(&*geo.get_single_shape().expect("no shape")).expect("set_shape failed");
                    }
                    Err(e) => {
                        eprintln!("Exec error: {:#?}", e);
                    }
                    _ => {}
                },
                Err(e) => {
                    eprintln!("Parse error: {:#?}", e);
                }
            }
            
            self.obj().queue_render();
        }
    }

    fn handle_event(&self, ev: Option<gtk::gdk::Event>, buttons_changed: bool) {
        let Some(ev) = ev else { return };

        let mut preview = self.preview.borrow_mut();
        let preview = preview.as_mut().unwrap();

        /*if let Some(ev) = ev.downcast::<gtk::gdk::ButtonEvent>() {
            pressed = ev.

            match ev.button() {
                0 => 
            }
        }*/

        /*let wheel = if input.scroll_delta.y != 0.0 {
            (input.scroll_delta.y * ppp) as i32
        } else {
            0
        };*/

        let wheel = 0;

        let (x, y) = ev.position().unwrap_or((0.0, 0.0));

        let mut flags = MouseFlags::empty();
        flags.set(MouseFlags::BUTTON_CHANGE, buttons_changed);
        if (buttons_changed) {
            flags.set(MouseFlags::BUTTON_LEFT, ev.modifier_state().contains(gtk::gdk::ModifierType::BUTTON1_MASK));
        }
        //flags.set(MouseFlags::BUTTON_MIDDLE, input.pointer.middle_down());
        //flags.set(MouseFlags::BUTTON_RIGHT, input.pointer.secondary_down());
        //flags.set(MouseFlags::MODIFIER_CTRL, input.modifiers.ctrl);
        //flags.set(MouseFlags::MODIFIER_SHIFT, input.modifiers.shift);
        //flags.set(MouseFlags::MODIFIER_ALT, input.modifiers.alt);

        println!("{:?}", flags);

        //self.obj().translate_coordinates(self.obj(), src_x, src_y)
        let coords = self.obj().native().unwrap().compute_point(self.obj().as_ref(), &Point::new(x as f32, y as f32)).unwrap();
        //self.obj().translate_coordinates(dest_widget, src_x, src_y)

        preview.mouse_event(coords.x() as i32, coords.y() as i32, wheel, flags).unwrap();

        self.obj().queue_render();
    }
}

#[glib::object_subclass]
impl ObjectSubclass for OcctGLArea {
    const NAME: &'static str = "OcctGLArea";
    type Type = super::OcctGLArea;
    type ParentType = gtk::GLArea;
}

impl ObjectImpl for OcctGLArea {
    fn constructed(&self) {
        self.parent_constructed();

        let widget = self.obj();

        let click = GestureClick::new();

        click.connect_pressed(clone!(@weak self as obj => move |ctrl, _, _, _| {
            obj.handle_event(ctrl.current_event(), true);
        }));
        click.connect_released(clone!(@weak self as obj => move |ctrl, _, _, _| {
            obj.handle_event(ctrl.current_event(), true);
        }));

        widget.add_controller(click);

        let key = EventControllerKey::new();

        key.connect_key_pressed(clone!(@weak self as obj => @default-panic, move |ctrl, keyval, keycode, state| {
            obj.handle_event(ctrl.current_event(), false);
            glib::signal::Inhibit(false)
        }));

        widget.add_controller(key);

        let motion = gtk::EventControllerMotion::new();

        motion.connect_motion(clone!(@weak self as obj => move |ctrl, x, y| {
            println!("{} {}", x, y);
            obj.handle_event(ctrl.current_event(), false);
            
        }));

        widget.add_controller(motion);
    }
}

impl WidgetImpl for OcctGLArea {
    fn realize(&self) {
        let widget = self.obj();

        widget.set_has_depth_buffer(true);
        widget.set_has_stencil_buffer(true);
        //widget.set_use_es(true);

        self.parent_realize();

        if widget.error().is_some() {
            return;
        }

        widget.make_current();
        if widget.error().is_some() {
            return;
        }
        println!("API: {:?}", widget.context().unwrap().api());

        //println!("native: {}", xid);

        //println!("surface: {}", GLContextExt::surface(&widget.context().unwrap()).unwrap().downcast::<X11Surface>().unwrap().xid());
    }

    fn unrealize(&self) {
        *self.preview.borrow_mut() = None;

        self.parent_unrealize();
    }
}

impl GLAreaImpl for OcctGLArea {
    fn render(&self, _context: &gtk::gdk::GLContext) -> bool {
        let widget = self.obj();

        self.obj().make_current();

        println!("paint {:?} {:?}", self.obj().width(), self.obj().height());

        //let err = glib::Error::new(glib::MarkupError::Empty, "RIP");
        //self.obj().set_error(Some(&err));

        let surface = widget.native().unwrap().surface();
        //let surface = GLContextExt::surface(&widget.context().unwrap()).unwrap();
        let display = surface.display();
        let xid = surface.downcast::<X11Surface>().unwrap().xid();
        let xdisplay = unsafe { display.downcast::<X11Display>().unwrap().xdisplay() };

        if self.preview.borrow().is_none() {
            *self.preview.borrow_mut() = Some(
                CascadePreview::new(&Handles(xdisplay as *mut std::ffi::c_void, xid as *mut std::ffi::c_void)).expect("creating preview failed"));
        }

        let f = widget.scale_factor();

        //return true;

        /*self.preview.borrow().as_mut().unwrap()
            .paint(
                (info.viewport.left() * ppp) as u32, (info.viewport.top() * ppp) as u32,
                (info.viewport.width() * ppp) as u32, (info.viewport.height() * ppp) as u32).expect("paint failed");*/
        self.preview.borrow_mut().as_mut().unwrap().paint(0, 0, (widget.width() * f) as u32, (widget.height() * f) as u32).expect("paint failed");

        let has_animation = self.preview.borrow().as_ref().unwrap().has_animation().unwrap_or(false);

        match ((self.preview.borrow().as_ref().unwrap().has_animation().unwrap_or(false), &mut *self.tick_callback.borrow_mut())) {
            (true, callback@None) => {
                println!("begin");
                *callback = Some(widget.add_tick_callback(|widget, _| {
                    println!("tick");
                    widget.queue_render();
                    widget.queue_draw();
                    Continue(true)
                }));
            },
            (false, callback@Some(_)) => {
                println!("end");
                callback.take().unwrap().remove();
            },
            _ => {},
        }

        /*if (has_animation && !self.animating.get()) {
            widget.add_tick_callback(|widget, _| {
                widget.queue_render();
                Continue(false)
            });
        } else if (!has_animation && self.animating.get()) {

        }

        */

        true
    }
}
