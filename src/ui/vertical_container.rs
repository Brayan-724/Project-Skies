use glyphon::{FontSystem, TextArea};
use wgpu::Color;

use crate::{app::Size, rendering::vertex::VertexUi};

use super::{ui_node::{UiNode, UiNodeParameters}, ui_transform::Rect};

/// # Vertical Container
/// This struct will be designed for "rendering listed data" like for example, subtitles in a certain order (in this case on a vertical one)
/// while respecting elements  like margin or separation between all of them.

pub struct VerticalContainerData {
    pub margin: f32,
    pub separation: f32,
    pub children: Vec<UiNode>
}

impl VerticalContainerData {
    pub fn new(margin: f32, separation: f32, children: Vec<UiNode>) -> Self {
        Self {
            margin,
            separation,
            children,
        }
    }

    // this function will do the positioning of all the elements in the screen based on their separation
    pub fn ui_node_data_creation(&self, size: &Size, vertices: &mut Vec<VertexUi>, vertices_slice: &[VertexUi; 4], indices: &mut Vec<u16>, indices_slice: &[u16; 6]) -> (u16, u32) {
        vertices.extend_from_slice(vertices_slice);
        indices.extend_from_slice(indices_slice); 

        (vertices_slice.len() as u16, UiNode::NUM_INDICES)
    }
}