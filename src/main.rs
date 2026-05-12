pub mod textures;
pub mod coordinate;
pub mod camera;
pub mod utils;
pub mod chunk;
pub mod world;

use crate::{
    camera::Camera,
    world::VoxelWorld,
    coordinate::WorldPosition,
    utils::SdlContext,
};

use {
    std::f32::consts::PI,
    glam::{ Mat4, Vec3, vec3 },
    glow::HasContext,
    sdl2::{
        keyboard::Scancode,
        mouse::MouseButton,
        event::Event,
    },
};

pub type Error = Box<dyn std::error::Error>;

static FOV: f32 = 90.0;
static MOUSE_SENSATIVITY: f32 = 6.0;

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
    gl_attr.set_depth_size(24);

    let _gl_context = ctx.window.gl_create_context().unwrap();
    let gl = unsafe {
        let gl = glow::Context::from_loader_function(|s| ctx.video.gl_get_proc_address(s) as *const _);
        gl.enable(glow::DEPTH_TEST);
        gl
    };
    let vertex_shader_source = r#"
        #version 330 core
        layout (location = 0) in vec3 aPos;
        layout (location = 1) in vec2 aTexCoord;

        uniform mat4 u_transform; // MVP matrix
        uniform mat4 u_model;     // Model matrix

        out vec2 vTexCoord;
        out vec3 vFragPos;        // Pass the world position instead of a normal

        void main() { 
            gl_Position = u_transform * vec4(aPos, 1.0); 
            vTexCoord = aTexCoord; 
            
            // Calculate the position of the vertex in the 3D world
            vFragPos = vec3(u_model * vec4(aPos, 1.0)); 
        }
    "#;

    let fragment_shader_source = r#"
        #version 330 core
        in vec2 vTexCoord; 
        in vec3 vFragPos; 
        out vec4 FragColor;

        uniform sampler2D u_Texture; 
        uniform vec3 u_viewPos; 

        // 1. Direction TOWARDS the sun. 
        // If your voxels face +Y (up), a sun with Y=1.0 will hit the top.
        const vec3 sunDir = normalize(vec3(0.4, 1.0, 0.2)); 
        const vec3 sunColor = vec3(1.0, 0.95, 0.8); // Warm sunlight
        const float ambientStrength = 0.4;          // Boosted for visibility

        void main() {
            // 2. Generate Normals
            vec3 fdx = dFdx(vFragPos);
            vec3 fdy = dFdy(vFragPos);
            vec3 norm = normalize(cross(fdx, fdy));

            // 3. Lighting Calculation
            // We use 'abs' here as a temporary test: if this makes it bright, 
            // your normals were just pointing inside the cubes!
            float diff = max(dot(norm, sunDir), 0.0);
            
            // 4. Sun Disk (Visual Sun in the sky)
            vec3 viewDir = normalize(vFragPos - u_viewPos);
            float sunAlignment = dot(viewDir, sunDir); 
            float sunDisk = smoothstep(0.997, 0.999, sunAlignment);
            vec3 sunElement = sunDisk * vec3(5.0, 5.0, 3.0); 

            // 5. Final Color
            vec4 texColor = texture(u_Texture, vTexCoord);
            
            // Apply a slight blue tint to shadows (ambient) for a better voxel look
            vec3 ambient = ambientStrength * vec3(0.6, 0.7, 0.9);
            vec3 diffuse = diff * sunColor;
            
            vec3 finalResult = (ambient + diffuse) * texColor.rgb + sunElement;
            
            FragColor = vec4(finalResult, texColor.a);

            // --- DEBUG LINE ---
            // Uncomment the line below to see your normals as colors. 
            // Red/Green/Blue = facing X/Y/Z. If it's all black, vFragPos is broken.
            // FragColor = vec4(norm * 0.5 + 0.5, 1.0); 
        }
    "#;
    // let vertex_shader_source = r#"
    //     #version 330 core
    //     layout (location = 0) in vec3 aPos;
    //     layout (location = 1) in vec2 aTexCoord; // 1. Accept UV coords from buffer
    //
    //     uniform mat4 u_transform;
    //     out vec2 vTexCoord; // 2. Pass UVs to the fragment shader
    //
    //     void main() { 
    //         gl_Position = u_transform * vec4(aPos, 1.0); 
    //         vTexCoord = aTexCoord; 
    //     }
    // "#;
    //
    // let fragment_shader_source = r#"
    //     #version 330 core
    //     in vec2 vTexCoord; // 1. Receive UVs from vertex shader
    //     out vec4 FragColor;
    //
    //     uniform sampler2D u_Texture; // 2. Accept the bound texture
    //
    //     void main() {
    //         // 3. Sample the pixel color from the texture
    //         FragColor = texture(u_Texture, vTexCoord); 
    //     }
    // "#;

    // let vertex_shader_source = r#"
    //     #version 330 core
    //     layout (location = 0) in vec3 aPos;
    //     uniform mat4 u_transform;
    //     out vec3 vLocalPos;
    //
    //     void main() { 
    //         gl_Position = u_transform * vec4(aPos, 1.0); 
    //         vLocalPos = aPos; 
    //     }
    // "#;
    //
    // let fragment_shader_source = r#"
    //     #version 330 core
    //     in vec3 vLocalPos;
    //     out vec4 FragColor;
    //
    //     void main() {
    //         FragColor = vec4(255.0, 255.0, 255.0, 1.0); 
    //     }
    // "#;
    //
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
        pos:   WorldPosition::new(0., 0., -3.),
    };

    let mut view = Mat4::ZERO;
    cam.update_view(&mut view);

    let timer = ctx.sdl.timer().unwrap();
    let mut last_frame_time = timer.ticks();
    let mut event_pump = ctx.sdl.event_pump().unwrap();

    let texture = textures::load_texture("example.jpg", &gl).unwrap();

    let selected_block = 1;

    'running: loop {
        let mut movement_speed = 14.;

        for event in event_pump.poll_iter() {
            let reach_dist = 2.;
            let pos = WorldPosition::from_relative_pos(cam.pos.world_position + (cam.front() * reach_dist));

            match event {
                Event::Quit { .. } | Event::KeyDown { scancode: Some(Scancode::Escape), .. } => { break 'running; }
                Event::Window { win_event: sdl2::event::WindowEvent::Resized(width, height), .. } => {
                    unsafe { gl.viewport(0, 0, width, height) };
                    projection = glam::Mat4::perspective_rh_gl(
                        f32::to_radians(FOV), 
                        width as f32 / height as f32, 
                        0.1, 
                        10000.0
                    );
                }

                Event::MouseMotion { xrel, yrel, .. } => {
                    cam.angle.y += (xrel as f32 / 1000.0) * MOUSE_SENSATIVITY;
                    cam.angle.x -= (yrel as f32 / 1000.0) * MOUSE_SENSATIVITY;
                }

                Event::MouseButtonDown { mouse_btn, .. } => {
                    println!("click!");
                    match mouse_btn {
                        MouseButton::Right => {
                            match world.set_block(&pos, Some(selected_block)) {
                                Ok(()) => {}
                                Err(_) => {
                                    println!("fuck, this should not happen!");
                                    world.build_chunk(&gl, &pos.chunk_pos());
                                    world.set_block(&pos, Some(selected_block)).unwrap();
                                }
                            }
                        }

                        MouseButton::Left => {
                            match world.set_block(&pos, None) {
                                Ok(()) => {}
                                Err(_) => {
                                    println!("fuck, this should not happen!");
                                    world.build_chunk(&gl, &pos.chunk_pos());
                                    world.set_block(&pos, None).unwrap();
                                }
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

        print!("\rFPS: {:.2} pos: {}", 1. / fps_time, &cam.pos.world_position);

        let ks = event_pump.keyboard_state();
        if event_pump.is_event_enabled(sdl2::event::EventType::KeyDown) {
            movement_speed = 40.;
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
            cam.pos.world_position += next_move.normalize() * movement_speed * delta_time;
        }

        cam.update_view(&mut view);

        world.try_build_nearby_chunks(&gl, &cam.pos);
         
        unsafe {
            gl.clear_color(0.5, 0.7, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            let view_pos_location = gl.get_uniform_location(program, "u_viewPos");
            gl.uniform_3_f32(view_pos_location.as_ref(), cam.pos.world_position.x, cam.pos.world_position.y, cam.pos.world_position.z);

            gl.use_program(Some(program));
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            let mvp = projection * view * Mat4::IDENTITY;
            world.render(&gl, &cam.pos, &mvp);
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
