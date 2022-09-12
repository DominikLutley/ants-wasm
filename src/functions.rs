use crate::consts::{PI, ANT_RADIUS, NEST_RADIUS};
use crate::Ant;
use rand::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Window};

pub fn draw_nest(ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
    ctx.begin_path();
    ctx.ellipse(x, y, NEST_RADIUS, NEST_RADIUS, 0.0, 0.0, 2.0 * PI)
        .expect("error drawing nest");
    ctx.fill();
}

pub fn draw_ant(ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
    ctx.fill_rect(
        x - ANT_RADIUS / 2.0,
        y - ANT_RADIUS / 2.0,
        ANT_RADIUS,
        ANT_RADIUS,
    );
}

pub fn get_canvas_dimensions_and_context(window: &Window) -> (f64, f64, CanvasRenderingContext2d) {
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

    (width, height, ctx)
}

pub fn initialize_ants(width: f64, height: f64, ant_count: usize) -> Vec<Ant> {
    let mut rng = rand::thread_rng();

    let mut ants = vec![
        Ant {
            x: width / 2.0,
            y: height / 2.0,
            dir: 0.0,
        };
        ant_count
    ];

    for ant in &mut ants {
        ant.dir = rng.gen::<f64>() * 2.0 * PI;
    }

    ants
}
