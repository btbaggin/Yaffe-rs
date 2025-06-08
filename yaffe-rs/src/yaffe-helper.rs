use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use wry::WebViewBuilder;
use std::process::{Command, exit};

#[derive(Default)]
struct App {
  url: String,
  window: Option<Window>,
  webview: Option<wry::WebView>,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window = event_loop.create_window(Window::default_attributes()).unwrap();
    window.set_maximized(true);
    let webview = WebViewBuilder::new()
      .with_url(self.url.clone())
      .build(&window)
      .unwrap();

    self.window = Some(window);
    self.webview = Some(webview);
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.window.as_ref().unwrap().request_redraw(),
            _ => (),
        }
  }
}

fn update_yaffe(patch: &str, app: &str) {
    println!("Applying patch file");
    if std::path::Path::new(patch).exists() {
        if let Err(e) = std::fs::rename(patch, app) {
            println!("{e:?}");
        }

        if let Err(e) = Command::new(app).spawn() {
            println!("{e:?}");
        }
    }
}

fn main() {
  let mut args = std::env::args();
  args.next(); // burn current executable
  
  let action = args.next().unwrap();
  match action.as_str() {
    "webview" => {
      let url = args.next().unwrap_or_else(|| exit(1));
      let mut app = App { url, ..Default::default() };
      let event_loop = EventLoop::new().unwrap();
      event_loop.set_control_flow(ControlFlow::Wait);
      event_loop.run_app(&mut app).unwrap();
    },
    "update" => {
      let patch = args.next().unwrap_or_else(|| exit(1));
      let app = args.next().unwrap_or_else(|| exit(1));
      update_yaffe(&patch, &app);
    },
    _ => exit(1),
  }
}
