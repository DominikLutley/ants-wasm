use crate::{
    consts::{GRID_COLORS, GRID_SIZE},
    functions::{compile_shader, draw_points, initialize_grid, link_program},
};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject,
};

#[derive(PartialEq)]
pub enum GridResource {
    Blank,
    Nest,
    Food,
    Wall,
}

pub struct GridRenderer {
    program: WebGlProgram,
    grid: Vec<f32>,
    nest_coords: (usize, usize),
    width: f32,
    height: f32,
    u_resolution_location: Option<WebGlUniformLocation>,
    u_grid_size_location: Option<WebGlUniformLocation>,
    u_colors_location: Option<WebGlUniformLocation>,
    grid_buffer: WebGlBuffer,
    vao: WebGlVertexArrayObject,
    vertex_count: i32,
    // grid_array_buf_view: js_sys::Float32Array,
}

impl GridRenderer {
    pub fn new(gl: &WebGl2RenderingContext, width: f32, height: f32) -> Result<Self, JsValue> {
        let vertex_shader = compile_shader(
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

        let nest_coords = (
            (width / GRID_SIZE) as usize / 2,
            (height / GRID_SIZE) as usize / 2,
        );
        let grid = initialize_grid(width, height, nest_coords);

        let program = link_program(&gl, &vertex_shader, &fragment_shader)?;

        let a_grid_location = gl.get_attrib_location(&program, "a_grid");

        let u_resolution_location = gl.get_uniform_location(&program, "u_resolution");
        let u_grid_size_location = gl.get_uniform_location(&program, "u_grid_size");
        let u_colors_location = gl.get_uniform_location(&program, "u_colors");

        let grid_buffer = gl.create_buffer().ok_or("Failed to create grid buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&grid_buffer));

        let location: u32 = grid.as_ptr() as u32 / 4;
        let next_location = location + grid.len() as u32;

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<js_sys::WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let grid_array_buf_view =
            js_sys::Float32Array::new(&memory_buffer).subarray(location, next_location);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &grid_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );

        let vao = gl
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        gl.bind_vertex_array(Some(&vao));

        gl.vertex_attrib_pointer_with_i32(
            a_grid_location as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(a_grid_location as u32);

        let vertex_count = (grid.len() / 2) as i32;

        Ok(GridRenderer {
            program,
            width,
            height,
            vao,
            u_colors_location,
            u_resolution_location,
            u_grid_size_location,
            grid_buffer,
            vertex_count,
            // grid_array_buf_view,
            grid,
            nest_coords,
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
            Some(&self.grid_buffer),
        );

        // gl.buffer_data_with_array_buffer_view(
        //     WebGl2RenderingContext::ARRAY_BUFFER,
        //     &self.grid_array_buf_view,
        //     WebGl2RenderingContext::STATIC_DRAW,
        // );

        draw_points(&gl, self.vertex_count);
    }

    pub fn get_resource_at_position(&self, pos: (f32, f32)) -> GridResource {
        if pos.0 < 0.0 || pos.1 < 0.0 || pos.0 > self.width || pos.1 > self.height {
            return GridResource::Wall;
        }
        let idx = self.pos_to_idx(pos);
        match self.grid[idx] as usize {
            1 => GridResource::Nest,
            2 => GridResource::Food,
            3 => GridResource::Wall,
            _ => GridResource::Blank,
        }
    }

    pub fn pos_to_idx(&self, pos: (f32, f32)) -> usize {
        (pos.1 as usize / GRID_SIZE as usize * self.width as usize / GRID_SIZE as usize
            + pos.0 as usize / GRID_SIZE as usize)
            * 2
    }

    pub fn coords_to_pos(&self, coords: (usize, usize)) -> (f32, f32) {
        (coords.0 as f32 * GRID_SIZE, coords.1 as f32 * GRID_SIZE)
    }

    pub fn dir_to_nest(&self, pos: (f32, f32)) -> f32 {
        let nest_pos = self.coords_to_pos(self.nest_coords);
        let d_x = nest_pos.0 - pos.0;
        let d_y = nest_pos.1 - pos.1;
        d_y.atan2(d_x)
    }
}
