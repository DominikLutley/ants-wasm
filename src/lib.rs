mod functions;
use functions::*;
// use web_sys::console;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
mod consts;
// use easybench_wasm::bench;
mod ants;
use ants::*;
mod grid;
use grid::*;
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

    let mut ant_renderer = AntRenderer::new(&gl, width, height).expect("Error initializing ant renderer");
    let grid_renderer = GridRenderer::new(&gl, width, height).expect("Error initializing grid renderer");

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        clear(&gl);
        ant_renderer.render(&gl, &mut rng);
        grid_renderer.render(&gl);

        // ctx.set_fill_style(&JsValue::from_str("#88f"));
        // draw_nest(&ctx, width / 2.0, height / 2.0);

        // for idx in 0..pheromone_map.len() {
        //     let intensity = pheromone_map[idx];
        //     if intensity == 0 {
        //         continue;
        //     }
        //     let x = idx % width as usize;
        //     let y = idx / width as usize;
        //     ctx.set_fill_style(&JsValue::from_str(&format!("#ff8888{:02x}", intensity)));
        //     ctx.fill_rect(x as f64, y as f64, 1.0, 1.0);
        //     pheromone_map[idx] -= 2;
        // }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
