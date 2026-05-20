use crate::chunk::{
    CHUNK_SIZE,
    CHUNK_PADDING,
};

#[allow(unused_imports)]
use {
    glam::{Vec2, Vec3, vec3},
    std::ops,
};

pub type ChunkPosition = Vec2;
pub type RelativePosition = Vec3;
pub type ChunkRelativePosition = Vec3;
#[derive(Debug)]
pub struct WorldPosition {
    pub world_position: RelativePosition,
}

impl WorldPosition {
    pub fn from_relative_pos(world_position: RelativePosition) -> Self {
        Self { world_position }
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { world_position: Vec3::new(x, y, z) }
    }

    pub fn chunk_pos(&self) -> ChunkPosition {
        ChunkPosition {
            x: (self.world_position.x / CHUNK_SIZE as f32).floor(),
            y: (self.world_position.z / CHUNK_SIZE as f32).floor(),
        }
    }

    pub fn chunk_rel_pos(&self) -> ChunkRelativePosition {
        let pad = (CHUNK_PADDING / 2) as f32;
        ChunkRelativePosition {
            x: self.world_position.x.rem_euclid(CHUNK_SIZE as f32) + pad,
            y: self.world_position.y,
            z: self.world_position.z.rem_euclid(CHUNK_SIZE as f32) + pad,
        }
    }
}

pub trait Formable3D {
    fn iterform(&self) -> (usize, usize, usize);
}

impl Formable3D for RelativePosition {
    fn iterform(&self) -> (usize, usize, usize) {
        (self.x as usize, self.y as usize, self.z as usize)
    }
}

pub trait Hashable {
    fn to_chunk_key(&self) -> (i32, i32);
}

impl Hashable for ChunkPosition {
    fn to_chunk_key(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}
