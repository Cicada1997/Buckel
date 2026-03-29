use crate::{
    camera::Camera, 
    util::Context,
};

use {
    std::f32::consts::PI,
    glam::{ Mat4, Vec3, vec3 },
    glow::HasContext,
    sdl2::{
        keyboard::Scancode,
        event::Event,
    },
};

macro_rules! limit {
    ($val:ident, $max:expr) => {
        if $val > $max {
            panic!("Value {} exceeds limit of {} with value {}.", stringify!($val), $max, $val);
        }
    };

    ($val:ident, $max:expr, $err_msg:literal) => {
        if $val > $max {
            panic!("Value {} exceeds limit of {} with value {}: {}", stringify!($val), $max, $val, $err_msg);
        }
    };

    ($val:ident, $min:expr, $max:expr) => {
        if $val < $min && $val > $max {
            panic!("Value {} exceeds limit from {} to {} with value {}.", stringify!($val), $min, $max, $val);
        }
    };

    ($val:ident, $min:expr, $max:expr, $err_msg:literal) => {
        if $val < $min && $val > $max {
            panic!("Value {} exceeds limit from {} to {} with value {}: {}", stringify!($val), $min, $max, $val, $err_msg);
        }
    };

}

static FOV: f32 = 90.0;
pub mod camera;
pub mod util {
    use sdl2::{ Sdl, VideoSubsystem, video::Window };

    pub struct Context {
        pub sdl:    Sdl,
        pub video:  VideoSubsystem,
        pub window: Window,
    }
}

static CHUNK_SIZE:   u16 = 16;
static CHUNK_HEIGHT: u16 = 64;
type ByteChunkLayer      = [u16;            CHUNK_SIZE   as usize];
type ByteChunk           = [ByteChunkLayer; CHUNK_HEIGHT as usize];

struct Chunk {
    blocks: ByteChunk,
    mesh:   Vec<f32>,
    dirty:  bool,
}

impl Default for Chunk {
    fn default() -> Self {
        let mut blocks = [[0u16; CHUNK_SIZE as usize]; CHUNK_HEIGHT as usize];
        for y in 0..5 {
            blocks[y].fill(0b_1111_1111_1111_1111);
        }

        let mesh  = Vec::new();
        let dirty = true;

        Self { 
            blocks,
            mesh,
            dirty
        }
    }
}

type VertexArray = Vec<f32>;

impl Chunk {
    pub fn get(&self, x: usize, y: usize, z: usize) -> bool {
        (self.blocks[y][x] >> z) & 1 == 1
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, val: bool) {
        limit!(x, CHUNK_SIZE as usize,   "chunk relative position outside chunk");
        limit!(y, CHUNK_HEIGHT as usize, "chunk relative position outside chunk");
        limit!(z, CHUNK_SIZE as usize,   "chunk relative position outside chunk");

        self.blocks[y][x] = self.blocks[y][x] & (2u16.pow(z as u32));
    }

    pub fn build_mesh(&mut self) {
        self.mesh.clear();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                let mut row = self.blocks[y as usize][x as usize];

                row = row ^ (row >> 1);

                for z in 0..CHUNK_SIZE {
                    if row & 1 == 1 {
                        let fx = x as f32;
                        let fy = y as f32;
                        let fz = z as f32;

                        let right_face = [
                            fx+1.0, fy+0.0, fz+0.0,   fx+1.0, fy+1.0, fz+0.0,   fx+1.0, fy+1.0, fz+1.0, 
                            fx+1.0, fy+1.0, fz+1.0,   fx+1.0, fy+0.0, fz+1.0,   fx+1.0, fy+0.0, fz+0.0,
                        ];
                        self.mesh.extend_from_slice(&right_face);
                    }
                    row = row >> 1;
                }
            }
        }

        self.dirty = false;
    }
}

