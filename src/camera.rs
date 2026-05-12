use crate::WorldPosition;

use {
    glam::{ Mat4, Vec3, vec3 },
};

pub struct Camera {
    pub angle:  Vec3,
    pub pos:    WorldPosition,
}


impl Camera {
    pub fn update_view(&self, view: &mut Mat4) {
        *view = Mat4::look_at_rh(self.pos.world_position, self.pos.world_position + self.front(), Vec3::Y);
    }

    // set
    pub fn set_pitch(&mut self) {

    }

    pub fn set_jaw(&mut self) {

    }

    // add
    pub fn add_pitch(&mut self) {

    }

    pub fn add_jaw(&mut self) {

    }

    pub fn front(&self) -> Vec3 {
        vec3(
            self.angle.y.cos() * self.angle.x.cos(),
            self.angle.x.sin(),
            self.angle.y.sin() * self.angle.x.cos(),
        ).normalize()
    }

    pub fn flat_front(&self) -> Vec3 {
        vec3(
            self.angle.y.cos() * self.angle.x.cos(),
            0.,
            self.angle.y.sin() * self.angle.x.cos(),
        ).normalize()
    }
    
}
