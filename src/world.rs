use crate::{
    Error,
    coordinate::{
        Hashable,
        ChunkPosition,
        WorldPosition,
    },
    chunk::{
        CHUNK_SIZE,
        CHUNK_HEIGHT,
        RENDER_DISTANCE,

        Chunk,
        BlockTypeId,
    },
};

use {
    std::collections::HashMap,
    glam::{
        Vec3,
        Mat4,
    },
    glow::{
        NativeUniformLocation,
        Context,
        HasContext,
    },
};

pub struct VoxelWorld {
    world: HashMap<(i32, i32), Chunk>,
    pub transform_loc: Option<NativeUniformLocation>,
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
    pub fn set_block(&mut self, pos: &WorldPosition, block_type: Option<BlockTypeId>) -> Result<(), Error> {
        // let cp = Self::chunk_pos(x, z);

        // let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        // let local_z = z.rem_euclid(CHUNK_SIZE as i32) as usize;
        // let local_x = (x % CHUNK_SIZE as i32) as usize;
        // let local_z = (z % CHUNK_SIZE as i32) as usize;

        if pos.relative_position.y >= 0 as f32 && pos.relative_position.y < CHUNK_HEIGHT as f32 {
            let chunk = self.world.get_mut(&pos.chunk_position.to_chunk_key()).ok_or("Chunk entry does not exist ({}, {}).")?;
            chunk.dirty = true;
            chunk.set_block(pos.relative_position, block_type);
        }

        Ok(())
    }

    pub fn get_block(&self, pos: &WorldPosition) -> Option<BlockTypeId> {
        // let cp = Self::chunk_pos(x as i32, z as i32);
        //
        // let local_x = (x % CHUNK_SIZE as i32) as usize;
        // let local_z = (z % CHUNK_SIZE as i32) as usize;

        if let Some(chunk) = self.world.get(&pos.chunk_position.to_chunk_key()) {
            return chunk.get_block(&pos.relative_position)
        }

        None
    }

    pub fn build_chunk(&mut self, gl: &Context, pos: &ChunkPosition) {
        self.world.insert(pos.to_chunk_key(), 
            Chunk::new(*pos, gl)
        );
    }

    pub fn try_build_nearby_chunks(&mut self, gl: &Context, pos: &WorldPosition) {
        for cx in (pos.chunk_position.x as i32 - RENDER_DISTANCE)..=(pos.chunk_position.x as i32 + RENDER_DISTANCE) {
            for cy in (pos.chunk_position.y as i32 - RENDER_DISTANCE)..=(pos.chunk_position.y as i32 + RENDER_DISTANCE) {
                let cp = ChunkPosition::new(cx as f32, cy as f32);

                match self.world.get_mut(&cp.to_chunk_key()) {
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
                        self.world.insert(cp.to_chunk_key(), new_chunk);
                    }
                }
            }
        }
    }

    pub fn render(&mut self, gl: &Context, position: &Vec3, mvp: &Mat4) {
        let p_cx = (position.x / CHUNK_SIZE as f32).floor() as i32;
        let p_cz = (position.z / CHUNK_SIZE as f32).floor() as i32;

        unsafe {
            gl.uniform_matrix_4_f32_slice(self.transform_loc.as_ref(), false, &mvp.to_cols_array());
        }

        for cx in (p_cx - RENDER_DISTANCE)..=(p_cx + RENDER_DISTANCE) {
            for cy in (p_cz - RENDER_DISTANCE)..=(p_cz + RENDER_DISTANCE) {
                let cp = ChunkPosition::new(cx as f32, cy as f32);

                // unsafe {
                //     gl.uniform_matrix_4_f32_slice(self.transform_loc.as_ref(), false, &mvp.to_cols_array());
                // }
                if let Some(chunk) = self.world.get_mut(&cp.to_chunk_key()) {
                    chunk.render(gl);
                }
            }
        }
    }
}
