use crate::{
    BLOCK_TYPE_COUNT, limit
};

use {
    std::collections::HashMap,
    glam::Vec3,
};

pub static CHUNK_SIZE:   u16 = 16;
pub static CHUNK_HEIGHT: u16 = 64;
pub type ByteChunkLayer      = [u16;            CHUNK_SIZE   as usize];
pub type ByteChunk           = [ByteChunkLayer; CHUNK_HEIGHT as usize];

pub struct Chunk {
    blocks: [ByteChunk; BLOCK_TYPE_COUNT],
    pub mesh: Vec<f32>,
    pub dirty: bool,
}

type BlockTypeId = usize;

impl Chunk {
    fn new(cx: i32, cz: i32) -> Self {
        let blocks = [
            [
                [0u16; CHUNK_SIZE as usize];
            CHUNK_HEIGHT as usize];
        BLOCK_TYPE_COUNT];

        let mut chunk = Self { 
            blocks,
            mesh: Vec::new(),
            dirty: true
        };

        chunk.gen_perlin(cx, cz);

        chunk
    }

    fn gen_perlin(&mut self, cx: i32, cz: i32) {
        use noise::{NoiseFn, Perlin};

        self.dirty = true;
        let perlin = Perlin::new(1);

        let frequency = 0.03; 
        let terrain_height_multiplier = 20.0;
        let sea_level = 10.0;

        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let world_x = (x + cx * CHUNK_SIZE as i32) as f64;
                let world_z = (z + cz * CHUNK_SIZE as i32) as f64;

                // 2. Use 2D Noise to get a height value at this (x, z)
                let noise_val = perlin.get([world_x * frequency, world_z * frequency]);

                // 3. Convert noise (-1.0 to 1.0) into a block height
                let height = ((noise_val + 1.0) * 0.5 * terrain_height_multiplier + sea_level) as i32;

                for y in 0..CHUNK_HEIGHT as i32 {
                    if y <= height {
                        self.set_block(x as usize, y as usize, z as usize, Some(1));
                    }
                }
            }
        }
    }

    // fn gen_perlin(&mut self, cx: i32, cz: i32) {
    //     use noise::{NoiseFn, Perlin, Seedable};
    //
    //     self.dirty = true;
    //
    //     let perlin = Perlin::new(1);
    //
    //     for x in 0..CHUNK_SIZE as i32 {
    //         for y in 0..CHUNK_HEIGHT as i32 {
    //             for z in 0..CHUNK_SIZE as i32 {
    //                 if perlin.get([
    //                     (x + cx) as f64, 
    //                     y as f64, 
    //                     (z + cz) as f64
    //                 ]) > 0.01 {
    //
    //                     dbg!(&y);
    //                     self.set_block(x as usize, y as usize, z as usize, Some(1));
    //                 }
    //             }
    //         }
    //     }
    // }
    //
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

    pub fn build_mesh(&mut self) {
        self.mesh.clear();

        for x in 0..CHUNK_SIZE as usize {
            for y in 0..CHUNK_HEIGHT as usize {
                for z in 0..CHUNK_SIZE as usize {

                    if self.get_block(x, y, z).is_none() {
                        continue;
                    }

                    let fx = x as f32;
                    let fy = y as f32;
                    let fz = z as f32;

                    // 1. RIGHT FACE (+X)
                    if x == CHUNK_SIZE as usize - 1 || self.get_block(x + 1, y, z).is_none() {
                        self.mesh.extend_from_slice(&[
                            fx+1., fy+0., fz+0.,   fx+1., fy+1., fz+0.,   fx+1., fy+1., fz+1., 
                            fx+1., fy+1., fz+1.,   fx+1., fy+0., fz+1.,   fx+1., fy+0., fz+0.,
                        ]);
                    }
                    // 2. LEFT FACE (-X)
                    if x == 0 || self.get_block(x - 1, y, z).is_none() {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+0., fz+0.,   fx+0., fy+0., fz+1.,   fx+0., fy+1., fz+1., 
                            fx+0., fy+1., fz+1.,   fx+0., fy+1., fz+0.,   fx+0., fy+0., fz+0.,
                        ]);
                    }
                    // 3. TOP FACE (+Y)
                    if y == CHUNK_HEIGHT as usize - 1 || self.get_block(x, y + 1, z).is_none() {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+1., fz+0.,   fx+0., fy+1., fz+1.,   fx+1., fy+1., fz+1., 
                            fx+1., fy+1., fz+1.,   fx+1., fy+1., fz+0.,   fx+0., fy+1., fz+0.,
                        ]);
                    }
                    // 4. BOTTOM FACE (-Y)
                    if y == 0 || self.get_block(x, y - 1, z).is_none() {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+0., fz+0.,   fx+1., fy+0., fz+0.,   fx+1., fy+0., fz+1., 
                            fx+1., fy+0., fz+1.,   fx+0., fy+0., fz+1.,   fx+0., fy+0., fz+0.,
                        ]);
                    }
                    // 5. FRONT FACE (+Z)
                    if z == CHUNK_SIZE as usize - 1 || self.get_block(x, y, z + 1).is_none() {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+0., fz+1.,   fx+1., fy+0., fz+1.,   fx+1., fy+1., fz+1., 
                            fx+1., fy+1., fz+1.,   fx+0., fy+1., fz+1.,   fx+0., fy+0., fz+1.,
                        ]);
                    }
                    // 6. BACK FACE (-Z)
                    if z == 0 || self.get_block(x, y, z - 1).is_none() {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+0., fz+0.,   fx+0., fy+1., fz+0.,   fx+1., fy+1., fz+0., 
                            fx+1., fy+1., fz+0.,   fx+1., fy+0., fz+0.,   fx+0., fy+0., fz+0.,
                        ]);
                    }
                }
            }
        }
        self.dirty = false;
    }
}