fn main() {
    let ctx = {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let window = video
            .window("Glow 3D Render", 800, 600)
            .opengl()
            .resizable()
            .build()
            .unwrap();

        Context { sdl, video, window }
    };

    let gl_attr = ctx.video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let _gl_context = ctx.window.gl_create_context().unwrap();
    let gl = unsafe {
        let gl = glow::Context::from_loader_function(|s| ctx.video.gl_get_proc_address(s) as *const _);
        gl.enable(glow::DEPTH_TEST);
        gl
    };

    let chunk = Chunk::default();
    let vertex_count = (chunk.mesh.len() / 3) as i32;

    let (vao, vbo) = unsafe {
        let vao = gl.create_vertex_array().unwrap();
        let vbo = gl.create_buffer().unwrap();

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        // Upload the Vec<f32> slice using bytemuck
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER, 
            bytemuck::cast_slice(&chunk.mesh), 
            glow::STATIC_DRAW // Use STATIC_DRAW if the chunk rarely changes
        );

        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        (vao, vbo)
    };
    //
    // let (vao, _vbo) = unsafe {
    //     let vao = gl.create_vertex_array().unwrap();
    //     let vbo = gl.create_buffer().unwrap();
    //
    //     gl.bind_vertex_array(Some(vao));
    //     gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    //     gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices), glow::STATIC_DRAW);
    //
    //     gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
    //     gl.enable_vertex_attrib_array(0);
    //
    //     (vao, vbo)
    // };

    let vertex_shader_source = r#"
        #version 330 core
        layout (location = 0) in vec3 aPos;
        uniform mat4 u_transform;
        out vec3 vLocalPos;

        void main() { 
            gl_Position = u_transform * vec4(aPos, 1.0); 
            vLocalPos = aPos; 
        }
    "#;

    let fragment_shader_source = r#"
        #version 330 core
        in vec3 vLocalPos;
        out vec4 FragColor;

        void main() {
            FragColor = vec4(vLocalPos + 0.5, 1.0); 
        }
    "#;

    let program = unsafe { create_program(&gl, vertex_shader_source, fragment_shader_source) };
    
    let transform_loc = unsafe { gl.get_uniform_location(program, "u_transform") };

    let (win_w, win_h) = ctx.window.size();
    let mut projection = glam::Mat4::perspective_rh_gl(
        f32::to_radians(FOV), 
        win_w as f32 / win_h as f32, 
        0.1, 
        100.0
    );

    let mut cam = Camera {
        angle: vec3(0.0, -0.5 * PI, 0.0),
        pos:   Vec3::Z * 3.0,
    };

    let mut view = Mat4::ZERO;
    cam.update_view(&mut view);

    let timer = ctx.sdl.timer().unwrap();
    let mut last_frame_time = timer.ticks();
    let mut event_pump = ctx.sdl.event_pump().unwrap();
    let mut triangles = Vec::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => { break 'running; }
                Event::Window { win_event, .. } => {
                    if let sdl2::event::WindowEvent::Resized(width, height) = win_event {
                        unsafe { gl.viewport(0, 0, width, height) };
                        projection = glam::Mat4::perspective_rh_gl(
                            f32::to_radians(FOV), 
                            width as f32 / height as f32, 
                            0.1, 
                            100.0
                        );
                    }
                }
                Event::KeyDown { scancode: Some(Scancode::B), .. } => {
                    let reach_dist = 2.;
                    triangles.push(cam.pos + (cam.front() * reach_dist)); 
                }
                _ => {}
            }
        }

        let now         = timer.ticks();
        let delta_time  = (now - last_frame_time) as f32 / 1000.0;
        last_frame_time = now;

        let ks = event_pump.keyboard_state();
        let speed = 4.0 * delta_time;

        let front = cam.flat_front();
        let right = front.cross(Vec3::Y).normalize();

        let mut next_move = Vec3::ZERO;

        if ks.is_scancode_pressed(Scancode::W) { next_move += front; }
        if ks.is_scancode_pressed(Scancode::A) { next_move -= right; }
        if ks.is_scancode_pressed(Scancode::S) { next_move -= front; }
        if ks.is_scancode_pressed(Scancode::D) { next_move += right; }

        if ks.is_scancode_pressed(Scancode::Space) { next_move += Vec3::Y; }
        if ks.is_scancode_pressed(Scancode::C)     { next_move -= Vec3::Y; }

        let camera_speed: f32 = 2.0 * delta_time;

        if ks.is_scancode_pressed(Scancode::Up)     { cam.angle.x += camera_speed; }
        if ks.is_scancode_pressed(Scancode::Down)   { cam.angle.x -= camera_speed; }
        if ks.is_scancode_pressed(Scancode::Left)   { cam.angle.y -= camera_speed; }
        if ks.is_scancode_pressed(Scancode::Right)  { cam.angle.y += camera_speed; }

        {
            let max_pitch = 89.0_f32.to_radians();
            cam.angle.x = cam.angle.x.clamp(-max_pitch, max_pitch);
        }

        if next_move != Vec3::ZERO {
            cam.pos += next_move.normalize() * speed;
        }

        cam.update_view(&mut view);

        unsafe {
            gl.clear_color(0.1, 0.15, 0.2, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(program));
            gl.bind_vertex_array(Some(vao));

            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_HEIGHT {
                    for z in 0..CHUNK_SIZE {
                        if chunk.get(x as usize, y as usize, z as usize) {
                            let model = Mat4::from_translation(vec3(x as f32, y as f32, z as f32));
                            let mvp = projection * view * model;

                            gl.uniform_matrix_4_f32_slice(transform_loc.as_ref(), false, &mvp.to_cols_array());
                            gl.draw_arrays(glow::TRIANGLES, 0, 3);
                            
                        }
                    }
                }
            }

            for t in triangles.iter() {
                let model = Mat4::from_translation(*t);
                let mvp = projection * view * model;

                gl.uniform_matrix_4_f32_slice(transform_loc.as_ref(), false, &mvp.to_cols_array());
                gl.draw_arrays(glow::TRIANGLES, 0, 3);
            }
        }

        ctx.window.gl_swap_window();
    }
}

unsafe fn create_program(gl: &glow::Context, vert: &str, frag: &str) -> glow::Program {
    unsafe {
        let program = gl.create_program().unwrap();
        let vs = compile_shader(gl, glow::VERTEX_SHADER, vert);
        let fs = compile_shader(gl, glow::FRAGMENT_SHADER, frag);
        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);
        gl.link_program(program);
        gl.delete_shader(vs);
        gl.delete_shader(fs);
        program
    }
}

unsafe fn compile_shader(gl: &glow::Context, shader_type: u32, source: &str) -> glow::Shader {
    unsafe {
        let shader = gl.create_shader(shader_type).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        shader
    }
}
