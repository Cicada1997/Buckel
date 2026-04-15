use crate::{
    BLOCK_TYPE_COUNT, limit
};

use {
    std::collections::HashMap,
    glam::Vec3,
};

pub static SEED:   u32 = 12938;
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

/// Z is bit position
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
        let perlin = Perlin::new(SEED);

        let frequency = 0.03; 
        let terrain_height_multiplier = 20.0;
        let sea_level = 10.0;

        for x in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                let world_x = (x + cx * CHUNK_SIZE as i32) as f64;
                let world_z = (z + cz * CHUNK_SIZE as i32) as f64;

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
    pub last_mesh: Vec<f32>,
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self { world: HashMap::new(), last_mesh: Vec::new() }
    }
}

impl VoxelWorld {
    pub fn build_chunk_mesh(&mut self, cp: ChunkPosition) {
        let chunk = self.world.entry(cp).or_insert(Chunk::new(cp.0, cp.1));
                
        chunk.mesh.clear();

        for (_block_type_id, bytechunk) in chunk.blocks.iter().enumerate() {
            for y in 0..CHUNK_HEIGHT as usize {
                let layer = bytechunk[y];

                for x in 0..CHUNK_SIZE as usize {
                    let row = layer[x];

                    // (-Z) x-shift right 
                    let render_scheme = (!row & (row >> 1)) << 1;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let fx = x as f32;
                            let fy = y as f32;
                            let fz = z as f32;

                            chunk.mesh.extend_from_slice(&[
                                fx+0., fy+0., fz+0.,   fx+0., fy+1., fz+0.,   fx+1., fy+1., fz+0., 
                                fx+1., fy+1., fz+0.,   fx+1., fy+0., fz+0.,   fx+0., fy+0., fz+0.,
                            ]);
                        }
                    }

                    // (+Z) x-shift left
                    let render_scheme = (!row & (row << 1)) >> 1;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let fx = x as f32;
                            let fy = y as f32;
                            let fz = z as f32;

                            chunk.mesh.extend_from_slice(&[
                                fx+0., fy+0., fz+1.,   fx+1., fy+0., fz+1.,   fx+1., fy+1., fz+1., 
                                fx+1., fy+1., fz+1.,   fx+0., fy+1., fz+1.,   fx+0., fy+0., fz+1.,
                            ]);
                        }
                    }

                    // (+X) z-shift right
                    let left_row = layer[(x+1).clamp(0, 15)];
                    let render_scheme = row & !left_row;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let fx = x as f32;
                            let fy = y as f32;
                            let fz = z as f32;
                            chunk.mesh.extend_from_slice(&[
                                fx+1., fy+0., fz+0.,   fx+1., fy+1., fz+0.,   fx+1., fy+1., fz+1., 
                                fx+1., fy+1., fz+1.,   fx+1., fy+0., fz+1.,   fx+1., fy+0., fz+0.,
                            ]);
                        }
                    }

                    // (-X) z-shift left
                    let right_row = layer[x.checked_sub(1).unwrap_or(0)];
                    let render_scheme = row & !right_row;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let fx = x as f32 - 1.;
                            let fy = y as f32;
                            let fz = z as f32;
                            chunk.mesh.extend_from_slice(&[
                                fx+1., fy+0., fz+0.,   fx+1., fy+1., fz+0.,   fx+1., fy+1., fz+1., 
                                fx+1., fy+1., fz+1.,   fx+1., fy+0., fz+1.,   fx+1., fy+0., fz+0.,
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
                            let fx = x as f32;
                            let fy = y as f32;
                            let fz = z as f32;

                            chunk.mesh.extend_from_slice(&[
                                fx+0., fy+1., fz+0.,   fx+0., fy+1., fz+1.,   fx+1., fy+1., fz+1., 
                                fx+1., fy+1., fz+1.,   fx+1., fy+1., fz+0.,   fx+0., fy+1., fz+0.,
                            ]);
                        }
                    }

                    let past_layer = bytechunk.get(y.checked_sub(1).unwrap_or(0)).unwrap_or(&a);
                    let past_layer_row = past_layer[x];
                    let render_scheme = row & !past_layer_row;
                    for z in 0..CHUNK_SIZE {
                        if render_scheme >> z & 1 == 1 {
                            let fx = x as f32;
                            let fy = y as f32;
                            let fz = z as f32;

                            chunk.mesh.extend_from_slice(&[
                                fx+0., fy+0., fz+0.,   fx+0., fy+0., fz+1.,   fx+1., fy+0., fz+1., 
                                fx+1., fy+0., fz+1.,   fx+1., fy+0., fz+0.,   fx+0., fy+0., fz+0.,
                            ]);
                        }
                    }
                }
            }
        }
        chunk.dirty = false
    }

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

        const RENDER_DISTANCE: i32 = 4;

        for cx in (p_cx - RENDER_DISTANCE)..=(p_cx + RENDER_DISTANCE) {
            for cz in (p_cz - RENDER_DISTANCE)..=(p_cz + RENDER_DISTANCE) {
                let cp = (cx, cz);
                
                self.build_chunk_mesh(cp);
                let chunk = self.world.get(&cp).expect("Building chunk failed; could not get chunk after meshbuild (should assure chunk exists).");

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
        // println!("vertecies: {}", total_mesh.len());
        total_mesh
    }
}
