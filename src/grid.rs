use crate::{
    consts::{GRID_COLORS, GRID_SIZE},
    functions::{compile_shader, draw_points, link_program},
};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{
    console, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

pub struct GridRenderer {
    program: WebGlProgram,
    width: f32,
    height: f32,
    u_resolution_location: Option<WebGlUniformLocation>,
    u_grid_size_location: Option<WebGlUniformLocation>,
    u_colors_location: Option<WebGlUniformLocation>,
    position_buffer: WebGlBuffer,
    vao: WebGlVertexArrayObject,
    vertex_count: i32,
    positions_array_buf_view: js_sys::Float32Array,
}

impl GridRenderer {
    pub fn new(gl: &WebGl2RenderingContext, width: f32, height: f32) -> Result<Self, JsValue> {
        let vertex_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es

            in vec4 a_position;

            uniform vec2 u_resolution;
            uniform float u_grid_size;
            uniform mat4 u_colors;

            out vec4 v_color;
            const float eps = 0.001;

            void main() {
                vec4 color;
                if (abs(a_position.z - 1.0) < eps) {
                    color = u_colors[1];
                } else if (abs(a_position.z - 2.0) < eps) {
                    color = u_colors[2];
                } else if (abs(a_position.z - 3.0) < eps) {
                    color = u_colors[3];
                } else {
                    color = u_colors[0];
                }
                v_color = vec4(color.rgb * a_position.a, 1.0);

                vec2 pixel_space = u_grid_size * a_position.xy + vec2(u_grid_size / 2.0, u_grid_size / 2.0);
                vec2 clip_space = 2.0 * pixel_space / u_resolution - 1.0;
                gl_Position = vec4(clip_space * vec2(1, -1), 0, 1);

                gl_PointSize = u_grid_size;
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

        let program = link_program(&gl, &vertex_shader, &fragment_shader)?;

        let a_position_location = gl.get_attrib_location(&program, "a_position");

        let u_resolution_location = gl.get_uniform_location(&program, "u_resolution");
        let u_grid_size_location = gl.get_uniform_location(&program, "u_grid_size");
        let u_colors_location = gl.get_uniform_location(&program, "u_colors");

        let center_point = (width / 2.0 / GRID_SIZE, height / 2.0 / GRID_SIZE);
        let positions: Vec<f32> = vec![
            center_point.0,
            center_point.1,
            1.0,
            1.0,
            center_point.0 - 1.0,
            center_point.1,
            1.0,
            1.0,
            center_point.0,
            center_point.1 - 1.0,
            1.0,
            1.0,
            center_point.0 - 1.0,
            center_point.1 - 1.0,
            1.0,
            1.0,
            10.0,
            10.0,
            2.0,
            1.0
        ];
        let position_buffer = gl
            .create_buffer()
            .ok_or("Failed to create position buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));

        let location: u32 = positions.as_ptr() as u32 / 4;
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<js_sys::WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let positions_array_buf_view = js_sys::Float32Array::new(&memory_buffer)
            .subarray(location, location + positions.len() as u32);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        // vertex array
        let vao = gl
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        gl.bind_vertex_array(Some(&vao));

        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&position_buffer));
        // gl.vertex_attrib_i_pointer_with_i32(
        //     a_position_location as u32,
        //     4,
        //     WebGl2RenderingContext::UNSIGNED_SHORT,
        //     0,
        //     0,
        // );
        gl.vertex_attrib_pointer_with_i32(
            a_position_location as u32,
            4,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(a_position_location as u32);

        let vertex_count = (positions.len() / 4) as i32;

        gl.use_program(Some(&program));

        Ok(GridRenderer {
            program,
            width,
            height,
            vao,
            u_colors_location,
            u_resolution_location,
            u_grid_size_location,
            position_buffer,
            vertex_count,
            positions_array_buf_view,
        })
    }

    pub fn render(&self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
        gl.bind_vertex_array(Some(&self.vao));

        gl.uniform_matrix4fv_with_f32_array(self.u_colors_location.as_ref(), false, GRID_COLORS);
        gl.uniform2f(self.u_resolution_location.as_ref(), self.width, self.height);
        gl.uniform1f(self.u_grid_size_location.as_ref(), GRID_SIZE);

        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.position_buffer),
        );

        // gl.buffer_data_with_array_buffer_view(
        //     WebGl2RenderingContext::ARRAY_BUFFER,
        //     &self.positions_array_buf_view,
        //     WebGl2RenderingContext::STATIC_DRAW,
        // );

        draw_points(&gl, self.vertex_count);
    }
}
