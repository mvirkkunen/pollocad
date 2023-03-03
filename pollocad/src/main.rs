#![allow(dead_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Very haphazardly copy-pasted together from egui/wgpu examples

use std::sync::{Arc, Mutex};

use eframe::egui;

mod ast;
mod builtins;
mod geometry;
mod parser;
//mod preview;
mod runtime;

use pollocad_cascade::{CascadePreview, MouseFlags};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 800.0)),
        multisampling: 1,
        renderer: eframe::Renderer::Glow,
        depth_buffer: 24,
        stencil_buffer: 8,
        ..Default::default()
    };
    eframe::run_native(
        "pollocad",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc).unwrap())),
    )
}

pub struct MyApp {
    code: String,
    //preview: preview::Renderer,
    preview: Arc<Mutex<pollocad_cascade::CascadePreview>>,
    num_indices: u32,
    num_vertices: u32,
    valid: bool,
}

const CODE: &'static str = r#"
x = 2;
translate(z=-5, y=-5) union() {
    cube(x, 30, 20);
    translate(z=5, y=5) anti() cube(10, 10, 10);
}

translate(x=-10, y=2, z=2) cube(20, 6, 6);
"#;

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Option<Self> {
        //let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        cc.egui_ctx.set_pixels_per_point(2.0);

        Some(Self {
            code: CODE.to_string(),
            //preview: preview::Renderer::new(wgpu_render_state),
            preview: Arc::new(
                Mutex::new(
                    CascadePreview::new(&cc.integration_info.window_info).expect("create preview failed"))),
            num_indices: 0,
            num_vertices: 0,
            valid: false,
        })
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("code_panel")
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Min).with_main_justify(true),
                    |ui| {
                        let response =
                            ui.add(egui::TextEdit::multiline(&mut self.code).frame(false));

                        if response.changed() || !self.valid {
                            self.valid = true;
                            match parser::parse_source(&self.code) {
                                Ok((_, body)) => match runtime::exec(body.as_ref()) {
                                    Ok(runtime::Value::Solid(geo)) => {
                                        self.preview.lock().unwrap().set_shape(&*geo.get_single_shape().expect("no shape")).expect("set_shape failed");
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
                        }
                    },
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style())
                .fill(egui::Color32::WHITE)
                .show(ui, |ui| {

                    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
                    let ppp = ctx.pixels_per_point();

                    {
                        let mut preview = self.preview.lock().unwrap();

                        ctx.input(|input| {
                            let (x, y) = response.hover_pos()
                                .map(|p| p - rect.left_top())
                                .map(|p| ((p.x * ppp) as i32, (p.y * ppp) as i32))
                                .unwrap_or((0, 0));

                            let wheel = if input.scroll_delta.y != 0.0 {
                                (input.scroll_delta.y * ppp) as i32
                            } else {
                                0
                            };

                            let mut flags = MouseFlags::empty();
                            flags.set(MouseFlags::BUTTON_CHANGE, input.pointer.any_pressed() || input.pointer.any_released());
                            flags.set(MouseFlags::BUTTON_LEFT, input.pointer.primary_down());
                            flags.set(MouseFlags::BUTTON_MIDDLE, input.pointer.middle_down());
                            flags.set(MouseFlags::BUTTON_RIGHT, input.pointer.secondary_down());
                            flags.set(MouseFlags::MODIFIER_CTRL, input.modifiers.ctrl);
                            flags.set(MouseFlags::MODIFIER_SHIFT, input.modifiers.shift);
                            flags.set(MouseFlags::MODIFIER_ALT, input.modifiers.alt);

                            if input.pointer.is_moving() || wheel != 0 || flags.contains(MouseFlags::BUTTON_CHANGE) {
                                preview.mouse_event(x, y, wheel, flags).unwrap();
                            }
                        });
                    }

                    let preview = self.preview.clone();

                    let cb = eframe::egui_glow::CallbackFn::new(move |info, _painter| {
                        preview.lock().unwrap()
                            .paint(
                                (info.viewport.left() * ppp) as u32, (info.viewport.top() * ppp) as u32,
                                (info.viewport.width() * ppp) as u32, (info.viewport.height() * ppp) as u32).expect("paint failed");
                    });

                    ui.painter().add(egui::PaintCallback {
                        rect,
                        callback: Arc::new(cb),
                    });
                });
        });
    }
}
