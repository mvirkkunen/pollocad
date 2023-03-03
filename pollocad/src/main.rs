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
    angle: f32,
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
                    pollocad_cascade::CascadePreview::new(&cc.integration_info.window_info).expect("create preview failed"))),
            num_indices: 0,
            num_vertices: 0,
            angle: 0.2,
            valid: false,
        })
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut mesh_data = None;

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
                                        if let Ok(m) = geo.to_mesh_data() {
                                            mesh_data = m;

                                            if let Some(m) = &mesh_data {
                                                self.num_vertices =
                                                    (m.vertex_data().len() / (6 * 4)) as u32;
                                                self.num_indices =
                                                    (m.index_data().len() / 2) as u32;

                                                println!(
                                                    "num_vert {} size {}",
                                                    self.num_vertices,
                                                    m.vertex_data().len()
                                                );
                                            }
                                        }
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
                    //let response = self.preview.paint(ui, mesh_data, self.angle);
                    let ppp = ctx.pixels_per_point();

                    {
                        let mut preview = self.preview.lock().unwrap();

                        ctx.input(|input| {
                            let (x, y) = response.hover_pos()
                                .map(|p| p - rect.left_top())
                                .map(|p| ((p.x * ppp) as i32, (p.y * ppp) as i32))
                                .unwrap_or((0, 0));

                            let mut flags: u32 = 0;

                            if input.pointer.any_pressed() || input.pointer.any_released() {
                                flags |= pollocad_cascade::MouseFlags::BUTTON_CHANGE;
                            }

                            if input.pointer.primary_down() {
                                flags |= pollocad_cascade::MouseFlags::BUTTON_LEFT;
                            }

                            if input.pointer.middle_down() {
                                flags |= pollocad_cascade::MouseFlags::BUTTON_MIDDLE;
                            }

                            if input.pointer.secondary_down() {
                                flags |= pollocad_cascade::MouseFlags::BUTTON_RIGHT;
                            }

                            let wheel = if input.scroll_delta.y != 0.0 {
                                (input.scroll_delta.y * ppp) as i32
                            } else {
                                0
                            };

                            if input.pointer.is_moving() || flags & pollocad_cascade::MouseFlags::BUTTON_CHANGE != 0 || wheel != 0 {
                                println!("mouse {x} {y} {wheel} {flags}");
                                preview.mouse_event(x, y, wheel, flags).unwrap();
                            }
                        });
                    }

                    let preview = self.preview.clone();
                    let angle = self.angle;

                    let cb = eframe::egui_glow::CallbackFn::new(move |info, _painter| {
                        preview.lock().unwrap()
                            .paint(
                                (info.viewport.left() * ppp) as u32, (info.viewport.top() * ppp) as u32,
                                (info.viewport.width() * ppp) as u32, (info.viewport.height() * ppp) as u32,
                                angle).expect("paint failed");
                    });

                    ui.painter().add(egui::PaintCallback {
                        rect,
                        callback: Arc::new(cb),
                    });
                       /* .prepare(move |device, queue, _encoder, paint_callback_resources| {
                            let resources: &mut RendererResources = paint_callback_resources.get_mut().unwrap();
                            resources.prepare(device, queue, mesh_data.as_ref(), angle, view_proj);
                            Vec::new()
                        })
                        .paint(move |_info, render_pass, paint_callback_resources| {
                            let resources: &RendererResources = paint_callback_resources.get().unwrap();
                            resources.paint(render_pass, num_vertices);
                        });

                    let callback = egui::PaintCallback {
                        rect,
                        callback: Arc::new(cb),
                    };

                    ui.painter().add(callback);*/


                    //self.angle += response.drag_delta().x * 0.01;
                });
        });
    }
}
