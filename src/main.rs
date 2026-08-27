mod any_shape;
mod document;
mod app;
mod camera;
mod tmp;
mod cli;
mod config;
mod element;
mod external_event;
mod style;

fn main() {
    app::run().unwrap();
}

// use vello::kurbo::Size;
// use winit::application::ApplicationHandler;
// use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
// use winit::event_loop::{ActiveEventLoop, EventLoop};
// use winit::keyboard::{Key, NamedKey};
// use winit::window::{Window, WindowAttributes, WindowId};
//
// use vello::peniko::color::palette;
// use vello::util::{RenderContext, RenderSurface};
// use vello::wgpu::CurrentSurfaceTexture;
// use vello::{AaConfig, Renderer, RendererOptions, Scene, wgpu};
//
// use vello::kurbo::{
//     Affine, BezPath, Circle, CircleSegment, CubicBez, Ellipse, Line, Point, QuadBez, Rect,
//     RoundedRect, Triangle, Vec2,
// };
//
// use std::sync::Arc;
//
// mod app;
// mod camera;
// mod cli;
// mod config;
// mod elem;
//
// use elem::{Elem, Style as ElStyle};
//
// use camera::Camera;
//
// enum RenderState {
//     Suspended(Option<Arc<Window>>),
//     Active {
//         surface: RenderSurface<'static>,
//         window: Arc<Window>,
//         renderer: Renderer,
//     },
// }
//
// struct MouseState {}
//
// struct App {
//     init_window_attr: Option<WindowAttributes>,
//     rctx: RenderContext,
//     rstate: RenderState,
//     scene: Scene,
//     camera: Camera,
//     els: Vec<elem::El>,
//
//     cursor_pos: Point,
//     panning: bool,
//     pan_last: Point,
// }
//
// impl ApplicationHandler for App {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         let RenderState::Suspended(cached_window) = &mut self.rstate else {
//             return;
//         };
//
//         let attr = self.init_window_attr.take().unwrap_or_default();
//         let window = cached_window
//             .take()
//             .unwrap_or_else(|| Arc::new(event_loop.create_window(attr).unwrap()));
//
//         let size = window.inner_size();
//         let surface_future = self.rctx.create_surface(
//             window.clone(),
//             size.width,
//             size.height,
//             wgpu::PresentMode::AutoVsync,
//         );
//         let surface = pollster::block_on(surface_future).expect("Error creating surface");
//         let renderer = Renderer::new(
//             &self.rctx.devices[surface.dev_id].device,
//             RendererOptions::default(),
//         )
//         .expect("Couldn't create renderer");
//
//         self.rstate = RenderState::Active {
//             surface,
//             window,
//             renderer,
//         };
//     }
//
//     fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
//         if let RenderState::Active { window, .. } = &self.rstate {
//             self.rstate = RenderState::Suspended(Some(window.clone()));
//         }
//     }
//
//     fn window_event(
//         &mut self,
//         event_loop: &ActiveEventLoop,
//         window_id: WindowId,
//         event: WindowEvent,
//     ) {
//         let (surface, window, renderer) = match &mut self.rstate {
//             RenderState::Active {
//                 surface,
//                 window,
//                 renderer,
//             } if window.id() == window_id => (surface, window, renderer),
//             _ => return,
//         };
//
//         match event {
//             WindowEvent::CloseRequested => event_loop.exit(),
//             WindowEvent::Resized(size) => {
//                 if size.width != 0 && size.height != 0 {
//                     self.rctx.resize_surface(surface, size.width, size.height);
//                     self.camera.state_mut().viewport =
//                         Size::new(size.width as f64, size.height as f64);
//                     window.request_redraw();
//                 }
//             }
//             WindowEvent::CursorMoved { position, .. } => {
//                 self.cursor_pos = Point::new(position.x, position.y);
//
//                 if self.panning {
//                     let delta = self.cursor_pos - self.pan_last;
//                     // экранный сдвиг -> мировой, с учётом zoom и rotation
//                     let zoom = self.camera.state_mut().zoom; // см. примечание ниже
//                     let (sin_t, cos_t) = self.camera.state_mut().rotation.sin_cos();
//                     let dx = cos_t * delta.x + sin_t * delta.y;
//                     let dy = -sin_t * delta.x + cos_t * delta.y;
//
//                     let s = self.camera.state_mut();
//                     s.position.x -= dx / s.zoom;
//                     s.position.y -= dy / s.zoom;
//
//                     self.pan_last = self.cursor_pos;
//                     window.request_redraw();
//                 }
//             }
//
//             WindowEvent::MouseInput { state, button, .. } => {
//                 if button == MouseButton::Middle || button == MouseButton::Left {
//                     match state {
//                         ElementState::Pressed => {
//                             self.panning = true;
//                             self.pan_last = self.cursor_pos;
//                         }
//                         ElementState::Released => {
//                             self.panning = false;
//                         }
//                     }
//                 }
//             }
//
//             WindowEvent::MouseWheel { delta, .. } => {
//                 let scroll_y = match delta {
//                     MouseScrollDelta::LineDelta(_, y) => y as f64,
//                     MouseScrollDelta::PixelDelta(pos) => pos.y / 40.0,
//                 };
//
//                 let factor = 1.1_f64.powf(scroll_y);
//                 self.camera.zoom_by_at(self.cursor_pos, factor);
//                 window.request_redraw();
//             }
//             WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
//                 match event.logical_key.as_ref() {
//                     Key::Named(NamedKey::Space) => {
//                         self.camera.reset_with_viewport();
//                         window.request_redraw();
//                     }
//                     Key::Character(char) => {
//                         match char {
//                             "j" => self.camera.state_mut().zoom += 0.2,
//                             "k" => self.camera.state_mut().zoom -= 0.2,
//                             "h" => self.camera.state_mut().position.x += 2.1,
//                             "l" => self.camera.state_mut().position.x -= 2.1,
//                             _ => {}
//                         }
//                         window.request_redraw();
//                     }
//                     _ => {}
//                 }
//             }
//             WindowEvent::RedrawRequested => {
//                 self.scene.reset();
//
//                 self.camera.render(&mut self.scene, &mut self.els);
//
//                 let width = surface.config.width;
//                 let height = surface.config.height;
//
//                 let device_handle = &self.rctx.devices[surface.dev_id];
//
//                 renderer
//                     .render_to_texture(
//                         &device_handle.device,
//                         &device_handle.queue,
//                         &self.scene,
//                         &surface.target_view,
//                         &vello::RenderParams {
//                             base_color: palette::css::BLACK,
//                             width,
//                             height,
//                             antialiasing_method: AaConfig::Msaa16,
//                         },
//                     )
//                     .expect("failed to render to surface");
//
//                 let surface_texture = match surface.surface.get_current_texture() {
//                     CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
//                     CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Suboptimal(_) => {
//                         self.rctx.configure_surface(surface);
//                         window.request_redraw();
//                         return;
//                     }
//                     CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Timeout => {
//                         window.request_redraw();
//                         return;
//                     }
//                     CurrentSurfaceTexture::Lost => panic!("Surface was lost"),
//                     CurrentSurfaceTexture::Validation => {
//                         panic!("Validation error getting surface")
//                     }
//                 };
//
//                 let mut encoder =
//                     device_handle
//                         .device
//                         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
//                             label: Some("Surface Blit"),
//                         });
//
//                 surface.blitter.copy(
//                     &device_handle.device,
//                     &mut encoder,
//                     &surface.target_view,
//                     &surface_texture
//                         .texture
//                         .create_view(&wgpu::TextureViewDescriptor::default()),
//                 );
//
//                 device_handle.queue.submit([encoder.finish()]);
//
//                 surface_texture.present();
//
//                 device_handle.device.poll(wgpu::PollType::Poll).unwrap();
//             }
//             _ => (),
//         }
//     }
// }
//
// fn main() {
//     let args = cli::Args::parse();
//
//     let event_loop = EventLoop::new().unwrap();
//
//     let mut app = App {
//         init_window_attr: Some(args.config.window_attributes()),
//         rctx: RenderContext::new(),
//         rstate: RenderState::Suspended(None),
//         scene: Scene::new(),
//         camera: Camera::builder()
//             .with_viewport_size(
//                 args.config.window.width as f64,
//                 args.config.window.height as f64,
//             )
//             .build(),
//         els: Vec::new(),
//         cursor_pos: Point::ORIGIN,
//         panning: false,
//         pan_last: Point::ORIGIN,
//     };
//
//     app.els.push(
//         elem::El::builder()
//             .with_shape(Rect::new(10., 10., 40., 40.))
//             .with_style(ElStyle::filled(palette::css::RED))
//             .with_rotation(30.)
//             .build(),
//     );
//     let _ = event_loop.run_app(&mut app);
// }
