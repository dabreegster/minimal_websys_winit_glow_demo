# minimal_websys_winit_glow_demo

This is a simple demo of how to make winit, glow, and web-sys work together.

TODO:

- [ ] Bring in [glow](https://github.com/grovesNL/glow/tree/main/examples/hello)
- [ ] Make the example also compile and run natively

To run:

1.  Make sure you have [wasm-pack](https://github.com/rustwasm/wasm-pack) and
    Python 3 (for a simple HTTP file server)
2.  Run `./run.sh`
3.  Open http://0.0.0.0:8000 in your browser

Credits:

- [winit web example](https://github.com/rust-windowing/winit/blob/master/examples/web.rs)
- [wasm-project-template](https://github.com/alvinhochun/wasm-project-template) and help from Alvin with build instructions
