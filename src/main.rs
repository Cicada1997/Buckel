pub mod camera;
pub mod utils;
pub mod chunk;

use crate::{
    camera::Camera, chunk::{VoxelWorld}, utils::SdlContext
};

use {
    std::f32::consts::PI,
    glam::{ Mat4, Vec3, vec3 },
    glow::{
        Context,
        HasContext,
        NativeBuffer,
    },
    sdl2::{
        keyboard::{
            Scancode,
        },
        mouse::MouseButton,
        event::Event,
    },
};

pub type Error = Box<dyn std::error::Error>;

static FOV: f32 = 90.0;
static MOUSE_SENSATIVITY: f32 = 6.0;

// fn update_chunk(gl: &glow::Context, vbo: &mut NativeBuffer, vertex_count: &mut i32, mesh: &Vec<f32>) {
//     *vertex_count = (mesh.len() / 3) as i32;
//     unsafe {
//         gl.bind_buffer(glow::ARRAY_BUFFER, Some(*vbo));
//         gl.buffer_data_u8_slice(
//             glow::ARRAY_BUFFER,
//             bytemuck::cast_slice(mesh),
//             glow::DYNAMIC_DRAW,
//         );
//     }
// }

fn create_chunk_vbo(gl: Context) -> NativeBuffer {
    unsafe {
        // let vao = gl.create_vertex_array().unwrap();
        let vbo = gl.create_buffer().unwrap();

        // gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        // gl.buffer_data_u8_slice(
        //     glow::ARRAY_BUFFER, 
        //     bytemuck::cast_slice(&world.last_mesh), 
        //     glow::STATIC_DRAW 
        // );

        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        return vbo;
    };
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

    // world.nearby_chunk_mesh(vec3(0., 0., 0.));
    // let mut vertex_count = (world.last_mesh.len() / 3) as i32;

    // let (vao, mut vbo) = unsafe {
    //     let vao = gl.create_vertex_array().unwrap();
    //     let vbo = gl.create_buffer().unwrap();
    //
    //     gl.bind_vertex_array(Some(vao));
    //     gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    //
    //     gl.buffer_data_u8_slice(
    //         glow::ARRAY_BUFFER, 
    //         bytemuck::cast_slice(&world.last_mesh), 
    //         glow::STATIC_DRAW 
    //     );
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
            FragColor = vec4(fract(vLocalPos), 1.0); 
        }
    "#;

    let program       = unsafe { create_program(&gl, vertex_shader_source, fragment_shader_source) };
    let transform_loc = unsafe { gl.get_uniform_location(program, "u_transform") }.expect("Could not get uniform location");

    let mut world = VoxelWorld::default();
    world.transform_loc = Some(transform_loc.clone());

    let (win_w, win_h) = ctx.window.size();
    let mut projection = glam::Mat4::perspective_rh_gl(
        f32::to_radians(FOV), 
        win_w as f32 / win_h as f32, 
        0.1, 
        10000.0
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


    let mut selected_block = 1;

    'running: loop {
        let mut movement_speed = 14.;

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
                            10000.0
                        );
                    }
                }

                // Event::KeyDown { keymod: Mod::LSHIFTMOD, .. } => {
                //     movement_speed = 120.;
                // }
                //
                Event::MouseMotion { xrel, yrel, .. } => {
                    cam.angle.y += (xrel as f32 / 1000.0) * MOUSE_SENSATIVITY;
                    cam.angle.x -= (yrel as f32 / 1000.0) * MOUSE_SENSATIVITY;
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    let bx = pos.x.floor() as i32;
                    let by = pos.y.floor() as i32;
                    let bz = pos.z.floor() as i32;

                    match mouse_btn {
                        MouseButton::Right => {
                            if world.set_block(bx, by, bz, Some(selected_block)).is_err() {
                                world.build_chunk(&gl, &VoxelWorld::chunk_pos(bx, bz));
                                world.set_block(bx, by, bz, Some(selected_block)).unwrap();
                            }
                        }
                        MouseButton::Left => {
                            if world.set_block(bx, by, bz, None).is_err() {
                                world.build_chunk(&gl, &VoxelWorld::chunk_pos(bx, bz));
                                world.set_block(bx, by, bz, None).unwrap();
                            }
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        let now         = timer.ticks();
        let fps_time  = (now - last_frame_time) as f64 / 1000.0;
        let delta_time  = (now - last_frame_time) as f32 / 1000.0;
        last_frame_time = now;

        print!("\rFPS: {:.2}", 1. / fps_time);

        let ks = event_pump.keyboard_state();
        if event_pump.is_event_enabled(sdl2::event::EventType::KeyDown) {
            movement_speed = 120.;
        }

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
            cam.pos += next_move.normalize() * movement_speed * delta_time;
        }

        cam.update_view(&mut view);

        world.try_build_nearby_chunks(&gl, &cam.pos);
         
        // update_chunk(&gl, &mut vertex_count, &world.last_mesh);

        unsafe {
            gl.clear_color(0.1, 0.15, 0.2, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            gl.use_program(Some(program));
            // gl.bind_vertex_array(Some(vao));

            let mvp = projection * view * Mat4::IDENTITY;
            world.render(&gl, &cam.pos, &mvp);
            // if vertex_count > 0 {
            //     let chunk_model = Mat4::IDENTITY;
            //     gl.uniform_matrix_4_f32_slice(transform_loc.as_ref(), false, &chunk_mvp.to_cols_array());
            //     gl.draw_arrays(glow::TRIANGLES, 0, vertex_count);
            // }
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
