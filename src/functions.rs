use crate::consts::PI;
use rand::prelude::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, Window};

pub fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn draw(context: &WebGl2RenderingContext, vert_count: i32) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(WebGl2RenderingContext::POINTS, 0, vert_count);
}

// pub fn draw_nest(ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
//     ctx.begin_path();
//     ctx.ellipse(x, y, NEST_RADIUS, NEST_RADIUS, 0.0, 0.0, 2.0 * PI)
//         .expect("error drawing nest");
//     ctx.fill();
// }
//
// pub fn draw_ant(ctx: &CanvasRenderingContext2d, x: f32, y: f32) {
//     ctx.fill_rect(
//         x - ANT_RADIUS / 2.0,
//         y - ANT_RADIUS / 2.0,
//         ANT_RADIUS,
//         ANT_RADIUS,
//     );
// }

pub fn get_canvas_dimensions_and_context(window: &Window) -> (f32, f32, WebGl2RenderingContext) {
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id("canvas")
        .expect("document should have a #canvas element");
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let width: f32 = canvas.width() as f32;
    let height: f32 = canvas.height() as f32;

    let ctx = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .unwrap();

    (width, height, ctx)
}

pub fn initialize_ants(width: f32, height: f32, ant_count: usize) -> (Vec<f32>, Vec<f32>) {
    let mut rng = Xoshiro256Plus::seed_from_u64(0);

    let mut ants = Vec::new();
    for _i in 0..ant_count {
        ants.push(width / 2.0);
        ants.push(height / 2.0);
    }

    let mut dirs: Vec<f32> = Vec::new();
    for _i in 0..ant_count {
        dirs.push(rng.gen::<f32>() * 2.0 * PI);
    }

    (ants, dirs)
}
