use std::{collections::HashMap, fs, path::Path};

use glam::Mat4;
use gltf::{Document, Node, buffer::Data};

use crate::{
    draws::model::Player,
    gl_unit::{
        define::{TextureParm, TextureType},
        texture::{Texture2D, TextureWrapper},
    },
};

fn node_mat4(node: &Node) -> Mat4 {
    Mat4::from_cols_array_2d(&node.transform().matrix())
}

fn node_path(root: &Node, target: &Node) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    if root.index() == target.index() {
        return Some(path);
    } else {
        for (id, child) in root.children().enumerate() {
            if let Some(next) = node_path(&child, target) {
                path.push(id);
                path.extend(next);
                return Some(path);
            }
        }
    }
    None
}
// fn global_mat<'a>(root: &'a Node, target: &'a Node, change: &HashMap<usize, Mat4>) -> Mat4 {
//     if root.index() == target.index() {
//         let mut mat = node_mat4(root);
//         // 将offset作为相对变换应用到原始矩阵上
//         if let Some(offset) = change.get(&root.index()) {
//             mat = mat * *offset;  // 或者 *offset * mat，取决于顺序
//         }
//         return mat;
//     }

//     let path = node_path(root, target).unwrap_or_default();
//     let mut mat = node_mat4(root);
//     let mut node = root.clone();

//     for index in path {
//         node = node.children().nth(index).unwrap();
//         let mut node_mat = node_mat4(&node);

//         // 应用offset作为相对变换
//         if let Some(offset) = change.get(&node.index()) {
//             node_mat = node_mat * *offset;  // 关键修复！
//         }

//         mat = mat * node_mat;
//     }

//     mat
// }
fn global_mat<'a>(root: &'a Node, target: &'a Node, change: &HashMap<usize, Mat4>) -> Mat4 {
    if root.index() == target.index() {
        return change
            .get(&root.index())
            .copied()
            .unwrap_or(node_mat4(root));
    }
    let path = node_path(root, target);
    if path.is_none() {
        return change.get(&target.index()).map(|mat|{*mat}).unwrap_or(node_mat4(target));
    }
    let path = path.unwrap();
    let mut mat = change
        .get(&root.index())
        .map(|mat| *mat)
        .unwrap_or(node_mat4(root));
    let mut node = root.clone();
    for index in path {
        node = node.children().nth(index).unwrap();
        mat = mat
            * change
                .get(&node.index())
                .map(|mat| *mat)
                .unwrap_or(node_mat4(&node));
    }
    mat
}

pub struct Model {
    pub data: Vec<Data>,
    pub document: Document,
    pub texs: Vec<TextureWrapper<Texture2D>>,
}

impl Model {
    pub fn player(&'_ self) -> Player<'_> {
        Player::new(self)
    }
    pub fn joint_mat(&self, node: &Node) -> Option<Vec<Mat4>> {
        self.joint_mat_change(node, &HashMap::new())
    }
    pub fn joint_mat_change(
        &self,
        node: &Node,
        change: &HashMap<usize, Mat4>,
    ) -> Option<Vec<Mat4>> {
        let skin = node.skin()?;
        let node_global_mat = self.global_mat(node);
        let inverse_mat: Vec<Mat4> = skin
            .reader(|index| Some(&self.data[index.index()]))
            .read_inverse_bind_matrices()?
            .map(|mat| Mat4::from_cols_array_2d(&mat))
            .collect();
        let global_mat: Vec<Mat4> = skin
            .joints()
            .map(|joint| self.global_mat_change(&joint, &change))
            .collect();
        let joint_mat: Vec<Mat4> = inverse_mat
            .iter()
            .zip(global_mat.iter())
            .map(|(inverse, global)|  global * inverse)
            .collect();

        Some(joint_mat)
    }
    pub fn global_mat(&self, node: &Node) -> Mat4 {
        self.global_mat_change(node, &HashMap::new())
    }
    pub fn global_mat_change(&self, node: &Node, change: &HashMap<usize, Mat4>) -> Mat4 {
        global_mat(
            &self
                .document
                .default_scene()
                .unwrap()
                .nodes()
                .next()
                .unwrap(),
            node,
            change,
        )
    }
    pub fn from_path(p: impl AsRef<Path>) -> Self {
        Self::from_buffer(fs::read(p).unwrap())
    }
    pub fn from_buffer(buffer: impl AsRef<[u8]>) -> Self {
        let (document, buffers, images) = gltf::import_slice(buffer.as_ref()).unwrap();
        let textures = images
            .into_iter()
            .map(|data| {
                let tex_type = match data.format {
                    gltf::image::Format::R8 => TextureType::RED8,
                    gltf::image::Format::R8G8 | gltf::image::Format::R16G16 => todo!(),
                    gltf::image::Format::R8G8B8 => TextureType::RGB8,
                    gltf::image::Format::R8G8B8A8 => TextureType::RGBA8,
                    gltf::image::Format::R16 => TextureType::RED16,
                    gltf::image::Format::R16G16B16 => TextureType::RGB16,
                    gltf::image::Format::R16G16B16A16 => TextureType::RGBA16,
                    gltf::image::Format::R32G32B32FLOAT => TextureType::RGB32,
                    gltf::image::Format::R32G32B32A32FLOAT => TextureType::RGBA32,
                };
                TextureWrapper(Texture2D::load(
                    Some(data.pixels.as_slice()),
                    tex_type,
                    data.width,
                    data.height,
                    if (data.width) % 2 != 0 {
                        TextureParm::new().once_load_size(1)
                    } else {
                        TextureParm::new()
                    },
                ))
            })
            .collect::<Vec<TextureWrapper<Texture2D>>>();
        Self {
            texs: textures,
            document,
            data: buffers,
        }
    }
}
