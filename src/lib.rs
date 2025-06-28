#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use winit::event_loop::EventLoop;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, StartCause, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

#[cfg(not(target_os = "android"))]
use wry::{
    dpi::{LogicalPosition, LogicalSize},
    Rect, WebViewBuilder,
};

pub struct AppHandler<'a> {
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    config: Option<wgpu::SurfaceConfiguration>,

    #[cfg(not(target_os = "android"))]
    webview: Option<wry::WebView>,
}

impl<'a> AppHandler<'a> {
    pub fn new() -> Self {
        Self {
            surface: None,
            device: None,
            queue: None,
            render_pipeline: None,
            config: None,
            #[cfg(not(target_os = "android"))]
            webview: None,
        }
    }
}

impl<'a> ApplicationHandler<()> for AppHandler<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attr = WindowAttributes::default()
            .with_title("Snow Player")
            .with_transparent(true);
        let window = event_loop.create_window(attr).unwrap();
        let size = window.inner_size();

        #[cfg(not(target_os = "android"))]
        {
            self.webview = Some(
                WebViewBuilder::new()
                    .with_bounds(Rect {
                        position: LogicalPosition::new(0, 0).into(),
                        size: LogicalSize::new(200, 200).into(),
                    })
                    .with_transparent(true)
                    .with_html(
                        r#"<html>
                    <body style="background-color:rgba(87,87,87,0.5);"></body>
                    <script>
                        window.onload = function() {
                            document.body.innerText = `سلام, ${navigator.userAgent}`;
                        };
                    </script>
                </html>"#,
                    )
                    .build_as_child(&window)
                    .unwrap(),
            );
        }

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();

        log::info!("Adapter: {:?}", adapter.get_info());
        log::info!("Device: {:?}", device.limits());

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Inline Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                    @vertex
                    fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
                        var positions = array<vec2<f32>, 3>(
                            vec2<f32>(0.0,  0.5),
                            vec2<f32>(-0.5, -0.5),
                            vec2<f32>(0.5, -0.5)
                        );
                        let pos = positions[vertex_index];
                        return vec4<f32>(pos, 0.0, 1.0);
                    }

                    @fragment
                    fn fs_main() -> @location(0) vec4<f32> {
                        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                    }
                "#.into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.render_pipeline = Some(render_pipeline);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let (Some(surface), Some(device), Some(config)) =
            (&self.surface, &self.device, &mut self.config)
        {
            match event {
                WindowEvent::Resized(new_size) => {
                    config.width = new_size.width;
                    config.height = new_size.height;
                    surface.configure(device, config);
                }
                WindowEvent::RedrawRequested => self.draw_frame(),
                WindowEvent::CloseRequested => event_loop.exit(),
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.draw_frame();
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }
}

// Private method implementation for AppHandler
impl<'a> AppHandler<'a> {
    fn draw_frame(&mut self) {
        if let (Some(surface), Some(device), Some(queue), Some(pipeline), Some(config)) = (
            &self.surface,
            &self.device,
            &self.queue,
            &self.render_pipeline,
            &self.config,
        ) {
            let output = match surface.get_current_texture() {
                Ok(frame) => frame,
                Err(_) => {
                    surface.configure(device, config);
                    surface.get_current_texture().unwrap()
                }
            };
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });
                rpass.set_pipeline(pipeline);
                rpass.draw(0..3, 0..1);
            }
            queue.submit(Some(encoder.finish()));
            output.present();
        }
    }
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_tag("wgpu"),
    );

    log::info!("Starting Snow Player on Android");

    let event_loop = EventLoop::builder().with_android_app(app).build().unwrap();
    let mut handler = AppHandler::new();
    event_loop.run_app(&mut handler).unwrap();
}

#[allow(dead_code)]
pub fn main() {
    log::info!("Starting Snow Player on Desktop");
    let mut handler = AppHandler::new();
    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut handler).unwrap();
}
