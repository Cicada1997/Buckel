use glow::{NativeUniformLocation, NativeVertexArray};

use crate::{
    limit,
    Error,
};

use {
    std::collections::HashMap,
    glam::{
        Vec3,
        Mat4,
    },
    glow::{
        Context,
        HasContext,
        NativeBuffer,
    },
};

pub static SEED:   u32 = 12938;
pub static CHUNK_SIZE:   u16 = 16;
pub static CHUNK_HEIGHT: u16 = 64;
pub type ByteChunkLayer      = [u16;            CHUNK_SIZE   as usize];
pub type ByteChunk           = [ByteChunkLayer; CHUNK_HEIGHT as usize];

pub struct Chunk {
    blocks: [ByteChunk; BLOCK_TYPE_COUNT],
    mesh: Vec<f32>,
    vbo: NativeBuffer,
    vao: NativeVertexArray,
    pos: ChunkPosition,
    pub dirty: bool,
}

pub static BLOCK_TYPE_COUNT: usize = 3;
type BlockTypeId = usize;

/// Z is bit position
impl Chunk {
    fn new(pos: ChunkPosition, gl: &Context) -> Self {
        let (vao, vbo) = unsafe {
            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();
            
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            
            // Define layout WHILE the VAO is bound
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(0);
            
            (vao, vbo)
        };

        let mut chunk = Self { 
            blocks: [[[0u16; CHUNK_SIZE as usize]; CHUNK_HEIGHT as usize]; BLOCK_TYPE_COUNT],
            mesh: Vec::new(),
            pos,
            vbo,
            vao,
            dirty: true
        };

        chunk.gen_perlin();
        // chunk.gen_mesh(); // Call this to generate initial mesh
        // chunk.upload_mesh(gl); // Send to GPU
        chunk
    }

