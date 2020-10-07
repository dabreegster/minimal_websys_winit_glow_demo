use futures_channel::oneshot;
use glow::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Request, RequestInit, RequestMode, Response};
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

    let mut http_resp = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        //log::debug!("{:?}", event);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == winit::event::ElementState::Pressed {
                        // Press space to start the HTTP fetch, press R to try to read it.
                        if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Space) {
                            if http_resp.is_none() {
                                let (tx, rx) = oneshot::channel::<String>();
                                http_resp = Some(rx);
                                start_http_call(
                                    "http://0.0.0.0:8000/black_humor.txt".to_string(),
                                    tx,
                                );
                            } else {
                                log::error!(
                                    "Haven't read the previously made request yet; press R first"
                                );
                            }
                        } else if input.virtual_keycode == Some(winit::event::VirtualKeyCode::R) {
                            if let Some(mut channel) = http_resp.take() {
                                if let Some(resp) = channel.try_recv().unwrap() {
                                    log::info!("Got response! {}", resp);
                                } else {
                                    log::info!("No response yet, try again later");
                                    http_resp = Some(channel);
                                }
                            } else {
                                log::error!("Haven't made an HTTP request yet, press space first");
                            }
                        } else if input.virtual_keycode == Some(winit::event::VirtualKeyCode::S) {
                            // TODO This locks up the browser, it doesn't work
                            let (tx, mut rx) = oneshot::channel::<String>();
                            start_http_call("http://0.0.0.0:8000/huge_seattle.bin".to_string(), tx);
                            log::info!("Made request, now spinlock and wait");
                            let mut cnt = 0;
                            loop {
                                cnt += 1;
                                if cnt % 1000 == 0 {
                                    log::info!("poll attempt {}", cnt);
                                }
                                if let Some(resp) = rx.try_recv().unwrap() {
                                    log::info!("Aha, got the response! {}", resp);
                                    break;
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                // TODO window.request_redraw(); ? Or not needed on web?
            }
            Event::RedrawRequested(_) => unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT);
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
            },
            _ => {}
        }
    });
}

fn start_http_call(url: String, tx: oneshot::Sender<String>) {
    spawn_local(async move {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts).unwrap();

        request
            .headers()
            .set("Accept", "application/vnd.github.v3+json")
            .unwrap();

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .unwrap();

        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        // Convert this other `Promise` into a rust `Future`.
        let text = JsFuture::from(resp.text().unwrap()).await.unwrap();
        let actual_txt = format!("{}", text.as_string().unwrap());
        tx.send(actual_txt).unwrap();
    });
}

#[wasm_bindgen(start)]
pub fn run() {
    console_log::init_with_level(log::Level::Debug).unwrap();
    std::panic::set_hook(Box::new(|info| {
        log::error!("Panicked: {}", info);
    }));

    main();
}
