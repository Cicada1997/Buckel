use crate::{
    chunk::{
        CHUNK_SIZE,
    },
};

#[allow(unused_imports)]
use {
    std::ops,
    glam::{
        Vec2,
        vec3,
        Vec3,
    },
};
pub type ChunkPosition = Vec2;
pub type RelativePosition = Vec3;
pub type ChunkRelativePosition = Vec3;
pub struct WorldPosition {
    pub chunk_position: ChunkPosition,
    pub relative_position: ChunkRelativePosition,
}

impl WorldPosition {
    pub fn from_world_xyz(world_pos: (f32, f32, f32)) -> Self {
        Self { 
            chunk_position: ChunkPosition {
                x: world_pos.0.div_euclid(CHUNK_SIZE as f32),
                y: world_pos.1.div_euclid(CHUNK_SIZE as f32),
            }, 
            relative_position: RelativePosition {
                x: world_pos.0 % CHUNK_SIZE as f32,
                y: world_pos.1 % CHUNK_SIZE as f32,
                z: world_pos.2 % CHUNK_SIZE as f32,
            }
        }
    }
    pub fn from_world_pos(world_pos: &RelativePosition) -> Self {
        Self::from_world_xyz((*world_pos).into())
    }

    pub fn world(&self) -> RelativePosition {
        RelativePosition {
            x: self.world_x(),
            y: self.relative_position.y,
            z: self.world_z(),
        }
    }

    pub fn world_x(&self) -> f32 {
        self.relative_position.x + (self.chunk_position.x * CHUNK_SIZE as f32)
    }

    pub fn world_y(&self) -> f32 {
        self.relative_position.y
    }

    pub fn world_z(&self) -> f32 {
        self.relative_position.z + (self.chunk_position.y * CHUNK_SIZE as f32)
    }
}

// impl ops::Add<ChunkPosition> for RelativePosition {
//     type Output = WorldPosition;
//
//     fn add(self, rhs: ChunkPosition) -> Self::Output {
//         return WorldPosition {
//             relative_position: self,
//             chunk_position: rhs,
//         }
//     }
// }

pub trait Formable3D {
    fn iterform(&self) -> (usize, usize, usize);
}

impl Formable3D for RelativePosition {
    fn iterform(&self) -> (usize, usize, usize) {
        ( self.x as usize, self.y as usize, self.z as usize )
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

