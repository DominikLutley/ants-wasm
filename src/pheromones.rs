use crate::consts::{PHEROMONE_COLOR, PHEROMONE_SIZE, ANT_COUNT};
use crate::functions::{compile_shader, draw_points, link_program};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{WebGlBuffer, console};
use web_sys::WebGlVertexArrayObject;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

pub struct PheromoneRenderer {
    pheromones: Vec<f32>,
    program: WebGlProgram,
    width: f32,
    height: f32,
    // a_position_location: i32,
    u_resolution_location: Option<WebGlUniformLocation>,
    u_pheromone_size_location: Option<WebGlUniformLocation>,
    u_color_location: Option<WebGlUniformLocation>,
    position_buffer: WebGlBuffer,
    positions_array_buf_view: js_sys::Float32Array,
    vertex_count: i32,
    vao: WebGlVertexArrayObject,
}

impl PheromoneRenderer {
    pub fn new(gl: &WebGl2RenderingContext, width: f32, height: f32) -> Result<Self, JsValue> {
        let vertex_shader = compile_shader(
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

        let fragment_shader = compile_shader(
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

        let pheromones: Vec<f32> = vec![-1.0; ANT_COUNT * 3];

        let program = link_program(&gl, &vertex_shader, &fragment_shader)?;

        let a_position_location = gl.get_attrib_location(&program, "a_position");

        let u_resolution_location = gl.get_uniform_location(&program, "u_resolution");
        let u_pheromone_size_location = gl.get_uniform_location(&program, "u_pheromone_size");
        let u_color_location = gl.get_uniform_location(&program, "u_color");

        let position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));

        let location: u32 = pheromones.as_ptr() as u32 / 4;
        let next_location = location + pheromones.len() as u32;

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
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(a_position_location as u32);

        let vertex_count = (pheromones.len() / 3) as i32;

        Ok(PheromoneRenderer {
            pheromones,
            width,
            height,
            program,
            // a_position_location,
            u_resolution_location,
            u_pheromone_size_location,
            u_color_location,
            position_buffer,
            positions_array_buf_view,
            vertex_count,
            vao,
        })
    }

    pub fn render(&mut self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
        gl.bind_vertex_array(Some(&self.vao));

        gl.uniform4fv_with_f32_array(self.u_color_location.as_ref(), PHEROMONE_COLOR);
        gl.uniform2f(self.u_resolution_location.as_ref(), self.width, self.height);
        gl.uniform1f(self.u_pheromone_size_location.as_ref(), PHEROMONE_SIZE);

        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.position_buffer),
        );

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &self.positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        
        draw_points(&gl, self.vertex_count);
        
        for idx in (0..self.pheromones.len()).step_by(3) {
            let (part1, part2) = self.pheromones.split_at_mut(idx + 1);
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
    }

    pub fn add_pheromone(&mut self, pos: (f32, f32)) {
        let result = self.pheromones.iter().step_by(3).position(|&x| x <= 0.0);
        match result {
            Some(idx) => {
                self.pheromones[idx * 3] = pos.0;
                self.pheromones[idx * 3 + 1] = pos.1;
                self.pheromones[idx * 3 + 2] = 1.0;
            },
            None => ()
        }
    }
}
