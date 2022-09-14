mod functions;
use functions::{get_canvas_dimensions_and_context, initialize_ants, draw_ant, draw_nest};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
mod consts;
use consts::*;
use easybench_wasm::bench;
use web_sys::{console, window};
use rand::prelude::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

#[derive(Clone, Debug)]
pub struct Ant {
    pub x: f64,
    pub y: f64,
    pub dir: f64,
    pub has_food: bool,
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    let window = window().expect("no global `window` object exists");
    let (width, height, ctx) = get_canvas_dimensions_and_context(&window);
    let mut ants = initialize_ants(width, height, ANT_COUNT);
    let mut pheromone_map: Vec<u8> = vec![0; (width * height) as usize];
    let mut rng = Xoshiro256Plus::seed_from_u64(0);
    // console::log_1(&format!("Normal RNG: {}", bench(|| {
    //     // bench
    // })).into());
    console::log_1(&format!("ff8888{:02x}", 10).into());

    let next_frame = Closure::wrap(Box::new(move || {
        ctx.clear_rect(0.0, 0.0, width, height);

        ctx.set_fill_style(&JsValue::from_str("#88f"));
        draw_nest(&ctx, width / 2.0, height / 2.0);

        for idx in 0..pheromone_map.len() {
            let intensity = pheromone_map[idx];
            if intensity == 0 { continue; }
            let x = idx % width as usize;
            let y = idx / width as usize;
            ctx.set_fill_style(&JsValue::from_str(&format!("#ff8888{:02x}", intensity)));
            ctx.fill_rect(x as f64, y as f64, 1.0, 1.0);
            pheromone_map[idx] -= 2;
        }

        ctx.set_fill_style(&JsValue::from_str("#fff"));
        for ant in &mut ants {
            if ant.x >= width - ANT_RADIUS || ant.x <= 0.0 + ANT_RADIUS {
                ant.dir += (PI / 2.0 - ant.dir) * 2.0;
            } else if ant.y <= 0.0 + ANT_RADIUS || ant.y >= height - ANT_RADIUS {
                ant.dir -= 2.0 * ant.dir;
            }
            ant.dir += (rng.gen::<f64>() - 0.5) * WANDER_COEFFICIENT;
            ant.x += ant.dir.cos() * WALK_SPEED;
            ant.y += ant.dir.sin() * WALK_SPEED;
            draw_ant(&ctx, ant.x, ant.y);
            let idx = ant.y as usize * width as usize + ant.x as usize;
            if idx > pheromone_map.len() { continue; }
            pheromone_map[idx] = 254;
        }
    }) as Box<dyn FnMut()>);

    window.set_interval_with_callback_and_timeout_and_arguments_0(
        next_frame.as_ref().unchecked_ref(),
        FRAME_TIME,
    )?;
    next_frame.forget();

    Ok(())
}