pub type ChunkPosition = (i32, i32);
pub struct VoxelWorld {
    world: HashMap<ChunkPosition, Chunk>,
    pub last_mesh: Vec<f32>,
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self { world: HashMap::new(), last_mesh: Vec::new() }
    }
}

impl VoxelWorld {
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block_type: Option<BlockTypeId>) {
        let cx = (x as f32 / CHUNK_SIZE as f32).floor() as i32;
        let cz = (z as f32 / CHUNK_SIZE as f32).floor() as i32;

        let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;

        if y >= 0 && y < CHUNK_HEIGHT as i32 {
            let chunk = self.world.entry((cx, cz)).or_insert(Chunk::new(cx, cz));
            chunk.set_block(local_x, y as usize, local_z, block_type);
            chunk.dirty = true;
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<BlockTypeId> {
        let cx = (x / CHUNK_SIZE as usize) as i32;
        let cz = (z / CHUNK_SIZE as usize) as i32;
        
        let local_x = x % CHUNK_SIZE as usize;
        let local_z = z % CHUNK_SIZE as usize;

        if let Some(chunk) = self.world.get(&(cx, cz)) {
            return chunk.get_block(local_x, y, local_z)
        }

        None
    }

    pub fn nearby_chunk_mesh(&mut self, position: Vec3) -> Vec<f32> {
        let mut total_mesh = Vec::new();
        
        let p_cx = (position.x / CHUNK_SIZE as f32).floor() as i32;
        let p_cz = (position.z / CHUNK_SIZE as f32).floor() as i32;

        const RENDER_DISTANCE: i32 = 2;

        for cx in (p_cx - RENDER_DISTANCE)..=(p_cx + RENDER_DISTANCE) {
            for cz in (p_cz - RENDER_DISTANCE)..=(p_cz + RENDER_DISTANCE) {
                let key = (cx, cz);
                
                let chunk = self.world.entry(key).or_insert(Chunk::new(cx, cz));
                
                if chunk.dirty {
                    chunk.build_mesh();
                }

                let world_x_offset = cx as f32 * CHUNK_SIZE as f32;
                let world_z_offset = cz as f32 * CHUNK_SIZE as f32;

                for i in (0..chunk.mesh.len()).step_by(3) {
                    total_mesh.push(chunk.mesh[i] + world_x_offset);
                    total_mesh.push(chunk.mesh[i + 1]);
                    total_mesh.push(chunk.mesh[i + 2] + world_z_offset);
                }
            }
        }
        self.last_mesh = total_mesh.clone();
        total_mesh
    }
}
