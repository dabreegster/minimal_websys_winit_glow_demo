use wasm_bindgen::prelude::*;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::web::WindowExtWebSys;
use winit::window::WindowBuilder;

pub fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .build(&event_loop)
        .unwrap();

    let canvas = window.canvas();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    body.append_child(&canvas)
        .expect("Append canvas to HTML body");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        log::debug!("{:?}", event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                //window.request_redraw();
            }
            _ => (),
        }
    });
}

#[wasm_bindgen(start)]
pub fn run() {
    console_log::init_with_level(log::Level::Debug).unwrap();

    main();
}