    pub fn upload_mesh(&self, gl: &Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&self.mesh),
                glow::DYNAMIC_DRAW,
            );
        }
    }

    fn render(&self, gl: &Context) {
        let vertex_count = (self.mesh.len() / 3) as i32;
        if vertex_count == 0 { return; }
        
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, vertex_count);
        }
    }

    fn mesh_coord(&self, x: usize, y: usize, z: usize) -> Vec3 {
        let world_x_offset = self.pos.0 as f32 * CHUNK_SIZE as f32;
        let world_z_offset = self.pos.1 as f32 * CHUNK_SIZE as f32;

        let fx = x as f32 + world_x_offset;
        let fy = y as f32;
        let fz = z as f32 + world_z_offset;

        return Vec3 { x: fx, y: fy, z: fz };
    }

    fn gen_mesh(&mut self) {
                
        let mut mesh = Vec::new();

        for (_block_type_id, bytechunk) in self.blocks.iter().enumerate() {
            for y in 0..CHUNK_HEIGHT as usize {
                let layer = bytechunk[y];

                for x in 0..CHUNK_SIZE as usize {
                    let row = layer[x];

                    // (-Z) x-shift right 
                    let render_scheme = (!row & (row >> 1)) << 1;
                    for z in 0..CHUNK_SIZE {
                        let pos = self.mesh_coord(x, y, z as usize);
                        if render_scheme >> z & 1 == 1 {
                            mesh.extend_from_slice(&[
                                pos.x+0., pos.y+0., pos.z+0.,   pos.x+0., pos.y+1., pos.z+0.,   pos.x+1., pos.y+1., pos.z+0., 
                                pos.x+1., pos.y+1., pos.z+0.,   pos.x+1., pos.y+0., pos.z+0.,   pos.x+0., pos.y+0., pos.z+0.,
                            ]);
                        }
                    }

                    // (+Z) x-shift left
                    let render_scheme = (!row & (row << 1)) >> 1;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x, y, z as usize);

                            mesh.extend_from_slice(&[
                                pos.x+0., pos.y+0., pos.z+1.,   pos.x+1., pos.y+0., pos.z+1.,   pos.x+1., pos.y+1., pos.z+1., 
                                pos.x+1., pos.y+1., pos.z+1.,   pos.x+0., pos.y+1., pos.z+1.,   pos.x+0., pos.y+0., pos.z+1.,
                            ]);
                        }
                    }

                    // (+X) z-shift right
                    let left_row = layer[(x+1).clamp(0, 15)];
                    let render_scheme = row & !left_row;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x, y, z as usize);
                            mesh.extend_from_slice(&[
                                pos.x+1., pos.y+0., pos.z+0.,   pos.x+1., pos.y+1., pos.z+0.,   pos.x+1., pos.y+1., pos.z+1., 
                                pos.x+1., pos.y+1., pos.z+1.,   pos.x+1., pos.y+0., pos.z+1.,   pos.x+1., pos.y+0., pos.z+0.,
                            ]);
                        }
                    }

                    // (-X) z-shift left
                    let right_row = layer[x.checked_sub(1).unwrap_or(0)];
                    let render_scheme = row & !right_row;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x-1, y, z as usize);
                            mesh.extend_from_slice(&[
                                pos.x+1., pos.y+0., pos.z+0.,   pos.x+1., pos.y+1., pos.z+0.,   pos.x+1., pos.y+1., pos.z+1., 
                                pos.x+1., pos.y+1., pos.z+1.,   pos.x+1., pos.y+0., pos.z+1.,   pos.x+1., pos.y+0., pos.z+0.,
                            ]);
                        }
                    }

                    let a = [16u16; CHUNK_SIZE as usize];
                    let next_layer = bytechunk.get(y+1).unwrap_or(&a);
                    let next_layer_row = next_layer[x];

                    // (+Y)
                    let render_scheme = row & !next_layer_row;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x, y, z as usize);

                            mesh.extend_from_slice(&[
                                pos.x+0., pos.y+1., pos.z+0.,   pos.x+0., pos.y+1., pos.z+1.,   pos.x+1., pos.y+1., pos.z+1., 
                                pos.x+1., pos.y+1., pos.z+1.,   pos.x+1., pos.y+1., pos.z+0.,   pos.x+0., pos.y+1., pos.z+0.,
                            ]);
                        }
                    }

                    let past_layer = bytechunk.get(y.checked_sub(1).unwrap_or(0)).unwrap_or(&a);
                    let past_layer_row = past_layer[x];
                    let render_scheme = row & !past_layer_row;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x, y, z as usize);

                            mesh.extend_from_slice(&[
                                pos.x+0., pos.y+0., pos.z+0.,   pos.x+0., pos.y+0., pos.z+1.,   pos.x+1., pos.y+0., pos.z+1., 
                                pos.x+1., pos.y+0., pos.z+1.,   pos.x+1., pos.y+0., pos.z+0.,   pos.x+0., pos.y+0., pos.z+0.,
                            ]);
                        }
                    }
                }
            }
        }

        // self.mesh.clear();
        self.mesh = mesh;

        self.dirty = false;
    }


    fn gen_perlin(&mut self) {
        use noise::{NoiseFn, Perlin};

        self.dirty = true;
        let perlin = Perlin::new(SEED);

        let frequency = 0.03; 
        let terrain_height_multiplier = 20.0;
        let sea_level = 10.0;

        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let world_x = (x + self.pos.0 * CHUNK_SIZE as i32) as f64;
                let world_z = (z + self.pos.1 * CHUNK_SIZE as i32) as f64;

                let noise_val = perlin.get([world_x * frequency, world_z * frequency]);

                let height = ((noise_val + 1.0) * 0.5 * terrain_height_multiplier + sea_level) as i32;

                for y in 0..CHUNK_HEIGHT as i32 {
                    if y <= height {
                        self.set_block(x as usize, y as usize, z as usize, Some(1));
                    }
                }
            }
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockTypeId> {
        limit!(x, CHUNK_SIZE as usize,   "chunk relative position outside chunk");
        limit!(y, CHUNK_HEIGHT as usize, "chunk relative position outside chunk");
        limit!(z, CHUNK_SIZE as usize,   "chunk relative position outside chunk");

        for (block_type_id, bytechunk) in self.blocks.iter().enumerate() {
            if (bytechunk[y][x] >> z) & 1 == 1 {
                return Some(block_type_id);
            }
        }

        return None;
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block_type: Option<BlockTypeId>) {
        limit!(x, CHUNK_SIZE as usize,   "chunk relative position outside chunk");
        limit!(y, CHUNK_HEIGHT as usize, "chunk relative position outside chunk");
        limit!(z, CHUNK_SIZE as usize,   "chunk relative position outside chunk");

        if let Some(block_type) = block_type {
            self.blocks[block_type][y][x] = self.blocks[block_type][y][x] | (1 << z);
        } else {
            for bytechunk in self.blocks.iter_mut() {
                bytechunk[y][x] = bytechunk[y][x] & !(1 << z);
            }
        }

        self.dirty = true;
    }
}

pub type ChunkPosition = (i32, i32);
pub struct VoxelWorld {
    world: HashMap<ChunkPosition, Chunk>,
    pub transform_loc: Option<NativeUniformLocation>,
    // pub last_mesh: Vec<f32>,
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self {
            world: HashMap::new(),
            transform_loc: None,
        }
    }
}

