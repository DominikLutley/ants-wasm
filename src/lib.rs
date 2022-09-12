mod functions;
use functions::{get_canvas_dimensions_and_context, initialize_ants, draw_ant, draw_nest};
use rand::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
mod consts;
use consts::*;
use easybench_wasm::bench;
use web_sys::{console, window};

#[derive(Clone, Debug)]
pub struct Ant {
    pub x: f64,
    pub y: f64,
    pub dir: f64,
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    let window = window().expect("no global `window` object exists");
    let (width, height, ctx) = get_canvas_dimensions_and_context(&window);
    let mut ants = initialize_ants(width, height, ANT_COUNT);
    let mut rng = rand::thread_rng();
    // console::log_1(&format!("paint nest: {}", bench(|| draw_nest(&ctx, 200.0, 200.0))).into());

    let next_frame = Closure::wrap(Box::new(move || {
        ctx.clear_rect(0.0, 0.0, width, height);
        ctx.set_fill_style(&JsValue::from_str("#88f"));
        draw_nest(&ctx, width / 2.0, height / 2.0);
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
        }
    }) as Box<dyn FnMut()>);

    window.set_interval_with_callback_and_timeout_and_arguments_0(
        next_frame.as_ref().unchecked_ref(),
        FRAME_TIME,
    )?;
    next_frame.forget();

    Ok(())
}
