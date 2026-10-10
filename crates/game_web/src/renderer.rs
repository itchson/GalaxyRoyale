//! Portable instanced pixel quads. No compute or storage buffers are required.
use engine_core::{CameraView, GpuTransform, PixelInstance, RenderFrame, Vec2};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

const TRANSFORM_BYTES: u64 = 128 * 32;
const PIXEL_LIMIT: usize = 384_000;
struct Draw {
    offset: u64,
    bytes: u64,
    count: u32,
    table: u32,
    ui: bool,
}
pub struct Renderer {
    pub canvas: HtmlCanvasElement,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    present_pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    ui_globals: wgpu::Buffer,
    transforms: wgpu::Buffer,
    instances: wgpu::Buffer,
    geometry_layout: wgpu::BindGroupLayout,
    geometry_bind: wgpu::BindGroup,
    ui_geometry_bind: wgpu::BindGroup,
    present_layout: wgpu::BindGroupLayout,
    present_bind: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    logical: [u32; 2],
    scale: u32,
    table_capacity: usize,
    instance_capacity: usize,
    pixel_scratch: Vec<PixelInstance>,
    transform_scratch: Vec<GpuTransform>,
    draws: Vec<Draw>,
    lost: Arc<AtomicBool>,
}
fn js(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
fn uniform(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
fn instances(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pixel instances"),
        size: (capacity * 16) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
fn target(device: &wgpu::Device, size: [u32; 2]) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("logical pixel universe"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}
fn geometry_bind(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    globals: &wgpu::Buffer,
    transforms: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pixel frame bindings"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: transforms,
                    offset: 0,
                    size: std::num::NonZeroU64::new(TRANSFORM_BYTES),
                }),
            },
        ],
    })
}
fn present_bind(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("nearest pixel presentation"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
impl Renderer {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        #[cfg(feature = "webgpu")]
        let backends = wgpu::Backends::BROWSER_WEBGPU;
        #[cfg(not(feature = "webgpu"))]
        let backends = wgpu::Backends::GL;
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(js)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(js)?;
        let limits = wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Galaxy Royale portable device"),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                ..Default::default()
            })
            .await
            .map_err(js)?;
        let lost = Arc::new(AtomicBool::new(false));
        let flag = lost.clone();
        device.set_device_lost_callback(move |_, _| {
            flag.store(true, Ordering::Relaxed);
        });
        let mut config = surface
            .get_default_config(&adapter, canvas.width().max(1), canvas.height().max(1))
            .ok_or_else(|| js("No supported presentation format"))?;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);
        let geometry_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("portable transform layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            (crate::lighting::UNIFORM_FLOATS * 4) as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: std::num::NonZeroU64::new(TRANSFORM_BYTES),
                    },
                    count: None,
                },
            ],
        });
        let globals = uniform(
            &device,
            "camera and bounded pixel lighting",
            (crate::lighting::UNIFORM_FLOATS * 4) as u64,
        );
        let ui_globals = uniform(
            &device,
            "fixed logical pixel UI projection",
            (crate::lighting::UNIFORM_FLOATS * 4) as u64,
        );
        let transforms = uniform(&device, "object transform tables", TRANSFORM_BYTES * 32);
        let ui_geometry_bind = geometry_bind(&device, &geometry_layout, &ui_globals, &transforms);
        let geometry_bind = geometry_bind(&device, &geometry_layout, &globals, &transforms);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("pixel quad shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pixel.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pixel pipeline layout"),
            bind_group_layouts: &[Some(&geometry_layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![0=>Float32x2,1=>Uint32,2=>Uint32];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("instanced local pixel quads"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &attributes,
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let present_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("presentation texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let present_source = include_str!("present.wgsl").replace(
            "MANUAL_SRGB_VALUE",
            if config.format.is_srgb() {
                "false"
            } else {
                "true"
            },
        );
        let present_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("crisp scene presentation"),
            source: wgpu::ShaderSource::Wgsl(present_source.into()),
        });
        let present_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("presentation pipeline layout"),
                bind_group_layouts: &[Some(&present_layout)],
                immediate_size: 0,
            });
        let present_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nearest scene presentation"),
            layout: Some(&present_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &present_shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &present_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest pixel sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let logical = [800, 360];
        let (target, target_view) = target(&device, logical);
        let present_bind = present_bind(&device, &present_layout, &target_view, &sampler);
        let instances = instances(&device, 131072);
        Ok(Self {
            canvas,
            surface,
            device,
            queue,
            config,
            pipeline,
            present_pipeline,
            globals,
            ui_globals,
            transforms,
            instances,
            geometry_layout,
            geometry_bind,
            ui_geometry_bind,
            present_layout,
            present_bind,
            sampler,
            target,
            target_view,
            logical,
            scale: 1,
            table_capacity: 32,
            instance_capacity: 131072,
            pixel_scratch: Vec::with_capacity(131072),
            transform_scratch: Vec::with_capacity(4096),
            draws: Vec::with_capacity(32),
            lost,
        })
    }
    pub fn viewport(&self) -> [f32; 2] {
        [self.logical[0] as f32, self.logical[1] as f32]
    }
    pub fn lose_device(&self) {
        self.device.destroy();
        self.lost.store(true, Ordering::Relaxed);
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        let max = self.device.limits().max_texture_dimension_2d;
        let width = width.clamp(1, max);
        let height = height.clamp(1, max);
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.scale = ((height as f32 / 360.0).round() as u32).max(1);
        self.logical = [(width / self.scale).max(1), (height / self.scale).max(1)];
        (self.target, self.target_view) = target(&self.device, self.logical);
        self.present_bind = present_bind(
            &self.device,
            &self.present_layout,
            &self.target_view,
            &self.sampler,
        );
    }
    pub fn draw(
        &mut self,
        frame: &RenderFrame,
        overlay: &RenderFrame,
        camera: [f32; 2],
        zoom: f32,
        lights: &[game_core::LightSource],
        biome: u8,
    ) -> Result<(), JsValue> {
        if self.lost.load(Ordering::Relaxed) {
            return Err(js("GPU device lost"));
        }
        if frame.pixel_count.saturating_add(overlay.pixel_count) > PIXEL_LIMIT {
            return Err(js("Pixel scene exceeded its explicit resource limit"));
        }
        let view = CameraView::new(
            Vec2::from_array(camera),
            Vec2::from_array(self.viewport()),
            zoom,
        )
        .ok_or_else(|| js("Invalid camera projection"))?;
        self.pixel_scratch.clear();
        self.transform_scratch.clear();
        self.draws.clear();
        // World and overlay retain separate pooled extractions, but share the
        // same GPU buffers and ordered render pass. Their uniform projections
        // differ; no inverse-zoom mutation of glyph or control transforms occurs.
        for (frame, ui) in [(frame, false), (overlay, true)] {
            for batch in &frame.batches {
                if batch.pixels.is_empty() {
                    continue;
                }
                let offset = (self.pixel_scratch.len() * 16) as u64;
                self.pixel_scratch.extend_from_slice(&batch.pixels);
                let table = self.transform_scratch.len() / 128;
                self.transform_scratch.extend(
                    batch
                        .transforms
                        .iter()
                        .map(|transform| view.relative_transform(*transform)),
                );
                self.transform_scratch.resize(
                    (table + 1) * 128,
                    GpuTransform {
                        a: [0.0; 4],
                        b: [0.0; 4],
                    },
                );
                self.draws.push(Draw {
                    offset,
                    bytes: (batch.pixels.len() * 16) as u64,
                    count: batch.pixels.len() as u32,
                    table: table as u32,
                    ui,
                });
            }
        }
        if self.pixel_scratch.len() > self.instance_capacity {
            self.instance_capacity = self.pixel_scratch.len().next_power_of_two();
            self.instances = instances(&self.device, self.instance_capacity);
        }
        let tables = self.transform_scratch.len() / 128;
        if tables > self.table_capacity {
            self.table_capacity = tables.next_power_of_two();
            self.transforms = uniform(
                &self.device,
                "grown transform tables",
                TRANSFORM_BYTES * self.table_capacity as u64,
            );
            self.geometry_bind = geometry_bind(
                &self.device,
                &self.geometry_layout,
                &self.globals,
                &self.transforms,
            );
            self.ui_geometry_bind = geometry_bind(
                &self.device,
                &self.geometry_layout,
                &self.ui_globals,
                &self.transforms,
            );
        }
        self.queue.write_buffer(
            &self.globals,
            0,
            bytemuck::cast_slice(&crate::lighting::uniforms_with_ambient(
                view.world_extent().to_array(),
                camera,
                lights,
                crate::lighting::ambient_for_biome(biome),
            )),
        );
        self.queue.write_buffer(
            &self.ui_globals,
            0,
            bytemuck::cast_slice(&crate::lighting::uniforms(self.viewport(), camera, &[])),
        );
        if !self.pixel_scratch.is_empty() {
            self.queue.write_buffer(
                &self.instances,
                0,
                bytemuck::cast_slice(&self.pixel_scratch),
            );
            self.queue.write_buffer(
                &self.transforms,
                0,
                bytemuck::cast_slice(&self.transform_scratch),
            );
        }
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(value)
            | wgpu::CurrentSurfaceTexture::Suboptimal(value) => value,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err(js("Presentation surface lost")),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(js("Presentation surface validation failed"));
            }
        };
        let output_view = output.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Galaxy Royale frame"),
            });
        {
            let attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &self.target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0001,
                        g: 0.00011,
                        b: 0.00014,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pixel world"),
                color_attachments: &attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            for draw in &self.draws {
                pass.set_bind_group(
                    0,
                    if draw.ui {
                        &self.ui_geometry_bind
                    } else {
                        &self.geometry_bind
                    },
                    &[draw.table * TRANSFORM_BYTES as u32],
                );
                pass.set_vertex_buffer(
                    0,
                    self.instances.slice(draw.offset..draw.offset + draw.bytes),
                );
                pass.draw(0..6, 0..draw.count);
            }
        }
        {
            let attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("integer nearest upscale"),
                color_attachments: &attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let w = self.logical[0] * self.scale;
            let h = self.logical[1] * self.scale;
            pass.set_viewport(
                ((self.config.width - w) / 2) as f32,
                ((self.config.height - h) / 2) as f32,
                w as f32,
                h as f32,
                0.0,
                1.0,
            );
            pass.set_pipeline(&self.present_pipeline);
            pass.set_bind_group(0, &self.present_bind, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(output);
        Ok(())
    }
}
