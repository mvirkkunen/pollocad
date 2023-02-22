use std::num::NonZeroU64;
use std::sync::Arc;

use bytemuck::Zeroable;
use cgmath::SquareMatrix as _;
use eframe::{
    egui,
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Renderer {
    num_vertices: u32,
    size: egui::Vec2,
    view_proj: cgmath::Matrix4<f32>,
}

impl Renderer {
    pub fn new(wgpu_render_state: &egui_wgpu::RenderState) -> Renderer {
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

        let view_proj = cgmath::Matrix4::identity();
        let view_proj_data: [[f32; 4]; 4] = view_proj.into();

        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&view_proj_data),
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
            .insert(RendererResources {
                pipeline,
                bind_group,
                camera_uniform_buffer,
                uniform_buffer,
                vertex_buffer,
                index_buffer,
            });

        Renderer {
            num_vertices: 0,
            size: egui::Vec2::zeroed(),
            view_proj,
        }
    }

    pub fn paint(
        &mut self,
        ui: &mut egui::Ui,
        mesh_data: Option<pollocad_cgal::MeshData>,
        angle: f32,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

        if rect.size() != self.size && rect.size().x > 0.0 && rect.size().y > 0.0 {
            let eye = cgmath::Point3::<f32>::new(0.0, 40.0, 40.0);
            let target = cgmath::Point3::<f32>::new(0.0, 0.0, 0.0);
            let up = cgmath::Vector3::<f32>::new(0.0, 1.0, 0.0);

            let view = cgmath::Matrix4::look_at_rh(eye, target, up);

            let axis = cgmath::Matrix4::<f32>::new(
                1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, -1.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0
            );

            let proj = cgmath::perspective(
                cgmath::Deg(80.0),
                rect.size().x / rect.size().y,
                0.1,
                1000.0,
            );
            
            self.view_proj = OPENGL_TO_WGPU_MATRIX * proj * view * axis;

            self.size = rect.size();
        }

        if let Some(md) = &mesh_data {
            self.num_vertices = (md.vertex_data().len() / (6 * 4)) as u32;
        }

        let num_vertices = self.num_vertices;
        let view_proj = self.view_proj;

        let cb = egui_wgpu::CallbackFn::new()
            .prepare(move |device, queue, _encoder, paint_callback_resources| {
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

        ui.painter().add(callback);

        response
    }
}

struct RendererResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl RendererResources {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_data: Option<&pollocad_cgal::MeshData>,
        angle: f32,
        view_proj: cgmath::Matrix4<f32>,
    ) {
        let view_proj_data: [[f32; 4]; 4] = view_proj.into();

        queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(&view_proj_data),
        );
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
