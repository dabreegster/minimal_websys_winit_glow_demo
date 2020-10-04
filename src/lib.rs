use glow::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::web::WindowExtWebSys;
use winit::window::WindowBuilder;

pub fn main() {
    // Winit setup
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

    // Glow setup
    let webgl2_context = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .unwrap();
    let gl = glow::Context::from_webgl2_context(webgl2_context);
    unsafe {
        let vertex_array = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vertex_array));

        let program = gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
            r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", "#version 300 es", shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!(gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!(gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program));
        gl.clear_color(0.1, 0.2, 0.3, 1.0);
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        log::debug!("{:?}", event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                // TODO window.request_redraw(); ? Or not needed on web?
            }
            Event::RedrawRequested(_) => unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
            },
            _ => (),
        }
    });
}

#[wasm_bindgen(start)]
pub fn run() {
    console_log::init_with_level(log::Level::Debug).unwrap();

    main();
}
