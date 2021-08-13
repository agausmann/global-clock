mod background;
mod clock_face;
mod globe;
pub(crate) mod macros;
mod viewport;

use self::background::Background;
use self::clock_face::ClockFace;
use self::globe::Globe;
use self::viewport::Viewport;
use anyhow::Context;
use chrono::{Local, Utc};
use instant::{Duration, Instant};
use pollster::block_on;
use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub type GraphicsContext = Arc<GraphicsContextInner>;

pub struct GraphicsContextInner {
    pub window: Window,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub render_format: wgpu::TextureFormat,
}

impl GraphicsContextInner {
    async fn new(window: Window) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .context("failed to create adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let render_format = adapter
            .get_swap_chain_preferred_format(&surface)
            .context("failed to select a render format")?;

        Ok(Self {
            window,
            surface,
            device,
            queue,
            render_format,
        })
    }
}

struct App {
    gfx: GraphicsContext,
    viewport: Viewport,
    background: Background,
    globe: Globe,
    clock_face: ClockFace,
    swap_chain: Option<wgpu::SwapChain>,
}

impl App {
    async fn new(window: Window) -> anyhow::Result<Self> {
        let gfx = Arc::new(GraphicsContextInner::new(window).await?);
        let viewport = Viewport::new(&gfx);
        let background = Background::new(&gfx);
        let globe = Globe::new(&gfx, &viewport)?;
        let clock_face = ClockFace::new(&gfx, &viewport)?;

        Ok(Self {
            gfx,
            viewport,
            background,
            globe,
            clock_face,
            swap_chain: None,
        })
    }

    fn update(&mut self) {
        let date = Utc::now();
        self.globe.set_date(&date);
        self.clock_face.set_time(&date.with_timezone(&Local).time())
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        let frame = loop {
            match self.swap_chain().get_current_frame() {
                Ok(frame) => break frame.output,
                Err(wgpu::SwapChainError::Lost) => {
                    self.swap_chain = None;
                }
                Err(wgpu::SwapChainError::Timeout) | Err(wgpu::SwapChainError::Outdated) => {
                    return Ok(());
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        };

        let mut encoder = self.gfx.device.create_command_encoder(&Default::default());

        self.background.draw(&mut encoder, &frame.view);
        self.globe.draw(&mut encoder, &frame.view, &self.viewport);
        self.clock_face
            .draw(&mut encoder, &frame.view, &self.viewport);
        self.gfx.queue.submit([encoder.finish()]);

        Ok(())
    }

    fn window_resized(&mut self) {
        self.swap_chain = None;
        self.viewport.window_resized();
    }

    fn swap_chain(&mut self) -> &wgpu::SwapChain {
        // Split borrows (otherwise the closure will capture `self` entirely)
        let &mut Self { ref gfx, .. } = self;
        self.swap_chain.get_or_insert_with(|| {
            gfx.device.create_swap_chain(
                &gfx.surface,
                &wgpu::SwapChainDescriptor {
                    usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                    format: gfx.render_format,
                    width: gfx.window.inner_size().width,
                    height: gfx.window.inner_size().height,
                    present_mode: wgpu::PresentMode::Fifo,
                },
            )
        })
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // The window decorations provided by winit when using wayland do not match the native system
    // theme, so fallback to X11 via XWayland if possible.
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(720, 720))
        .with_title("Global Clock")
        .build(&event_loop)?;

    let mut app = block_on(App::new(window))?;

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            // Get the ball rolling with an initial timeout of NOW
            *control_flow = ControlFlow::WaitUntil(Instant::now());
        }
        Event::NewEvents(StartCause::ResumeTimeReached {
            requested_resume, ..
        }) => {
            *control_flow = ControlFlow::WaitUntil(requested_resume + Duration::from_secs(1));
            app.gfx.window.request_redraw();
        }
        Event::RedrawRequested(..) => {
            app.update();
            app.redraw().unwrap();
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                app.window_resized();
            }
            _ => {}
        },
        _ => {}
    })
}
