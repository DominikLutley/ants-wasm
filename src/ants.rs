use rand::prelude::*;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256Plus;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};
use wasm_bindgen::JsCast;
use crate::{functions::{compile_shader, link_program, initialize_ants, draw}, consts::{ANT_COUNT, WALK_SPEED, ANT_RADIUS, WANDER_COEFFICIENT, PI}};

#[derive(Clone, Debug)]
pub struct Ant {
    pub x: f32,
    pub y: f32,
    pub dir: f32,
    pub has_food: bool,
}

pub struct AntRenderer {
    ants: Vec<f32>,
    dirs: Vec<f32>,
    program: WebGlProgram,
    width: f32,
    height: f32,
    a_position_location: i32,
    u_resolution_location: Option<WebGlUniformLocation>,
    u_color_location: Option<WebGlUniformLocation>,
    positions_array_buf_view: js_sys::Float32Array,
    vao: WebGlVertexArrayObject,
    vertex_count: i32,
}

impl AntRenderer {
    pub fn new(gl: &WebGl2RenderingContext, width: f32, height: f32) -> Result<Self, JsValue> {
        let (mut ants, mut dirs) = initialize_ants(width, height, ANT_COUNT);

        let vertex_shader = compile_shader(
            &gl,
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
        let u_color_location = gl.get_uniform_location(&program, "u_color");

        let position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));

        let positions_array_buf_view = {
            let memory_buffer = wasm_bindgen::memory()
                .dyn_into::<js_sys::WebAssembly::Memory>()
                .unwrap()
                .buffer();
            let location: u32 = ants.as_ptr() as u32 / 4;
            js_sys::Float32Array::new(&memory_buffer)
                .subarray(location, location + ants.len() as u32)
        };

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
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
            a_position_location,
            u_resolution_location,
            u_color_location,
            positions_array_buf_view,
            vao,
            vertex_count,
        })
    }

    pub fn render(&mut self, gl: &WebGl2RenderingContext) {
        let mut rng = Xoshiro256Plus::seed_from_u64(0);

        gl.use_program(Some(&self.program));

        gl.bind_vertex_array(Some(&self.vao));

        gl.uniform4fv_with_f32_array(self.u_color_location.as_ref(), &[1.0, 1.0, 1.0, 1.0]);
        gl.uniform2f(self.u_resolution_location.as_ref(), self.width, self.height);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &self.positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW
        );

        draw(&gl, self.vertex_count);

        for idx in (0..self.ants.len()).step_by(2) {
            let (part1, part2) = self.ants.split_at_mut(idx+1);
            let x = part1.last_mut().expect("Error indexing vector");
            let y = part2.first_mut().expect("Error indexing vector");
            let dir = &mut self.dirs[idx/2];
            *x += dir.cos() * WALK_SPEED;
            *y += dir.sin() * WALK_SPEED;
            if *x >= self.width - ANT_RADIUS || *x <= 0.0 + ANT_RADIUS {
                *dir += (PI / 2.0 - *dir) * 2.0;
            } else if *y >= self.height - ANT_RADIUS || *y <= 0.0 + ANT_RADIUS {
                *dir += 2.0 * *dir;
            }
            *dir += (rng.gen::<f32>() - 0.5) * WANDER_COEFFICIENT;
        }
    }
}
