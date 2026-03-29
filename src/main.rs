pub mod camera;
pub mod utils;
pub mod chunk;

use sdl2::mouse::MouseButton;

use {
    camera::Camera, 
    utils::SdlContext,
    chunk::Chunk,
};

use {
    std::f32::consts::PI,
    glam::{ Mat4, Vec3, vec3 },
    glow::{
        HasContext,
        NativeBuffer,
    },
    sdl2::{
        keyboard::Scancode,
        event::Event,
    },
};

static FOV: f32 = 90.0;
static MOUSE_SENSATIVITY: f32 = 6.0;

fn update_chunk(gl: &glow::Context, vbo: &mut NativeBuffer, vertex_count: &mut i32, mesh: &Vec<f32>) {
    *vertex_count = (mesh.len() / 3) as i32;
    unsafe {
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(*vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(mesh),
            glow::DYNAMIC_DRAW,
        );
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

        SdlContext { sdl, video, window }
    };
    
    let mouse = ctx.sdl.mouse();
    mouse.show_cursor(false);
    mouse.set_relative_mouse_mode(true);

    let gl_attr = ctx.video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let _gl_context = ctx.window.gl_create_context().unwrap();
    let gl = unsafe {
        let gl = glow::Context::from_loader_function(|s| ctx.video.gl_get_proc_address(s) as *const _);
        gl.enable(glow::DEPTH_TEST);
        gl
    };

    let mut chunk = Chunk::default();
    chunk.build_mesh();

    let mut vertex_count = (chunk.mesh.len() / 3) as i32;

    let (vao, mut vbo) = unsafe {
        let vao = gl.create_vertex_array().unwrap();
        let vbo = gl.create_buffer().unwrap();

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER, 
            bytemuck::cast_slice(&chunk.mesh), 
            glow::STATIC_DRAW 
        );

        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        (vao, vbo)
    };

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
            FragColor = vec4(fract(vLocalPos), 1.0); 
        }
    "#;

    let program       = unsafe { create_program(&gl, vertex_shader_source, fragment_shader_source) };
    let transform_loc = unsafe { gl.get_uniform_location(program, "u_transform") };

    let (win_w, win_h) = ctx.window.size();
    let mut projection = glam::Mat4::perspective_rh_gl(
        f32::to_radians(FOV), 
        win_w as f32 / win_h as f32, 
        0.1, 
        100.0
    );

    let mut cam = Camera {
        angle: vec3(0.0, 0.5 * PI, 0.0),
        pos:   Vec3::Z * -3.0,
    };

    let mut view = Mat4::ZERO;
    cam.update_view(&mut view);

    let timer = ctx.sdl.timer().unwrap();
    let mut last_frame_time = timer.ticks();
    let mut event_pump = ctx.sdl.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            let reach_dist = 2.;
            let pos = cam.pos + (cam.front() * reach_dist);

            match event {
                Event::Quit { .. } | Event::KeyDown { scancode: Some(Scancode::Escape), .. } => { break 'running; }
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

                Event::MouseMotion { xrel, yrel, .. } => {
                    cam.angle.y += (xrel as f32 / 1000.0) * MOUSE_SENSATIVITY;
                    cam.angle.x -= (yrel as f32 / 1000.0) * MOUSE_SENSATIVITY;
                }

                Event::MouseButtonDown { mouse_btn: MouseButton::Right, .. } => {
                    chunk.set_block(pos.x as usize, pos.y as usize, pos.z as usize, true);
                    chunk.dirty = true;
                    chunk.build_mesh();
                }

                Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                    chunk.set_block(pos.x as usize, pos.y as usize, pos.z as usize, false);
                    chunk.dirty = true;
                    chunk.build_mesh();
                }

                _ => {}
            }
        }

        let now         = timer.ticks();
        let delta_time  = (now - last_frame_time) as f32 / 1000.0;
        last_frame_time = now;

        let ks = event_pump.keyboard_state();
        let speed = 8.0 * delta_time; // Ökade speed lite så du kan flyga runt enklare

        let front = cam.flat_front();
        let right = front.cross(Vec3::Y).normalize();

        let mut next_move = Vec3::ZERO;

        if ks.is_scancode_pressed(Scancode::W)     { next_move += front; }
        if ks.is_scancode_pressed(Scancode::A)     { next_move -= right; }
        if ks.is_scancode_pressed(Scancode::S)     { next_move -= front; }
        if ks.is_scancode_pressed(Scancode::D)     { next_move += right; }
        if ks.is_scancode_pressed(Scancode::Space) { next_move += Vec3::Y; }
        if ks.is_scancode_pressed(Scancode::C)     { next_move -= Vec3::Y; }

        let camera_speed: f32 = 2.0 * delta_time;

        if ks.is_scancode_pressed(Scancode::Up)     { cam.angle.x += camera_speed; }
        if ks.is_scancode_pressed(Scancode::Down)   { cam.angle.x -= camera_speed; }
        if ks.is_scancode_pressed(Scancode::Left)   { cam.angle.y -= camera_speed; }
        if ks.is_scancode_pressed(Scancode::Right)  { cam.angle.y += camera_speed; }

        let max_pitch = 89.0_f32.to_radians();
        cam.angle.x = cam.angle.x.clamp(-max_pitch, max_pitch);

        if next_move != Vec3::ZERO {
            cam.pos += next_move.normalize() * speed;
        }

        cam.update_view(&mut view);

        update_chunk(&gl, &mut vbo, &mut vertex_count, &chunk.mesh);

        unsafe {
            gl.clear_color(0.1, 0.15, 0.2, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(program));
            gl.bind_vertex_array(Some(vao));

            if vertex_count > 0 {
                let chunk_model = Mat4::IDENTITY;
                let chunk_mvp = projection * view * chunk_model;
                gl.uniform_matrix_4_f32_slice(transform_loc.as_ref(), false, &chunk_mvp.to_cols_array());
                gl.draw_arrays(glow::TRIANGLES, 0, vertex_count);
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
