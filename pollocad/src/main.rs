#![allow(dead_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Very haphazardly copy-pasted together from egui/wgpu examples

use std::num::NonZeroU64;
use std::sync::Arc;

use eframe::{
    egui,
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

mod ast;
mod builtins;
mod runtime;
mod geometry;
mod parser;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 1024.0)),
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
    num_indices: u32,
    num_vertices: u32,
    angle: f32,
    valid: bool,
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

const CODE: &'static str = r#"
x = 2;
translate(z=-5, y=-5) union() {
    cube(x, 30, 20);
    translate(z=5, y=5) anti() cube(10, 10, 10);
}

cube(20, 5, 5);

"#;

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("custom3d"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shader.wgsl").into()),
        });

        /*let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom3d"),
            entries: &[],
        });*/

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom3d"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(
                            4 * 4 * std::mem::size_of::<f32>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(16),
                    },
                    count: None,
                },
            ],
        });

        cc.egui_ctx.set_pixels_per_point(2.0);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("custom3d"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("custom3d"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (6 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0 as wgpu::BufferAddress,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: (3 * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu_render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            /*multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },*/
            multiview: None,
        });

        let eye = cgmath::Point3::<f32>::new(0.0, 40.0, -40.0);
        let target = cgmath::Point3::<f32>::new(0.0, 10.0, 0.0);
        let up = cgmath::Vector3::<f32>::new(0.0, 1.0, 0.0);

        let view = cgmath::Matrix4::look_at_rh(eye, target, up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(80.0), 1.0, 0.1, 100.0);

        // 3.
        let view_proj = OPENGL_TO_WGPU_MATRIX * proj * view;
        let view_proj: [[f32; 4]; 4] = view_proj.into();

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&view_proj),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom3d"),
            contents: bytemuck::cast_slice(&[0.0_f32; 4]), // 16 bytes aligned!
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("custom3d"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex"),
            size: 0,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("index"),
            size: 0,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
            mapped_at_creation: false,
        });

        wgpu_render_state
            .renderer
            .write()
            .paint_callback_resources
            .insert(TriangleRenderResources {
                pipeline,
                bind_group,
                uniform_buffer,
                vertex_buffer,
                index_buffer,
            });

        Some(Self {
            code: CODE.to_string(),
            num_indices: 0,
            num_vertices: 0,
            angle: 0.0,
            valid: false,
        })
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui, mesh_data: Option<pollocad_cgal::MeshData>) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        self.angle += response.drag_delta().x * 0.01;
        let angle = self.angle;

        let num_vertices = self.num_vertices;

        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _encoder, paint_callback_resources| {
                let resources: &mut TriangleRenderResources =
                    paint_callback_resources.get_mut().unwrap();
                resources.prepare(device, queue, mesh_data.as_ref(), angle);
                Vec::new()
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let resources: &TriangleRenderResources = paint_callback_resources.get().unwrap();
                resources.paint(render_pass, num_vertices);
            });

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(cb),
        };

        ui.painter().add(callback);
    }
}

struct TriangleRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl TriangleRenderResources {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_data: Option<&pollocad_cgal::MeshData>,
        angle: f32,
    ) {
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[angle, 0.0, 0.0, 0.0]),
        );

        if let Some(mesh_data) = mesh_data {
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex"),
                contents: mesh_data.vertex_data(),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("index"),
                contents: mesh_data.index_data(),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
            });

            println!("Updated data");

            self.vertex_buffer = vertex_buffer;
            self.index_buffer = index_buffer;
        }
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>, num_vertices: u32) {
        //let Some(vertex_buffer) = self.vertex_buffer.as_ref() else { return };
        //let Some(index_buffer) = self.index_buffer.as_ref() else { return };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        //render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw(0..num_vertices, 0..1);
        //render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //egui::widgets::global_dark_light_mode_buttons(ui);

            let mut mesh_data = None;

            egui::SidePanel::left("code_panel")
                .resizable(true)
                .default_width(512.0)
                .show_inside(ui, |ui| {
                    ui.set_min_width(256.0);
                    let response = ui.text_edit_multiline(&mut self.code);

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
                                            self.num_indices = (m.index_data().len() / 2) as u32;

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
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    egui::Frame::canvas(ui.style())
                        .fill(egui::Color32::WHITE)
                        .show(ui, |ui| {
                            self.custom_painting(ui, mesh_data);
                        });
                });
            });
        });
    }
}
