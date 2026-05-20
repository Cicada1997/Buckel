use crate::{
    limit,
    coordinate::{
        Formable3D,
        ChunkPosition,
        ChunkRelativePosition,
        WorldPosition,
    },
};

use {
    glow::{
        NativeVertexArray,
        Context,
        HasContext,
        NativeBuffer,
    },
};

pub struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn new(pos: (f32, f32, f32), uv: (f32, f32)) -> Self {
        Self {
            pos: [pos.0, pos.1, pos.2],
            uv: [uv.0, uv.1]
        }
    }

    pub fn relative(pos_std: &WorldPosition, pos: (f32, f32, f32), uv: (f32, f32)) -> Self {
        let world = pos_std.world_position;

        Self::new((
            pos.0 + world.x,
            pos.1 + world.y,
            pos.2 + world.z,
        ), uv)
    }

    pub fn to_slice(self) -> [f32; 5] {
        [self.pos[0], self.pos[1], self.pos[2], self.uv[0], self.uv[1]]
    }
}

pub static SEED:   u32 = 12938;
pub static CHUNK_SIZE:   u16 = 14;
pub static CHUNK_PADDING: u16 = 2;
pub static CHUNK_HEIGHT: u16 = 256;
pub static RENDER_DISTANCE: i32 = 16;
pub type ByteChunkLayer      = [u16; (CHUNK_SIZE + CHUNK_PADDING) as usize];
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
pub type BlockTypeId = usize;

// Z is bit position
impl Chunk {
    pub fn new(pos: ChunkPosition, gl: &Context) -> Self {
        let (vao, vbo) = unsafe {
            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();
            
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            
            // Define layout WHILE the VAO is bound
            // pos
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 20, 0);
            // uv texture coords
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 20, 12);

            gl.enable_vertex_attrib_array(0);
            gl.enable_vertex_attrib_array(1);
            
