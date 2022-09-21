use rand::prelude::*;
use rand_xoshiro::Xoshiro256Plus;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::WebGlBuffer;
use web_sys::WebGlVertexArrayObject;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};
use crate::{functions::{compile_shader, link_program, initialize_ants, draw_points}, consts::{ANT_COUNT, WALK_SPEED, ANT_SIZE, WANDER_COEFFICIENT, PI}};

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
        let (ants, dirs) = initialize_ants(width, height, ANT_COUNT);

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

        let positions_array_buf_view = js_sys::Float32Array::new(&memory_buffer)
                .subarray(location, next_location);

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
            memory_buffer
        })
    }

    pub fn render(&mut self, gl: &WebGl2RenderingContext, rng: &mut Xoshiro256Plus) {
        gl.use_program(Some(&self.program));
        gl.bind_vertex_array(Some(&self.vao));

        gl.uniform4fv_with_f32_array(self.u_color_location.as_ref(), &[1.0, 1.0, 1.0, 1.0]);
        gl.uniform2f(self.u_resolution_location.as_ref(), self.width, self.height);
        gl.uniform1f(self.u_ant_size_location.as_ref(), ANT_SIZE);

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.position_buffer));

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &self.positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        draw_points(&gl, self.vertex_count);

        for idx in (0..self.ants.len()).step_by(2) {
            let (part1, part2) = self.ants.split_at_mut(idx+1);
            let x = part1.last_mut().expect("Error indexing vector");
            let y = part2.first_mut().expect("Error indexing vector");
            let dir = &mut self.dirs[idx/2];
            *x += dir.cos() * WALK_SPEED;
            *y += dir.sin() * WALK_SPEED;
            if *x >= self.width - ANT_SIZE || *x <= 0.0 + ANT_SIZE {
                *dir += (PI / 2.0 - *dir) * 2.0;
            } else if *y >= self.height - ANT_SIZE || *y <= 0.0 + ANT_SIZE {
                *dir += 2.0 * *dir;
            }
            *dir += (rng.gen::<f32>() - 0.5) * WANDER_COEFFICIENT;
        }
    }
}
