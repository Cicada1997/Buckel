use crate::Error;
use {
    glow::{
        Context,
        HasContext,
        NativeTexture,
    }, 
};

pub fn load_texture(path: &str, gl: &Context) -> Result<NativeTexture, Error> {
    let img = image::open(path).expect("Failed to load image").to_rgba8();
    let (w, h) = img.dimensions();

    unsafe {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            w as i32,
            h as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(&img),
        );
        Ok(texture)
    }
}

// pub fn t(gl: &Context) {
    // let tex = gl.create_texture()?;
    // gl.bind_texture(Texture, tex);
// }

// pub struct TextureHouse {
//     pub textures: HashMap<String, Texture>,
//     raw_data: HashMap<String, TextureData>,
// }
//
// pub struct TextureData {
//     pub width: u32,
//     pub height: u32,
//     pub data: Vec<u8>,
// }
//
// impl TextureHouse {
//     pub fn new() -> Self {
//         Self { 
//             textures: HashMap::new(),
//             raw_data: HashMap::new(),
//         }
//     }
//
//     pub fn load_texture(&mut self, path: &str) -> Result<(), Error> {
//         let surface = Surface::from_file(path)?;  
//         let width = surface.width();  
//         let height = surface.height();  
//
//         // Convert to RGBA for OpenGL
//         let surface = surface.convert_format(sdl2::pixels::PixelFormatEnum::RGBA8888)?;  
//         let data = surface.with_lock(|pixels| pixels.to_vec());  
//
//         self.raw_data.insert(path.to_string(), TextureData { width, height, data });
//         Ok(())
//     }
//
//     pub fn upload_textures(&mut self, gl: &Context) -> Result<(), Error> {
//         for (path, text) in &self.raw_data {
//             unsafe {
//                 let tex = gl.create_texture()?;
//                 gl.bind_texture(glow::TEXTURE_2D, Some(tex));
//
//                 // Essential: Set wrapping and filtering parameters 
//                 // Without these, textures often render as pure black
//                 gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
//                 gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
//                 gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
//                 gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
//
//                 gl.tex_image_2d(  
//                     glow::TEXTURE_2D, 0, glow::RGBA as i32,  
//                     text.width as i32, text.height as i32,  
//                     0, glow::RGBA, glow::UNSIGNED_BYTE, Some(&text.data)
//                 );
//
//                 self.textures.insert(path.clone(), tex);
//             }
//         }
//         Ok(())
//     }
// }
