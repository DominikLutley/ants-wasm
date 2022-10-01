mod functions;
use consts::GRID_SIZE;
use functions::*;
use pheromones::PheromoneRenderer;
use web_sys::WebGl2RenderingContext;
// use web_sys::console;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
mod consts;
// use easybench_wasm::bench;
mod ants;
use ants::*;
mod grid;
use grid::*;
mod pheromones;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    let window = window();
    let (width, height, gl) = get_canvas_dimensions_and_context(&window);
    let mut rng = Xoshiro256Plus::seed_from_u64(0);

    // let mut pheromone_map: Vec<u8> = vec![0; (width * height) as usize];
    // console::log_1(&format!("Normal RNG: {}", bench(|| {
    //     // bench
    // })).into());

    // let memory_buffer = wasm_bindgen::memory()
    //     .dyn_into::<js_sys::WebAssembly::Memory>()
    //     .unwrap()
    //     .buffer();

    let grid_renderer = GridRenderer::new(&gl, width, height)
        .expect("Error initializing grid renderer");
    let mut ant_renderer =
        AntRenderer::new(&gl, width, height).expect("Error initializing ant renderer");
    let mut pheromone_renderer =
        PheromoneRenderer::new(&gl, width, height).expect("Error initializing pheromone renderer");

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        clear(&gl);
        grid_renderer.render(&gl);
        ant_renderer.render(&gl, &mut rng, &grid_renderer, &mut pheromone_renderer);
        pheromone_renderer.render(&gl);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
