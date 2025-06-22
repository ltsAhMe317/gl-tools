use core::panic;
use std::path::Path;

use glam::{Vec2, Vec3};

use crate::gl_unit::texture;

// pub struct Model {
// }
// impl Model {
//     pub fn load_path(path: impl AsRef<Path>) -> Self {
//         let scene = Scene::from_file(
//             path.as_ref().to_str().unwrap(),
//             vec![
//                 PostProcess::MakeLeftHanded,
//                 PostProcess::Triangulate,
//                 PostProcess::JoinIdenticalVertices,
//                 PostProcess::ImproveCacheLocality,
//                 PostProcess::OptimizeMeshes,
//                 PostProcess::OptimizeGraph,
//             ],
//         );
//         let scene = scene.unwrap();
//         dbg!(&scene);
//         Self { scene }
//     }
//     pub fn draw(&self) {}
// }

#[cfg(test)]
mod tests {

    #[test]
    pub fn load_model() {}
}
