use crate::{
    consts::{ANT_COUNT, ANT_SIZE, NEST_HONING_STRENGTH, PI, WANDER_COEFFICIENT},
    functions::{compile_shader, draw_points, initialize_ants, link_program, next_ant_position},
    grid::{GridRenderer, GridResource},
};
use rand::prelude::*;
use rand_xoshiro::Xoshiro256Plus;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::WebGlBuffer;
use web_sys::WebGlVertexArrayObject;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

// #[derive(Clone, Debug)]
// pub struct Ant {
//     pub x: f32,
//     pub y: f32,
//     pub dir: f32,
//     pub has_food: bool,
// }

pub struct AntRenderer {
    ants: Vec<f32>,
    dirs: Vec<f32>,
    has_food: Vec<bool>,
    program: WebGlProgram,
    width: f32,
    height: f32,
    // a_position_location: i32,
    u_resolution_location: Option<WebGlUniformLocation>,
    u_ant_size_location: Option<WebGlUniformLocation>,
    u_color_location: Option<WebGlUniformLocation>,
    position_buffer: WebGlBuffer,
    positions_array_buf_view: js_sys::Float32Array,
    vertex_count: i32,
    vao: WebGlVertexArrayObject,
    pub next_location: u32,
    pub memory_buffer: JsValue,
}

impl AntRenderer {
    pub fn new(gl: &WebGl2RenderingContext, width: f32, height: f32) -> Result<Self, JsValue> {
        let vertex_shader = compile_shader(
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

        let fragment_shader = compile_shader(
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

        let (ants, dirs, has_food) = initialize_ants(width, height, ANT_COUNT);

        let program = link_program(&gl, &vertex_shader, &fragment_shader)?;

        let a_position_location = gl.get_attrib_location(&program, "a_position");

        let u_resolution_location = gl.get_uniform_location(&program, "u_resolution");
        let u_ant_size_location = gl.get_uniform_location(&program, "u_ant_size");
        let u_color_location = gl.get_uniform_location(&program, "u_color");

        let position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));

        let location: u32 = ants.as_ptr() as u32 / 4;
        let next_location = location + ants.len() as u32;

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<js_sys::WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let positions_array_buf_view =
            js_sys::Float32Array::new(&memory_buffer).subarray(location, next_location);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        let vao = gl
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        gl.bind_vertex_array(Some(&vao));

        gl.vertex_attrib_pointer_with_i32(
            a_position_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(a_position_location as u32);

        let vertex_count = (ants.len() / 2) as i32;

        Ok(AntRenderer {
            ants,
            dirs,
            has_food,
            width,
            height,
            program,
            // a_position_location,
            u_resolution_location,
            u_ant_size_location,
            u_color_location,
            position_buffer,
            positions_array_buf_view,
            vertex_count,
            vao,
            next_location,
            memory_buffer,
        })
    }

    pub fn render(
        &mut self,
        gl: &WebGl2RenderingContext,
        rng: &mut Xoshiro256Plus,
        grid_renderer: &GridRenderer,
    ) {
        gl.use_program(Some(&self.program));
        gl.bind_vertex_array(Some(&self.vao));

        gl.uniform4fv_with_f32_array(self.u_color_location.as_ref(), &[1.0, 1.0, 1.0, 1.0]);
        gl.uniform2f(self.u_resolution_location.as_ref(), self.width, self.height);
        gl.uniform1f(self.u_ant_size_location.as_ref(), ANT_SIZE);

        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.position_buffer),
        );

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &self.positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        draw_points(&gl, self.vertex_count);

        for idx in (0..self.ants.len()).step_by(2) {
            let (part1, part2) = self.ants.split_at_mut(idx + 1);
            let x = part1.last_mut().expect("Error indexing vector");
            let y = part2.first_mut().expect("Error indexing vector");
            let dir = &mut self.dirs[idx / 2];
            if *dir >= PI {
                *dir -= 2.0 * PI;
            }
            if *dir < -1.0 * PI {
                *dir += 2.0 * PI;
            }
            let mut next_dir = dir.clone();
            if self.has_food[idx / 2] == true {
                let dir_diff = grid_renderer.dir_to_nest((*x, *y)) - next_dir;
                next_dir = next_dir + dir_diff * NEST_HONING_STRENGTH;
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
                match grid_renderer.get_resource_at_position(next_pos) {
                    GridResource::Blank => {
                        break;
                    }
                    GridResource::Food => {
                        if self.has_food[idx / 2] {
                            break;
                        }
                        self.has_food[idx / 2] = true;
                        next_dir = *dir + PI;
                        next_pos = next_ant_position((*x, *y), next_dir);
                        if grid_renderer.get_resource_at_position(next_pos) != GridResource::Blank {
                            next_pos = next_ant_position((*x, *y), *dir);
                            next_dir = *dir;
                        }
                        break;
                    }
                    GridResource::Nest => {
                        self.has_food[idx / 2] = false;
                        next_dir = *dir + PI;
                        next_pos = next_ant_position((*x, *y), next_dir);
                        if grid_renderer.get_resource_at_position(next_pos) != GridResource::Blank {
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
    }
}
