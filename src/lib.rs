mod functions;
use functions::*;
use web_sys::console;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
mod consts;
use consts::*;
use easybench_wasm::bench;
use rand::prelude::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use web_sys::WebGl2RenderingContext;

#[derive(Clone, Debug)]
pub struct Ant {
    pub x: f32,
    pub y: f32,
    pub dir: f32,
    pub has_food: bool,
}

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    let window = window();
    let (width, height, ctx) = get_canvas_dimensions_and_context(&window);
    let (mut ants, mut dirs) = initialize_ants(width, height, ANT_COUNT);
    // let mut pheromone_map: Vec<u8> = vec![0; (width * height) as usize];
    let mut rng = Xoshiro256Plus::seed_from_u64(0);
    // console::log_1(&format!("Normal RNG: {}", bench(|| {
    //     // bench
    // })).into());
    // console::log_1(&format!("ff8888{:02x}", 10).into());

    let vertex_shader = compile_shader(
        &ctx,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es

        in vec2 a_position;
        uniform vec2 u_resolution;

        void main() {
            vec2 clip_space = 2.0 * a_position / u_resolution - 1.0;
            gl_Position = vec4(clip_space * vec2(1, -1), 0, 1);
            gl_PointSize = 2.0;
        }
        "##,
    ).expect("Error creating vertex shader");

    let fragment_shader = compile_shader(
        &ctx,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
        
        precision highp float;
        uniform vec4 u_color;
        out vec4 out_color;

        void main() {
            out_color = u_color;
        }
        "##,
    ).expect("Error creating fragment shader");

    let program = link_program(&ctx, &vertex_shader, &fragment_shader)?;
    ctx.use_program(Some(&program));

    let a_position_location = ctx.get_attrib_location(&program, "a_position");

    let u_resolution_location = ctx.get_uniform_location(&program, "u_resolution");
    let u_color_location = ctx.get_uniform_location(&program, "u_color");
    ctx.uniform4fv_with_f32_array(u_color_location.as_ref(), &[1.0, 1.0, 1.0, 1.0]);
    ctx.uniform2f(u_resolution_location.as_ref(), width, height);

    let position_buffer = ctx.create_buffer().ok_or("Failed to create buffer")?;
    ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));

    let positions_array_buf_view = {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<js_sys::WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let location: u32 = ants.as_ptr() as u32 / 4;
        js_sys::Float32Array::new(&memory_buffer).subarray(location, location + ants.len() as u32)
    };

    ctx.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &positions_array_buf_view,
        WebGl2RenderingContext::STATIC_DRAW
    );

    let vao = ctx.create_vertex_array().ok_or("Could not create vertex array object")?;
    ctx.bind_vertex_array(Some(&vao));

    ctx.vertex_attrib_pointer_with_i32(
        a_position_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    ctx.enable_vertex_attrib_array(a_position_location as u32);

    ctx.bind_vertex_array(Some(&vao));

    let vertex_count = (ants.len() / 2) as i32;

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        // let positions_array_buf_view = js_sys::Float32Array::view(&ants);
        // let positions_array_buf_view = {
        //     let memory_buffer = wasm_bindgen::memory()
        //         .dyn_into::<js_sys::WebAssembly::Memory>()
        //         .unwrap()
        //         .buffer();
        //     let location: u32 = ants.as_ptr() as u32 / 4;
        //     js_sys::Float32Array::new(&memory_buffer).subarray(location, location + ants.len() as u32)
        // };

        ctx.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        draw(&ctx, vertex_count);

        for idx in (0..ants.len()).step_by(2) {
            let (part1, part2) = ants.split_at_mut(idx+1);
            let x = part1.last_mut().expect("Error indexing vector");
            let y = part2.first_mut().expect("Error indexing vector");
            let dir = &mut dirs[idx/2];
            *x += dir.cos() * WALK_SPEED;
            *y += dir.sin() * WALK_SPEED;
            if *x >= width - ANT_RADIUS || *x <= 0.0 + ANT_RADIUS {
                *dir += (PI / 2.0 - *dir) * 2.0;
            } else if *y >= height - ANT_RADIUS || *y <= 0.0 + ANT_RADIUS {
                *dir += 2.0 * *dir;
            }
            *dir += (rng.gen::<f32>() - 0.5) * WANDER_COEFFICIENT;
        }

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

        // ctx.set_fill_style(&JsValue::from_str("#fff"));
        // for ant in &mut ants {
        //     if ant.x >= width - ANT_RADIUS || ant.x <= 0.0 + ANT_RADIUS {
        //         ant.dir += (PI / 2.0 - ant.dir) * 2.0;
        //     } else if ant.y <= 0.0 + ANT_RADIUS || ant.y >= height - ANT_RADIUS {
        //         ant.dir -= 2.0 * ant.dir;
        //     }
        //     ant.dir += (rng.gen::<f32>() - 0.5) * WANDER_COEFFICIENT;
        //     ant.x += ant.dir.cos() * WALK_SPEED;
        //     ant.y += ant.dir.sin() * WALK_SPEED;
        //     draw_ant(&ctx, ant.x, ant.y);
        //     let idx = ant.y as usize * width as usize + ant.x as usize;
        //     if idx > pheromone_map.len() {
        //         continue;
        //     }
        //     pheromone_map[idx] = 254;
        // }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    // window.set_interval_with_callback_and_timeout_and_arguments_0(
    //     next_frame.as_ref().unchecked_ref(),
    //     FRAME_TIME,
    // )?;
    // next_frame.forget();

    Ok(())
}
