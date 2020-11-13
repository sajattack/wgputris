//use std::io::Cursor;

use wgpu::util::DeviceExt;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

mod texture;
mod gameboard;
mod game;
mod tetromino;

const BLOCK_SIZE: u32 = 12;
const GAMEBOARD_OFFSET: (usize, usize) = (15, 1);
const GAMEBOARD_WIDTH: usize = 10;
const GAMEBOARD_HEIGHT: usize = 20;
//const TETRIS_SONG: [u8; 410354] = *include_bytes!("../assets/tetris.ogg");

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32;3],
    pub tex_coords: [f32;2],
    pub color: [f32;4]
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32;5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float4,
                }
            ]
        }
    }
}

unsafe impl bytemuck::Pod for Vertex{}
unsafe impl bytemuck::Zeroable for Vertex{}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Uniforms {
    view_proj: cgmath::Matrix4<f32>,
}

impl Uniforms {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        let proj = cgmath::ortho(0.0, 480.0, 272.0, 0.0, -1.0, 1.0);
        let view = cgmath::Matrix4::identity();
        Self {
            view_proj: OPENGL_TO_WGPU_MATRIX * proj * view
        }
    }
}

unsafe impl bytemuck::Pod for Uniforms{}
unsafe impl bytemuck::Zeroable for Uniforms{}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,
    sc_desc: wgpu::SwapChainDescriptor,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    diffuse_bind_group: wgpu::BindGroup,
    game: game::Game,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                shader_validation: true,
            },
            None,
        ).await.unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../shaders/shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../shaders/shader.frag.spv"));

        let diffuse_bytes = include_bytes!("../assets/block.png");
        let diffuse_texture = texture::Texture::from_png_bytes(&device, &queue, diffuse_bytes, "block").unwrap();

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            }
        );

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        use bytemuck::Zeroable;
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&[Vertex::zeroed();1254]),
                usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniforms = Uniforms::new();

        let uniform_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            }
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("uniform_bind_group_layout"),
        });
        
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..))
                }
            ],
            label: Some("uniform_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &uniform_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),

            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },

            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),

            rasterization_state: Some(
                wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: wgpu::CullMode::Back,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                    clamp_depth: false,
                }
            ),

            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
            ],

            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    Vertex::desc(),
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let game = game::Game::new();

        Self {
            device,
            queue,
            swap_chain,
            sc_desc,
            render_pipeline,
            vertex_buffer,
            uniform_bind_group,
            diffuse_bind_group,
            game,
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput{input, ..} = event {
            return self.game.process_input(*input);
        }
        false
    }

    fn update(&mut self) {
        // TODO game start and game over
        self.game.process_game_loop();
    }

    fn render(&mut self) {
        let frame = self.swap_chain.get_current_frame()
            .expect("Timeout getting texture")
            .output;
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.2,
                                g: 0.267,
                                b: 0.333,
                                a: 1.0,
                            }),
                            store: true,
                        }
                    }
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

            let vertices = self.game.render();
            self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);

        }
            let mut staging_belt = wgpu::util::StagingBelt::new(1024);
            let font = ab_glyph::FontArc::try_from_slice(include_bytes!("../assets/RedOctober.ttf")).expect("Load font");

            let mut glyph_brush = GlyphBrushBuilder::using_font(font)
                .build(&self.device, self.sc_desc.format);

            let score_string = format!("Score: {}", self.game.get_score());
            let score_text = Section {
                screen_position: (680.0, 80.0),
                text: vec![Text::new(&score_string).with_color([1.0, 1.0, 1.0, 1.0])],
                ..Section::default()
            };

            glyph_brush.queue(score_text);

            let next_shape_text = Section {
                screen_position: (680.0, 120.0),

                text: vec![Text::new("Next Shape:").with_color([1.0, 1.0, 1.0, 1.0])],
                ..Section::default()
            };

            glyph_brush.queue(next_shape_text);

            glyph_brush.draw_queued(
                &self.device,
                &mut staging_belt,
                &mut encoder,
                &frame.view,
                960,
                544
            ).expect("Draw queued");

            staging_belt.finish();
            self.queue.submit(std::iter::once(encoder.finish()));
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("wgputris")
        .with_inner_size(winit::dpi::PhysicalSize::new(960, 544))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    use futures::executor::block_on;
    let mut state = block_on(State::new(&window));

    //let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    //let source = rodio::Decoder::new_looped(Cursor::new(&TETRIS_SONG)).unwrap();
    //let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    //sink.append(source);
    //sink.play();
    //sink.detach();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() =>  if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                state.render();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