impl VoxelWorld {
    pub fn chunk_pos(x: i32, z: i32) -> ChunkPosition {
        let cx = (x as f32 / CHUNK_SIZE as f32).floor() as i32;
        let cz = (z as f32 / CHUNK_SIZE as f32).floor() as i32;

        return (cx, cz);
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_type: Option<BlockTypeId>) -> Result<(), Error> {
        let cp = Self::chunk_pos(x, z);

        let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;

        if y >= 0 && y < CHUNK_HEIGHT as i32 {
            let chunk = self.world.get_mut(&cp).ok_or("Chunk entry does not exist ({}, {}).")?;
            chunk.dirty = true;
            chunk.set_block(local_x, y as usize, local_z, block_type);
        }

        Ok(())
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<BlockTypeId> {
        let cp = Self::chunk_pos(x as i32, z as i32);
        
        let local_x = (x % CHUNK_SIZE as i32) as usize;
        let local_z = (z % CHUNK_SIZE as i32) as usize;

        if let Some(chunk) = self.world.get(&cp) {
            return chunk.get_block(local_x, y as usize, local_z)
        }

        None
    }

    pub fn build_chunk(&mut self, gl: &Context, pos: &ChunkPosition) {
        self.world.insert(*pos, 
            Chunk::new(*pos, gl)
        );
    }

    pub fn try_build_nearby_chunks(&mut self, gl: &Context, position: &Vec3) {
        let p_cx = (position.x / CHUNK_SIZE as f32).floor() as i32;
        let p_cz = (position.z / CHUNK_SIZE as f32).floor() as i32;

        const RENDER_DISTANCE: i32 = 4;

        for cx in (p_cx - RENDER_DISTANCE)..=(p_cx + RENDER_DISTANCE) {
            for cz in (p_cz - RENDER_DISTANCE)..=(p_cz + RENDER_DISTANCE) {
                let cp = (cx, cz);

                match self.world.get_mut(&cp) {
                    Some(chunk) => {
                        if chunk.dirty {
                            chunk.gen_mesh();
                            chunk.upload_mesh(gl);
                        }
                    }
                    None => {
                        let mut new_chunk = Chunk::new(cp, gl);
                        new_chunk.gen_mesh();
                        new_chunk.upload_mesh(gl);
                        self.world.insert(cp, new_chunk);
                    }
                }

                // let chunk = self.world.get(&cp).expect("Building chunk failed; could not get chunk after meshbuild (should assure chunk exists).");

                // let world_x_offset = cx as f32 * CHUNK_SIZE as f32;
                // let world_z_offset = cz as f32 * CHUNK_SIZE as f32;
                //
                // for i in (0..chunk.mesh.len()).step_by(3) {
                //     total_mesh.push(chunk.mesh[i] + world_x_offset);
                //     total_mesh.push(chunk.mesh[i + 1]);
                //     total_mesh.push(chunk.mesh[i + 2] + world_z_offset);
                // }
            }
        }
    }

    pub fn render(&mut self, gl: &Context, position: &Vec3, mvp: &Mat4) {
        let p_cx = (position.x / CHUNK_SIZE as f32).floor() as i32;
        let p_cz = (position.z / CHUNK_SIZE as f32).floor() as i32;

        const RENDER_DISTANCE: i32 = 4;
        
        unsafe {
            gl.uniform_matrix_4_f32_slice(self.transform_loc.as_ref(), false, &mvp.to_cols_array());
        }

        for cx in (p_cx - RENDER_DISTANCE)..=(p_cx + RENDER_DISTANCE) {
            for cz in (p_cz - RENDER_DISTANCE)..=(p_cz + RENDER_DISTANCE) {
                let cp = (cx, cz);

                unsafe {
                    gl.uniform_matrix_4_f32_slice(self.transform_loc.as_ref(), false, &mvp.to_cols_array());
                }
                match self.world.get_mut(&cp) {
                    Some(chunk) => {
                        chunk.render(gl);
                    }
                    None => {
                        self.world.insert(cp, 
                            Chunk::new(cp, gl)
                        );
                    }
                }

                // let chunk = self.world.get(&cp).expect("Building chunk failed; could not get chunk after meshbuild (should assure chunk exists).");

                // let world_x_offset = cx as f32 * CHUNK_SIZE as f32;
                // let world_z_offset = cz as f32 * CHUNK_SIZE as f32;
                //
                // for i in (0..chunk.mesh.len()).step_by(3) {
                //     total_mesh.push(chunk.mesh[i] + world_x_offset);
                //     total_mesh.push(chunk.mesh[i + 1]);
                //     total_mesh.push(chunk.mesh[i + 2] + world_z_offset);
                // }
            }
        }
    }
}
