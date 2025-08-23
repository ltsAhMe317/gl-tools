use glam::{vec3, Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}
impl AABB {
    pub fn new() -> Self {
        Self {
            min_x: 0f32,
            min_y: 0f32,
            min_z: 0f32,
            max_x: 0f32,
            max_y: 0f32,
            max_z: 0f32,
        }
    }
    pub fn new_vec(min: Vec3, max: Vec3) -> Self {
        Self {
            min_x: min.x,
            min_y: min.y,
            min_z: min.z,
            max_x: max.x,
            max_y: max.y,
            max_z: max.z,
        }
    }
    pub fn min_pos(&self) -> Vec3 {
        vec3(self.min_x, self.min_y, self.min_z)
    }
    pub fn max_pos(&self) -> Vec3 {
        vec3(self.max_x, self.max_y, self.max_z)
    }
    pub fn transfrom(&self, mat: Mat4) -> Self {
        let min = self.min_pos();
        let max = self.max_pos();
        Self::new_vec(mat.transform_point3(min), mat.transform_point3(max))
    }
    fn update(&mut self, x: f32, y: f32, z: f32) {
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
        if z > self.max_z {
            self.max_z = z;
        }

        if x < self.min_x {
            self.min_x = x;
        }
        if y < self.min_y {
            self.min_y = y;
        }
        if z < self.min_z {
            self.min_z = z;
        }
    }
    pub fn as_vertexs(&self) -> [f32; 3 * 4 * 6] {
        let (min_x, min_y, min_z, max_x, max_y, max_z) = (
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z,
        );
        //痛苦面具
        //手打24个顶点66这是左边的      //2                  //3                     //4
        [
            min_x, min_y, min_z, min_x, max_y, min_z, min_x, max_y, max_z, min_x, min_y, max_z,
            //右边
            max_x, min_y, min_z, max_x, max_y, min_z, max_x, max_y, max_z, max_x, min_y, max_z,
            //上
            min_x, max_y, min_z, max_x, max_y, min_z, max_x, max_y, max_z, min_x, max_y, max_z,
            //下
            min_x, min_y, min_z, max_x, min_y, min_z, max_x, min_y, max_z, min_x, min_y, max_z,
            //后
            min_x, max_y, min_z, max_x, max_y, min_z, max_x, min_y, min_z, min_x, min_y, min_z,
            //前
            min_x, max_y, max_z, max_x, max_y, max_z, max_x, min_y, max_z, min_x, min_y, max_z,
        ]
    }
    pub fn is_touch(&self, other: &Self) -> bool {
        let x_touch = self.max_x > other.min_x && other.max_x > self.min_x;
        let y_touch = self.max_y > other.min_y && other.max_y > self.min_y;
        let z_touch = self.max_z > other.min_z && other.max_z > self.min_z;
        x_touch && y_touch && z_touch
    }
}
