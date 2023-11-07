use glium::{vertex::BufferCreationError, Display, VertexBuffer};
use glutin::surface::WindowSurface;
use petgraph::{algo, graph, Graph};

use crate::load::ObjVertex;

pub struct VertexGraph {
    pub graph: Graph<ObjVertex, f32>,
}

impl VertexGraph {
    pub fn new(g: Graph<ObjVertex, f32>) -> Self {
        Self { graph: g }
    }

    /// Returns a `VertexBuffer` containing the `ObjVertex` node weights from this `VertexGraph`
    pub fn to_buffer(
        &self,
        display: &Display<WindowSurface>,
    ) -> Result<VertexBuffer<ObjVertex>, BufferCreationError> {
        VertexBuffer::new(
            display,
            &self
                .graph
                .node_weights()
                .cloned()
                .collect::<Vec<ObjVertex>>(),
        )
    }

    /// Returns a `VertexBuffer` containing only the `ObjVertex` node weights from this
    /// `VertexGraph` along the longest continuous path from `start_idx`
    pub fn continuous_path_from(
        &self,
        start_idx: u32,
        display: &Display<WindowSurface>,
    ) -> Result<VertexBuffer<ObjVertex>, BufferCreationError> {
        let start_node = graph::NodeIndex::from(start_idx);
        let bellman_ford = algo::bellman_ford(&self.graph, start_node).unwrap();
        let mut prev = graph::NodeIndex::from(start_idx);

        let mut path_vertices = vec![
            ObjVertex {
                position: [0.0; 3],
                normal: [0.0; 3],
                texture: [-1.0; 2],
            };
            self.graph.node_count()
        ];

        // parse the predecessors field to step along the path from `start_idx`
        // TODO: explain `idx` and `predecessor` relationship
        while let Some((idx, _)) = bellman_ford
            .predecessors
            .iter()
            .enumerate()
            .find(|(_, &predecessor)| predecessor == Some(prev))
        {
            *path_vertices.get_mut(prev.index()).unwrap() = *self.graph.node_weight(prev).unwrap();
            prev = graph::NodeIndex::from(idx as u32);
        }

        VertexBuffer::new(display, &path_vertices)
    }
}
