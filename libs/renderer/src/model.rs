use crate::{Material, Mesh, Renderable, UniformBuffer};

pub struct Model {
    mesh: Mesh,
    material: Material,

    pipeline: wgpu::RenderPipeline,
    bind_group: Option<wgpu::BindGroup>,
}

impl Model {
    pub fn new(device: &wgpu::Device, mesh: Mesh, material: Material) -> Self {
        let attributes = mesh
            .vertex_formats
            .iter()
            .map(|x| x.wgpu_attributes(&material.vertex_shader.inputs))
            .collect::<Vec<_>>();

        let vertex_buffers = attributes
            .iter()
            .zip(mesh.strides.iter())
            .map(|(attributes, stride)| wgpu::VertexBufferDescriptor {
                stride: *stride as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes,
            })
            .collect::<Vec<_>>();

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &material.pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &material.vertex_shader.module,
                entry_point: material.vertex_shader.entry,
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &material.fragment_shader.module,
                entry_point: material.fragment_shader.entry,
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8Unorm,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &vertex_buffers,
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            mesh,
            material,
            pipeline,
            bind_group: None,
        }
    }
}

impl Renderable for Model {
    fn prepare(&mut self, mut command_encoder: &mut wgpu::CommandEncoder) {
        self.material.prepare(&mut command_encoder);
    }

    fn render<'a>(&'a mut self, device: &wgpu::Device, render_pass: &mut wgpu::RenderPass<'a>, mvp_buf: UniformBuffer) {
        // TODO store bind_group in material
        self.bind_group = Some(self.material.bind_group(&device, mvp_buf));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group.as_ref().unwrap(), &[]);
        render_pass.set_index_buffer(&self.mesh.index, 0, 0);
        for (i, vertex_buffer) in self.mesh.vertex_buffers.iter().enumerate() {
            render_pass.set_vertex_buffer(i as u32, &vertex_buffer, 0, 0);
        }
        render_pass.draw_indexed(0..self.mesh.index_count as u32, 0, 0..1);
    }
}
