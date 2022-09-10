use rand::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
mod consts;
mod helpers;
use consts::*;
mod ant;
use ant::Ant;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` object exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id("canvas")
        .expect("document should have a #canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let width: f64 = canvas.width().into();
    let height: f64 = canvas.height().into();

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let mut rng = rand::thread_rng();

    let mut ants = vec![
        Ant {
            x: width / 2.0,
            y: height / 2.0,
            dir: 0.0,
        };
        ANT_COUNT
    ];

    for ant in &mut ants {
        ant.dir = rng.gen::<f64>() * 2.0 * PI;
    }

    let next_frame = Closure::wrap(Box::new(move || {
        ctx.clear_rect(0.0, 0.0, width, height);
        ctx.begin_path();
        ctx.ellipse(
            width / 2.0,
            height / 2.0,
            NEST_RADIUS,
            NEST_RADIUS,
            0.0,
            0.0,
            2.0 * PI,
        )
        .expect("error drawing nest");
        ctx.set_fill_style(&JsValue::from_str("#88f"));
        ctx.fill();
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
            ctx.begin_path();
            ctx.ellipse(
                ant.x,
                ant.y,
                ANT_RADIUS,
                ANT_RADIUS / 1.5,
                ant.dir,
                0.0,
                2.0 * PI,
            )
            .expect("error drawing ant");
            ctx.fill();
        }
    }) as Box<dyn FnMut()>);

    window.set_interval_with_callback_and_timeout_and_arguments_0(
        next_frame.as_ref().unchecked_ref(),
        FRAME_TIME,
    )?;
    next_frame.forget();

    Ok(())
}
