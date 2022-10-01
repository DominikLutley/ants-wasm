use crate::consts::{GRID_SIZE, PI, WALK_SPEED};
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

pub fn coords_to_pos(coords: (usize, usize)) -> (f32, f32) {
    (coords.0 as f32 * GRID_SIZE, coords.1 as f32 * GRID_SIZE)
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

pub fn clear(gl: &WebGl2RenderingContext) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
}

pub fn draw_points(gl: &WebGl2RenderingContext, vert_count: i32) {
    gl.draw_arrays(WebGl2RenderingContext::POINTS, 0, vert_count);
}

// pub fn draw_triangles(gl: &WebGl2RenderingContext, vert_count: i32) {
//     gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vert_count);
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

pub fn initialize_ants(
    width: f32,
    height: f32,
    ant_count: usize,
) -> (Vec<f32>, Vec<f32>, Vec<bool>) {
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

    let has_food = vec![false; ant_count];

    (ants, dirs, has_food)
}

pub fn initialize_grid(width: f32, height: f32, nest_coords: (usize, usize)) -> Vec<f32> {
    // console::log_1(&JsValue::from(nest_location.1));
    let mut grid = Vec::new();
    let nest_coord_list = [
        nest_coords,
        (nest_coords.0, nest_coords.1 - 1),
        (nest_coords.0 - 1, nest_coords.1),
        (nest_coords.0 - 1, nest_coords.1 - 1),
    ];
    let wall_coord_list = [(50, 10), (50, 11), (50, 12)];
    let food_coord_list = [(25, 25), (26, 25), (27, 25)];
    for i in 0..((2.0 * width / GRID_SIZE * height / GRID_SIZE) as usize) {
        let coords = (
            i % (width / GRID_SIZE) as usize,
            i / (width / GRID_SIZE) as usize,
        );
        let items = match coords {
            p if nest_coord_list.contains(&p) => (1.0, 1.0),
            p if food_coord_list.contains(&p) => (2.0, 1.0),
            p if wall_coord_list.contains(&p) => (3.0, 1.0),
            _ => (0.0, 0.0),
        };
        grid.push(items.0);
        grid.push(items.1);
    }
    grid
}

pub fn next_ant_position(pos: (f32, f32), dir: f32) -> (f32, f32) {
    (
        pos.0 + dir.cos() * WALK_SPEED,
        pos.1 + dir.sin() * WALK_SPEED,
    )
}