            (vao, vbo)
        };

        let mut chunk = Self { 
            blocks: [[[0u16; (CHUNK_SIZE + CHUNK_PADDING) as usize]; CHUNK_HEIGHT as usize]; BLOCK_TYPE_COUNT],
            mesh: Vec::new(),
            pos,
            vbo,
            vao,
            dirty: true
        };

        chunk.gen_perlin();
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

    pub(super) fn render(&self, gl: &Context) {
        let vertex_count = (self.mesh.len() / 3) as i32;
        if vertex_count == 0 { return; }
        
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, vertex_count);
        }
    }

    fn mesh_coord(&self, x: usize, y: usize, z: usize) -> WorldPosition {
        let pad = (CHUNK_PADDING / 2) as f32;
        WorldPosition::new(
            (x as f32 - pad) + CHUNK_SIZE as f32 * self.pos.x,
            y as f32,
            (z as f32 - pad) + CHUNK_SIZE as f32 * self.pos.y,
        )
    }

    pub(super) fn gen_mesh(&mut self) {
                
        let mut mesh = Vec::<f32>::new();

        for bytechunk in self.blocks.iter() {
            for y in 0..CHUNK_HEIGHT as usize {
                let layer = bytechunk[y];

                for x in 1..=CHUNK_SIZE as usize {
                    let row = layer[x];

                    // (-Z) x-shift right 
                    let render_scheme = (!row & (row >> 1)) << 1;
                    for z in 1..=CHUNK_SIZE as usize {
                        let pos = self.mesh_coord(x, y, z);
                        if render_scheme >> z & 1 == 1 {
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 0., 0.), (0., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 1., 0.), (0., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 0.), (1., 1.)).to_slice());

                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 0.), (1., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 0.), (1., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 0., 0.), (0., 0.)).to_slice());
                        }
                    }

                    // (+Z) x-shift left
                    let render_scheme = (!row & (row << 1)) >> 1;
                    for z in 1..=CHUNK_SIZE as usize {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x, y, z);

                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 0., 1.), (0., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 1.), (0., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 1.), (1., 1.)).to_slice());

                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 1.), (1., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 1., 1.), (1., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 0., 1.), (0., 0.)).to_slice());
                        }
                    }

                    // (+X) z-shift right
                    let left_row = layer[(x+1).clamp(0, 15)];
                    let render_scheme = row & !left_row;
                    for z in 1..=CHUNK_SIZE as usize {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x, y, z);
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 0.), (0., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 0.), (0., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 1.), (1., 1.)).to_slice());

                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 1.), (1., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 1.), (1., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 0.), (0., 0.)).to_slice());
                        }
                    }

                    // (-X) z-shift left
                    let right_row = layer[x.saturating_sub(1)];
                    let render_scheme = row & !right_row;
                    for z in 1..=CHUNK_SIZE as usize {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x-1, y, z);
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 0.), (0., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 0.), (0., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 1.), (1., 1.)).to_slice());

                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 1.), (1., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 1.), (1., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 0.), (0., 0.)).to_slice());
                        }
                    }

                    let a = [16u16; (CHUNK_SIZE + CHUNK_PADDING) as usize];
                    let next_layer = bytechunk.get(y+1).unwrap_or(&a);
                    let next_layer_row = next_layer[x];

                    // (+Y)
                    let render_scheme = row & !next_layer_row;
                    for z in 1..=CHUNK_SIZE as usize {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x, y, z);

                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 1., 0.), (0., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 1., 1.), (0., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 1.), (1., 1.)).to_slice());

                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 1.), (1., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 1., 0.), (1., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 1., 0.), (0., 0.)).to_slice());
                        }
                    }

                    let past_layer = bytechunk.get(y.saturating_sub(1)).unwrap_or(&a);
                    let past_layer_row = past_layer[x];
                    let render_scheme = row & !past_layer_row;
                    for z in 1..=CHUNK_SIZE as usize {
                        if render_scheme >> z & 1 == 1 {
                            let pos = self.mesh_coord(x, y, z);

                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 0., 0.), (0., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 0., 1.), (0., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 1.), (1., 1.)).to_slice());

                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 1.), (1., 1.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (1., 0., 0.), (1., 0.)).to_slice());
                            mesh.extend_from_slice(&Vertex::relative(&pos, (0., 0., 0.), (0., 0.)).to_slice());

                        }
                    }
                }
            }
        }

        self.mesh = mesh;

        self.dirty = false;
    }


    fn gen_perlin(&mut self) {
        use noise::{NoiseFn, Perlin};

        self.dirty = true;
        let perlin = Perlin::new(SEED);
        let perlin2 = Perlin::new(SEED / 2);

        let frequency = 0.03; 
        let terrain_height_multiplier = 20.0;
        let sea_level = 10.0;

        for x in 0..(CHUNK_SIZE + CHUNK_PADDING) as i32 {
            for z in 0..(CHUNK_SIZE + CHUNK_PADDING) as i32 {

                let world_x = (x as f32 + self.pos.x * CHUNK_SIZE as f32) as f64;
                let world_z = (z as f32 + self.pos.y * CHUNK_SIZE as f32) as f64;

                let noise_val = perlin.get([world_x * frequency, world_z * frequency]) + perlin2.get([world_x * frequency, world_z * frequency]);
                let height = ((noise_val + 1.0) * 0.5 * terrain_height_multiplier + sea_level) as i32;

                for y in 0..CHUNK_HEIGHT as i32 {
                    if y <= height {
                        self.set_block(ChunkRelativePosition::new(x as f32, y as f32, z as f32), Some(1));
                    }
                }
            }
        }
    }

    pub fn get_block(&self, pos: &ChunkRelativePosition) -> Option<BlockTypeId> {
        let (x, y, z) = pos.iterform();

        limit!(x, (CHUNK_SIZE + CHUNK_PADDING) as usize,   "chunk relative position outside chunk");
        limit!(y, CHUNK_HEIGHT as usize, "chunk relative position outside chunk");
        limit!(z, (CHUNK_SIZE + CHUNK_PADDING) as usize,   "chunk relative position outside chunk");

        for (block_type_id, bytechunk) in self.blocks.iter().enumerate() {
            if (bytechunk[y][x] >> z) & 1 == 1 {
                return Some(block_type_id);
            }
        }

        None
    }

    pub fn set_block(&mut self, pos: ChunkRelativePosition, block_type: Option<BlockTypeId>) {
        let (x, y, z) = pos.iterform();
        limit!(x, (CHUNK_SIZE + CHUNK_PADDING) as usize,   "chunk relative position outside chunk");
        limit!(y, CHUNK_HEIGHT as usize, "chunk relative position outside chunk");
        limit!(z, (CHUNK_SIZE + CHUNK_PADDING) as usize,   "chunk relative position outside chunk");

        if let Some(block_type) = block_type {
            self.blocks[block_type][y][x] = self.blocks[block_type][y][x] | (1 << z);
        } else {
            for bytechunk in self.blocks.iter_mut() {
                bytechunk[y][x] = bytechunk[y][x] & !(1 << z+1);
            }
        }

        self.dirty = true;
    }
}
