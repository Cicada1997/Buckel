use crate::{
    limit
};

pub static CHUNK_SIZE:   u16 = 16;
pub static CHUNK_HEIGHT: u16 = 64;
pub type ByteChunkLayer      = [u16;            CHUNK_SIZE   as usize];
pub type ByteChunk           = [ByteChunkLayer; CHUNK_HEIGHT as usize];

pub struct Chunk {
    blocks: ByteChunk,
    pub mesh: Vec<f32>,
    pub dirty: bool,
}

impl Default for Chunk {
    fn default() -> Self {
        let mut blocks = [[0u16; CHUNK_SIZE as usize]; CHUNK_HEIGHT as usize];

        for y in 0..5 {
            blocks[y].fill(0b_1111_1111_1111_1111);
        }

        Self { 
            blocks,
            mesh: Vec::new(),
            dirty: true
        }
    }
}

impl Chunk {
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> bool {
        limit!(x, CHUNK_SIZE as usize,   "chunk relative position outside chunk");
        limit!(y, CHUNK_HEIGHT as usize, "chunk relative position outside chunk");
        limit!(z, CHUNK_SIZE as usize,   "chunk relative position outside chunk");

        (self.blocks[y][x] >> z) & 1 == 1
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, val: bool) {
        limit!(x, CHUNK_SIZE as usize,   "chunk relative position outside chunk");
        limit!(y, CHUNK_HEIGHT as usize, "chunk relative position outside chunk");
        limit!(z, CHUNK_SIZE as usize,   "chunk relative position outside chunk");

        if val {
            self.blocks[y][x] = self.blocks[y][x] | (1 << z);
        } else {
            self.blocks[y][x] = self.blocks[y][x] & !(1 << z);
        }
        self.dirty = true;
    }

    pub fn toggle_block(&mut self, x: usize, y: usize, z: usize) {
        limit!(x, CHUNK_SIZE as usize,   "chunk relative position outside chunk");
        limit!(y, CHUNK_HEIGHT as usize, "chunk relative position outside chunk");
        limit!(z, CHUNK_SIZE as usize,   "chunk relative position outside chunk");

        self.blocks[y][x] = self.blocks[y][x] ^ (1 << z);
        self.dirty = true;
    }

    pub fn build_mesh(&mut self) {
        self.mesh.clear();

        for x in 0..CHUNK_SIZE as usize {
            for y in 0..CHUNK_HEIGHT as usize {
                for z in 0..CHUNK_SIZE as usize {

                    if !self.get_block(x, y, z) {
                        continue; 
                    }

                    let fx = x as f32;
                    let fy = y as f32;
                    let fz = z as f32;

                    // 1. RIGHT FACE (+X)
                    if x == CHUNK_SIZE as usize - 1 || !self.get_block(x + 1, y, z) {
                        self.mesh.extend_from_slice(&[
                            fx+1., fy+0., fz+0.,   fx+1., fy+1., fz+0.,   fx+1., fy+1., fz+1., 
                            fx+1., fy+1., fz+1.,   fx+1., fy+0., fz+1.,   fx+1., fy+0., fz+0.,
                        ]);
                    }
                    // 2. LEFT FACE (-X)
                    if x == 0 || !self.get_block(x - 1, y, z) {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+0., fz+0.,   fx+0., fy+0., fz+1.,   fx+0., fy+1., fz+1., 
                            fx+0., fy+1., fz+1.,   fx+0., fy+1., fz+0.,   fx+0., fy+0., fz+0.,
                        ]);
                    }
                    // 3. TOP FACE (+Y)
                    if y == CHUNK_HEIGHT as usize - 1 || !self.get_block(x, y + 1, z) {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+1., fz+0.,   fx+0., fy+1., fz+1.,   fx+1., fy+1., fz+1., 
                            fx+1., fy+1., fz+1.,   fx+1., fy+1., fz+0.,   fx+0., fy+1., fz+0.,
                        ]);
                    }
                    // 4. BOTTOM FACE (-Y)
                    if y == 0 || !self.get_block(x, y - 1, z) {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+0., fz+0.,   fx+1., fy+0., fz+0.,   fx+1., fy+0., fz+1., 
                            fx+1., fy+0., fz+1.,   fx+0., fy+0., fz+1.,   fx+0., fy+0., fz+0.,
                        ]);
                    }
                    // 5. FRONT FACE (+Z)
                    if z == CHUNK_SIZE as usize - 1 || !self.get_block(x, y, z + 1) {
                        self.mesh.extend_from_slice(&[
                            fx+0., fy+0., fz+1.,   fx+1., fy+0., fz+1.,   fx+1., fy+1., fz+1., 
                            fx+1., fy+1., fz+1.,   fx+0., fy+1., fz+1.,   fx+0., fy+0., fz+1.,
                        ]);
                    }
                    // 6. BACK FACE (-Z)
                    if z == 0 || !self.get_block(x, y, z - 1) {
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
