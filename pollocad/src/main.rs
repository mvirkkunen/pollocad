#![allow(dead_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Very haphazardly copy-pasted together from egui/wgpu examples

use eframe::egui;

mod ast;
mod builtins;
mod geometry;
mod parser;
mod preview;
mod runtime;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 800.0)),
        multisampling: 1,
        renderer: eframe::Renderer::Wgpu,
        depth_buffer: 24,
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
    preview: preview::Renderer,
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

/*const CODE: &'static str = r#"
cube(10, 10, 10);
translate(x=2) cube(10, 10, 10);
"#;*/

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        cc.egui_ctx.set_pixels_per_point(2.0);

        Some(Self {
            code: CODE.to_string(),
            num_indices: 0,
            num_vertices: 0,
            angle: 0.7,
            valid: false,
            preview: preview::Renderer::new(wgpu_render_state),
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
                    let response = self.preview.paint(ui, mesh_data, self.angle);
                    self.angle += response.drag_delta().x * 0.01;
                });
        });
    }
}
