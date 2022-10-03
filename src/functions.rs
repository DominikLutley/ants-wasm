use crate::{
    consts::{GRID_SIZE, PI, WALK_SPEED},
    grid::GridResource,
};
use rand::prelude::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use wasm_bindgen::{prelude::Closure, JsValue};
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, Window, console};

pub fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn calc_dir(pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
    let d_x = pos2.0 - pos1.0;
    let d_y = pos2.1 - pos1.1;
    d_y.atan2(d_x)
}

pub fn calc_dist(pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
    ((pos2.0 - pos1.0).powi(2) + (pos2.1 - pos1.1).powi(2)).sqrt()
}

pub fn coords_to_pos(coords: (usize, usize)) -> (f32, f32) {
    (coords.0 as f32 * GRID_SIZE, coords.1 as f32 * GRID_SIZE)
}

pub fn dir_to_nest(pos: (f32, f32), nest_coords: (usize, usize)) -> f32 {
    let nest_pos = coords_to_pos(nest_coords);
    calc_dir(pos, nest_pos)
}

pub fn pos_to_idx(pos: (f32, f32), width: f32) -> usize {
    (pos.1 as usize / GRID_SIZE as usize * width as usize / GRID_SIZE as usize
        + pos.0 as usize / GRID_SIZE as usize)
        * 2
}

pub fn get_resource_at_position(
    grid: &Vec<f32>,
    width: f32,
    height: f32,
    pos: (f32, f32),
) -> GridResource {
    if pos.0 < 0.0 || pos.1 < 0.0 || pos.0 > width || pos.1 > height {
        return GridResource::Wall;
    }
    let idx = pos_to_idx(pos, width);
    match grid[idx] as usize {
        1 => GridResource::Nest,
        2 => GridResource::Food,
        3 => GridResource::Wall,
        _ => GridResource::Blank,
    }
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

pub fn add_pheromone(pheromones: &mut Vec<f32>, pheromone_dirs: &mut Vec<f32>, pos: (f32, f32), dir: f32) {
    let result = pheromones.iter().step_by(3).position(|&x| x <= 0.0);
    match result {
        Some(idx) => {
            pheromones[idx * 3] = pos.0;
            pheromones[idx * 3 + 1] = pos.1;
            pheromones[idx * 3 + 2] = 1.0;
            pheromone_dirs[idx / 3] = dir;
        }
        None => {
            console::log_1(&JsValue::from("Not enough pheromones"));
        },
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
