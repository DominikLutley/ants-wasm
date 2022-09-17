use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject, WebGlBuffer, console};
use wasm_bindgen::JsCast;
use crate::{functions::{compile_shader, link_program, draw_points}, consts::FOOD_SIZE};

pub struct FoodRenderer {
    program: WebGlProgram,
    width: f32,
    height: f32,
    u_resolution_location: Option<WebGlUniformLocation>,
    u_food_size_location: Option<WebGlUniformLocation>,
    u_color_location: Option<WebGlUniformLocation>,
    position_buffer: WebGlBuffer,
    vao: WebGlVertexArrayObject,
    vertex_count: i32,
}

impl FoodRenderer {
    pub fn new(gl: &WebGl2RenderingContext, width: f32, height: f32) -> Result<Self, JsValue> {
        let vertex_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es

            in vec2 a_position;
            uniform vec2 u_resolution;
            uniform float u_food_size;

            void main() {
                vec2 clip_space = 2.0 * a_position / u_resolution - 1.0;
                gl_Position = vec4(clip_space * vec2(1, -1), 0, 1);
                gl_PointSize = u_food_size;
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
        let u_food_size_location = gl.get_uniform_location(&program, "u_food_size");
        let u_color_location = gl.get_uniform_location(&program, "u_color");

        let position_buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));
        console::log_2(&JsValue::from_f64(width as f64), &JsValue::from_f64(height as f64));

        let vertices: [f32; 2] = [
            100.0, 500.0
        ];

        let positions_array_buf_view = {
            let memory_buffer = wasm_bindgen::memory()
                .dyn_into::<js_sys::WebAssembly::Memory>()
                .unwrap()
                .buffer();
            let location: u32 = vertices.as_ptr() as u32 / 4;
            js_sys::Float32Array::new(&memory_buffer)
                .subarray(location, location + vertices.len() as u32)
        };

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

        let vertex_count = (vertices.len() / 2) as i32;

        Ok(FoodRenderer {
            program,
            width,
            height,
            vao,
            u_color_location,
            u_resolution_location,
            u_food_size_location,
            position_buffer,
            vertex_count
        })
    }

    pub fn render(&self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
        gl.bind_vertex_array(Some(&self.vao));

        gl.uniform4fv_with_f32_array(self.u_color_location.as_ref(), &[0.5, 1.0, 0.5, 1.0]);
        gl.uniform2f(self.u_resolution_location.as_ref(), self.width, self.height);
        gl.uniform1f(self.u_food_size_location.as_ref(), FOOD_SIZE);

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.position_buffer));

        draw_points(&gl, self.vertex_count);

    }
}
