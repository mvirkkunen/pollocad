use gtk::glib;
use gtk::prelude::*;
use gtk::glib::clone;
use gtk::subclass::prelude::ObjectSubclassIsExt;

mod ast;
mod builtins;
mod geometry;
mod parser;
//mod preview;
mod runtime;

mod imp;

glib::wrapper! {
    pub struct OcctGLArea(ObjectSubclass<imp::OcctGLArea>)
        @extends gtk::GLArea, gtk::Widget;
}

impl OcctGLArea {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new();
        //obj.set_has_depth_buffer(true);
        //obj.set_has_stencil_buffer(true);
        obj
    }

    pub fn set_code(&self, code: &str) {
        self.imp().set_code(code);
    }
}

fn main() -> glib::ExitCode {
    let application = gtk::Application::builder()
        .application_id("com.github.gtk-rs.examples.basic")
        .build();
    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some("pollocad"));
    window.set_default_size(500, 500);

    //let display = window.display();

    //let button = gtk::Button::with_label("Click me!");

    let text = gtk::TextView::new();
    text.buffer().set_text(CODE);
    text.set_width_request(300);

    //window.set_child(Some(&button));

    let preview = OcctGLArea::new();
    preview.set_hexpand(true);

    text.buffer().connect_changed(clone!(@weak text, @weak preview => move |_| {
        let buf = text.buffer();
        let text = buf.text(&buf.start_iter(), &buf.end_iter(), true);
        preview.set_code(&text);
    }));
    
    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    layout.append(&text);
    layout.append(&preview);

    window.set_child(Some(&layout));
    
    window.present();

    preview.set_code(CODE);
}

const CODE: &'static str = r#"
x = 2;
translate(z=-5, y=-5) union() {
    cube(x, 20, 30);
    translate(z=5, y=5) anti() cube(10, 10, 10);
}

translate(x=-10, y=2, z=2) cube(20, 6, 6);
"#;