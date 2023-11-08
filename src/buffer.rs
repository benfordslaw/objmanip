use glium::{index::BufferCreationError, Display, IndexBuffer, VertexBuffer};
use glutin::surface::WindowSurface;

use crate::load::ObjVertex;

pub struct DisplayVertexBuffer<'a>(
    pub &'a VertexBuffer<ObjVertex>,
    pub &'a Display<WindowSurface>,
);

/// uses `LinesList` because if the indices aren't supplied, we probably only have a path
impl TryFrom<DisplayVertexBuffer<'_>> for IndexBuffer<u32> {
    type Error = BufferCreationError;
    fn try_from(d_vb: DisplayVertexBuffer) -> Result<IndexBuffer<u32>, BufferCreationError> {
        let index_vec: Vec<u32> = (0u32..u32::try_from(d_vb.0.len()).unwrap().saturating_sub(1))
            .flat_map(|idx| [idx, idx + 1])
            .collect();
        glium::IndexBuffer::new(d_vb.1, glium::index::PrimitiveType::LinesList, &index_vec)
    }
}
