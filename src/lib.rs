mod functions;
use consts::{
    ANT_COLOR, ANT_COUNT, ANT_PHEROMONE_TIMER, ANT_SIZE, ANT_VIEW_ARC, ANT_VIEW_RADIUS,
    GRID_COLORS, GRID_SIZE, NEST_HONING_STRENGTH, PHEROMONE_COLOR, PHEROMONE_SIZE, PI,
    WANDER_COEFFICIENT,
};
use functions::*;
use pheromones::PheromoneRenderer;
use web_sys::{WebGl2RenderingContext, console};
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
use rand::prelude::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    let window = window();
    let (width, height, gl) = get_canvas_dimensions_and_context(&window);
    let mut rng = Xoshiro256Plus::seed_from_u64(0);

    let (mut ants, mut dirs, mut has_food) = initialize_ants(width, height, ANT_COUNT);

    let nest_coords = (
        (width / GRID_SIZE) as usize / 2,
        (height / GRID_SIZE) as usize / 2,
    );

    let grid = initialize_grid(width, height, nest_coords);

    let mut pheromones: Vec<f32> = vec![-1.0; ANT_COUNT * 3];
    let mut pheromone_dirs: Vec<f32> = vec![0.0; pheromones.len() / 3];

    // let mut pheromone_map: Vec<u8> = vec![0; (width * height) as usize];
    // console::log_1(&format!("Normal RNG: {}", bench(|| {
    //     // bench
    // })).into());

    // let memory_buffer = wasm_bindgen::memory()
    //     .dyn_into::<js_sys::WebAssembly::Memory>()
    //     .unwrap()
    //     .buffer();

    let grid_vertex_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es

        in vec2 a_grid;

        uniform vec2 u_resolution;
        uniform float u_grid_size;
        uniform mat4 u_colors;

        out vec4 v_color;
        const float eps = 0.001;

        void main() {
            vec4 color;
            if (abs(a_grid.x - 1.0) < eps) {
                color = u_colors[1];
            } else if (abs(a_grid.x - 2.0) < eps) {
                color = u_colors[2];
            } else if (abs(a_grid.x - 3.0) < eps) {
                color = u_colors[3];
            } else {
                color = u_colors[0];
            }
            v_color = vec4(color.rgb * a_grid.y, 1.0);

            vec2 coords = vec2(gl_VertexID % 120, gl_VertexID / 120);
            vec2 pixel_space = u_grid_size * coords + vec2(u_grid_size / 2.0, u_grid_size / 2.0);
            vec2 clip_space = 2.0 * pixel_space / u_resolution - 1.0;
            gl_Position = vec4(clip_space * vec2(1, -1), 0, 1);

            gl_PointSize = u_grid_size;
        }
        "##,
    )
    .expect("Error creating vertex shader");

    let grid_fragment_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
        
        precision highp float;

        in vec4 v_color;

        out vec4 out_color;

        void main() {
            out_color = v_color;
        }
        "##,
    )
    .expect("Error creating fragment shader");

    let grid_program = link_program(&gl, &grid_vertex_shader, &grid_fragment_shader)?;

    let a_grid_location = gl.get_attrib_location(&grid_program, "a_grid");

    let u_grid_resolution_location = gl.get_uniform_location(&grid_program, "u_resolution");
    let u_grid_size_location = gl.get_uniform_location(&grid_program, "u_grid_size");
    let u_grid_colors_location = gl.get_uniform_location(&grid_program, "u_colors");

    let grid_buffer = gl.create_buffer().ok_or("Failed to create grid buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&grid_buffer));

    let grid_location: u32 = grid.as_ptr() as u32 / 4;
    let grid_next_location = grid_location + grid.len() as u32;

    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<js_sys::WebAssembly::Memory>()
        .unwrap()
        .buffer();

    let grid_array_buf_view =
        js_sys::Float32Array::new(&memory_buffer).subarray(grid_location, grid_next_location);

    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &grid_array_buf_view,
        WebGl2RenderingContext::DYNAMIC_DRAW,
    );

    let grid_vao = gl
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    gl.bind_vertex_array(Some(&grid_vao));

    gl.vertex_attrib_pointer_with_i32(
        a_grid_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(a_grid_location as u32);

    let grid_vertex_count = (grid.len() / 2) as i32;

    let ants_vertex_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es

        in vec2 a_position;
        uniform vec2 u_resolution;
        uniform float u_ant_size;

        void main() {
            vec2 clip_space = 2.0 * a_position / u_resolution - 1.0;
            gl_Position = vec4(clip_space * vec2(1, -1), 0, 1);
            gl_PointSize = u_ant_size;
        }
        "##,
    )
    .expect("Error creating vertex shader");

    let ants_fragment_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
        
        precision highp float;
        uniform vec4 u_color;
        out vec4 out_color;

        void main() {
            out_color = u_color;
        }
        "##,
    )
    .expect("Error creating fragment shader");

    let ants_program = link_program(&gl, &ants_vertex_shader, &ants_fragment_shader)?;

    let a_ants_position_location = gl.get_attrib_location(&ants_program, "a_position");

    let u_ants_resolution_location = gl.get_uniform_location(&ants_program, "u_resolution");
    let u_ant_size_location = gl.get_uniform_location(&ants_program, "u_ant_size");
    let u_ant_color_location = gl.get_uniform_location(&ants_program, "u_color");

    let ants_position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(
        WebGl2RenderingContext::ARRAY_BUFFER,
        Some(&ants_position_buffer),
    );

    let ants_location: u32 = ants.as_ptr() as u32 / 4;
    let ants_next_location = ants_location + ants.len() as u32;

    // let memory_buffer = wasm_bindgen::memory()
    //     .dyn_into::<js_sys::WebAssembly::Memory>()
    //     .unwrap()
    //     .buffer();

    let ant_positions_array_buf_view =
        js_sys::Float32Array::new(&memory_buffer).subarray(ants_location, ants_next_location);

    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &ant_positions_array_buf_view,
        WebGl2RenderingContext::DYNAMIC_DRAW,
    );

    let ants_vao = gl
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    gl.bind_vertex_array(Some(&ants_vao));

    gl.vertex_attrib_pointer_with_i32(
        a_ants_position_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(a_ants_position_location as u32);

    let ants_vertex_count = (ants.len() / 2) as i32;

    let mut pheromone_timer = ANT_PHEROMONE_TIMER;

    let pheromone_vertex_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es

            in vec3 a_position;
            uniform vec2 u_resolution;
            uniform float u_pheromone_size;
            uniform vec4 u_color;
            out vec4 v_color;

            void main() {
                vec2 clip_space = 2.0 * a_position.xy / u_resolution - 1.0;
                gl_Position = vec4(clip_space * vec2(1, -1), 0, 1);
                gl_PointSize = u_pheromone_size;
                v_color = vec4(u_color.rgb * a_position.z, 1.0);
            }
            "##,
    )
    .expect("Error creating vertex shader");

    let pheromone_fragment_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
            
            precision highp float;
            
            in vec4 v_color;
            out vec4 out_color;

            void main() {
                out_color = v_color;
            }
            "##,
    )
    .expect("Error creating fragment shader");

    let pheromone_program =
        link_program(&gl, &pheromone_vertex_shader, &pheromone_fragment_shader)?;

    let a_pheromone_position_location = gl.get_attrib_location(&pheromone_program, "a_position");

    let u_pheromone_resolution_location =
        gl.get_uniform_location(&pheromone_program, "u_resolution");
    let u_pheromone_size_location = gl.get_uniform_location(&pheromone_program, "u_pheromone_size");
    let u_pheromone_color_location = gl.get_uniform_location(&pheromone_program, "u_color");

    let pheromone_position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
    gl.bind_buffer(
        WebGl2RenderingContext::ARRAY_BUFFER,
        Some(&pheromone_position_buffer),
    );

    let pheromone_location: u32 = pheromones.as_ptr() as u32 / 4;
    let pheromone_next_location = pheromone_location + pheromones.len() as u32;

    // let memory_buffer = wasm_bindgen::memory()
    //     .dyn_into::<js_sys::WebAssembly::Memory>()
    //     .unwrap()
    //     .buffer();

    let pheromone_positions_array_buf_view = js_sys::Float32Array::new(&memory_buffer)
        .subarray(pheromone_location, pheromone_next_location);

    gl.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &pheromone_positions_array_buf_view,
        WebGl2RenderingContext::DYNAMIC_DRAW,
    );

    let pheromone_vao = gl
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    gl.bind_vertex_array(Some(&pheromone_vao));

    gl.vertex_attrib_pointer_with_i32(
        a_pheromone_position_location as u32,
        3,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(a_pheromone_position_location as u32);

    let pheromone_vertex_count = (pheromones.len() / 3) as i32;

    // let grid_renderer = GridRenderer::new(&gl, width, height)
    //     .expect("Error initializing grid renderer");
    // let mut ant_renderer =
    //     AntRenderer::new(&gl, width, height).expect("Error initializing ant renderer");
    // let mut pheromone_renderer =
    //     PheromoneRenderer::new(&gl, width, height).expect("Error initializing pheromone renderer");

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        clear(&gl);
        gl.use_program(Some(&grid_program));
        gl.bind_vertex_array(Some(&grid_vao));

        gl.uniform_matrix4fv_with_f32_array(u_grid_colors_location.as_ref(), false, GRID_COLORS);
        gl.uniform2f(u_grid_resolution_location.as_ref(), width, height);
        gl.uniform1f(u_grid_size_location.as_ref(), GRID_SIZE);

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&grid_buffer));

        // gl.buffer_data_with_array_buffer_view(
        //     WebGl2RenderingContext::ARRAY_BUFFER,
        //     &self.grid_array_buf_view,
        //     WebGl2RenderingContext::STATIC_DRAW,
        // );

        draw_points(&gl, grid_vertex_count);

        gl.use_program(Some(&ants_program));
        gl.bind_vertex_array(Some(&ants_vao));

        gl.uniform4fv_with_f32_array(u_ant_color_location.as_ref(), ANT_COLOR);
        gl.uniform2f(u_ants_resolution_location.as_ref(), width, height);
        gl.uniform1f(u_ant_size_location.as_ref(), ANT_SIZE);

        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&ants_position_buffer),
        );

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &ant_positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        draw_points(&gl, ants_vertex_count);

        pheromone_timer -= 1;

        for idx in (0..ants.len()).step_by(2) {
            let (part1, part2) = ants.split_at_mut(idx + 1);
            let x = part1.last_mut().expect("Error indexing vector");
            let y = part2.first_mut().expect("Error indexing vector");
            let dir = &mut dirs[idx / 2];
            if *dir >= PI {
                *dir -= 2.0 * PI;
            }
            if *dir < -1.0 * PI {
                *dir += 2.0 * PI;
            }
            let mut next_dir = dir.clone();
            if has_food[idx / 2] == true {
                let dir_diff = dir_to_nest((*x, *y), nest_coords) - next_dir;
                next_dir = next_dir + dir_diff * NEST_HONING_STRENGTH;
                if pheromone_timer == 0 {
                    add_pheromone(&mut pheromones, &mut pheromone_dirs, (*x, *y), next_dir + PI);
                }
            } else {
                // let mut weighted_dirs: Vec<(f32, f32)> = Vec::new();
                // let mut weight_sum: f32 = 0.0;
                let mut max_strength = 0.0;
                for idx in (0..pheromones.len()).step_by(3) {
                    let pher_x = pheromones[idx];
                    let pher_y = pheromones[idx + 1];
                    let pher_dist = calc_dist((*x, *y), (pher_x, pher_y));
                    // let pher_loc = calc_dir((*x, *y), (pher_x, pher_y));
                    let pher_dir = pheromone_dirs[idx / 3];
                    if pher_dist <= ANT_VIEW_RADIUS {
                        let pher_s = pheromones[idx + 2];
                        if pher_s > max_strength {
                            max_strength = pher_s;
                            next_dir = pher_dir;
                            console::log_1(&JsValue::from(next_dir));
                        }
                        // weighted_dirs.push((pheromone_dirs[idx / 3], pher_s));
                        // weight_sum += pher_s;
                    }
                }
                // match weighted_dirs.len() {
                //     0 => (),
                //     _ => {
                //         let pheromone_dir = weighted_dirs
                //             .iter()
                //             .fold(0.0, |accum, val| accum + val.0 * val.1)
                //             / weight_sum;
                //         next_dir = pheromone_dir;
                //     }
                // }
            }
            next_dir += (rng.gen::<f32>() - 0.5) * WANDER_COEFFICIENT;
            let mut next_pos = (*x, *y);
            for i in 0..4 {
                next_dir = match i {
                    0 => next_dir,
                    1 => *dir - 2.0 * *dir,
                    2 => *dir + (PI / 2.0 - *dir) * 2.0,
                    _ => *dir + PI,
                };
                next_pos = next_ant_position((*x, *y), next_dir);
                match get_resource_at_position(&grid, width, height, next_pos) {
                    GridResource::Blank => {
                        break;
                    }
                    GridResource::Food => {
                        if has_food[idx / 2] {
                            break;
                        }
                        has_food[idx / 2] = true;
                        next_dir = *dir + PI;
                        next_pos = next_ant_position((*x, *y), next_dir);
                        if get_resource_at_position(&grid, width, height, next_pos)
                            != GridResource::Blank
                        {
                            next_pos = next_ant_position((*x, *y), *dir);
                            next_dir = *dir;
                        }
                        break;
                    }
                    GridResource::Nest => {
                        has_food[idx / 2] = false;
                        next_dir = *dir + PI;
                        next_pos = next_ant_position((*x, *y), next_dir);
                        if get_resource_at_position(&grid, width, height, next_pos)
                            != GridResource::Blank
                        {
                            next_pos = next_ant_position((*x, *y), *dir);
                            next_dir = *dir;
                        }
                        break;
                    }
                    GridResource::Wall => {
                        continue;
                    }
                }
            }
            (*x, *y) = next_pos;
            *dir = next_dir;
        }

        if pheromone_timer <= 0 {
            pheromone_timer = ANT_PHEROMONE_TIMER;
        }

        gl.use_program(Some(&pheromone_program));
        gl.bind_vertex_array(Some(&pheromone_vao));

        gl.uniform4fv_with_f32_array(u_pheromone_color_location.as_ref(), PHEROMONE_COLOR);
        gl.uniform2f(u_pheromone_resolution_location.as_ref(), width, height);
        gl.uniform1f(u_pheromone_size_location.as_ref(), PHEROMONE_SIZE);

        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&pheromone_position_buffer),
        );

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &pheromone_positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        draw_points(&gl, pheromone_vertex_count);

        for idx in (0..pheromones.len()).step_by(3) {
            let (part1, part2) = pheromones.split_at_mut(idx + 1);
            let x = part1.last_mut().expect("Error indexing vector");
            let (part3, part4) = part2.split_at_mut(1);
            let y = part3.last_mut().expect("Error indexing vector");
            let str = part4.first_mut().expect("Error indexing vector");
            *str -= 0.003;
            if *str <= 0.0 {
                *x = -1.0;
                *y = -1.0;
                *str = -1.0;
            }
        }

        // grid_renderer.render(&gl);
        // ant_renderer.render(&gl, &mut rng, &grid_renderer, &mut pheromone_renderer);
        // pheromone_renderer.render(&gl);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
