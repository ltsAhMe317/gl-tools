use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync::LazyLock,
};

use glam::{Mat4, Quat, Vec3, Vec4, vec3};
use gltf::{
    Animation, Document, Node, Primitive, animation::util::ReadOutputs, buffer::Data,
    scene::iter::Children,
};
mod player;
mod model_load;
mod aabb;
pub use aabb::*;
pub use model_load::*;
pub use player::*;



