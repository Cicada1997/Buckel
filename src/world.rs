use crate::{
    Error,
    coordinate::{
        Hashable,
        ChunkPosition,
        WorldPosition,
    },
    chunk::{
        RENDER_DISTANCE,
        CHUNK_HEIGHT,
        Chunk,
        BlockTypeId,
    },
};

use {
    std::collections::HashMap,
    glam::{
        Mat4,
    },
    glow::{
        NativeUniformLocation,
        Context,
        HasContext,
    },
};

#[derive(Default)]
pub struct VoxelWorld {
    world: HashMap<(i32, i32), Chunk>,
    pub transform_loc: Option<NativeUniformLocation>,
}

impl VoxelWorld {
    pub fn set_block(&mut self, pos: &WorldPosition, block_type: Option<BlockTypeId>) -> Result<(), Error> {
        if pos.world_position.y >= 0. && ((pos.world_position.y as u16) < CHUNK_HEIGHT) {
            let chunk = self.world.get_mut(&pos.chunk_pos().to_chunk_key()).ok_or("Chunk entry does not exist ({}, {}).")?;
            chunk.dirty = true;
            chunk.set_block(pos.chunk_rel_pos(), block_type);
        }

        Ok(())
    }

    pub fn get_block(&self, pos: &WorldPosition) -> Option<BlockTypeId> {
        if let Some(chunk) = self.world.get(&pos.chunk_pos().to_chunk_key()) {
            return chunk.get_block(&pos.chunk_rel_pos())
        }

        None
    }

    pub fn build_chunk(&mut self, gl: &Context, pos: &ChunkPosition) {
        self.world.insert(pos.to_chunk_key(), 
            Chunk::new(*pos, gl)
        );
    }

    pub fn try_build_nearby_chunks(&mut self, gl: &Context, pos: &WorldPosition) {
        let center_cp = pos.chunk_pos();

        for cx in (center_cp.x as i32 - RENDER_DISTANCE)..=(center_cp.x as i32 + RENDER_DISTANCE) {
            for cy in (center_cp.y as i32 - RENDER_DISTANCE)..=(center_cp.y as i32 + RENDER_DISTANCE) {
                let cp = ChunkPosition::new(cx as f32, cy as f32);

                match self.world.get_mut(&cp.to_chunk_key()) {
                    Some(chunk) => {
                        if chunk.dirty {
                            chunk.gen_mesh();
                            chunk.upload_mesh(gl);
                        }
                    }
                    Option::None => {
                        let mut new_chunk = Chunk::new(cp, gl);
                        new_chunk.gen_mesh();
                        new_chunk.upload_mesh(gl);
                        self.world.insert(cp.to_chunk_key(), new_chunk);
                    }
                }
            }
        }
    }

    pub fn render(&mut self, gl: &Context, position: &WorldPosition, mvp: &Mat4) {
        let p_cp = position.chunk_pos();

        unsafe {
            gl.uniform_matrix_4_f32_slice(self.transform_loc.as_ref(), false, &mvp.to_cols_array());
        }

        for cx in (p_cp.x as i32 - RENDER_DISTANCE)..=(p_cp.x as i32 + RENDER_DISTANCE) {
            for cy in (p_cp.y as i32 - RENDER_DISTANCE)..=(p_cp.y as i32 + RENDER_DISTANCE) {
                let cp = ChunkPosition::new(cx as f32, cy as f32);

                if let Some(chunk) = self.world.get_mut(&cp.to_chunk_key()) {
                    chunk.render(gl);
                }
            }
        }
    }
}
