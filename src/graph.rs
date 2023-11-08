use glium::{vertex::BufferCreationError, Display, VertexBuffer};
use glutin::surface::WindowSurface;
use petgraph::{
    algo::{self, has_path_connecting},
    graph,
    visit::Dfs,
    Graph, Undirected,
};
use rustc_hash::FxHashSet;

use crate::load::ObjVertex;

/// Directed graph
pub struct VertexGraph {
    graph: Graph<ObjVertex, f32>,
    components: FxHashSet<graph::NodeIndex>,
}

impl VertexGraph {
    pub fn new(g: Graph<ObjVertex, f32>) -> Self {
        Self {
            graph: g,
            components: FxHashSet::default(),
        }
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

    /// Use the undirected edges from `undirected` to add directed edges in the order determined
    /// by depth first search.
    ///
    /// Assigns the start indices of each disconnected component to `VertexGraph::components`.
    pub fn add_dag_edges(&mut self, undirected: &Graph<ObjVertex, f32, Undirected>) {
        let mut seen_vertices = FxHashSet::<graph::NodeIndex>::default();
        let mut start_vertices = FxHashSet::<graph::NodeIndex>::default();
        for start in undirected.node_indices() {
            if seen_vertices.contains(&start) {
                continue;
            }
            start_vertices.insert(start);
            let mut dfs = Dfs::new(&self.graph, start);
            let mut prv = start;
            while let Some(visited) = dfs.next(undirected) {
                if !has_path_connecting(&self.graph, visited, prv, None)
                    && undirected.contains_edge(prv, visited)
                {
                    seen_vertices.insert(visited);
                    self.graph.update_edge(prv, visited, -1.0);
                }
                prv = visited;
            }
        }

        self.components = start_vertices;
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

    pub fn connected_subgraphs(
        &self,
        display: &Display<WindowSurface>,
    ) -> Vec<VertexBuffer<ObjVertex>> {
        self.components
            .iter()
            .map(|x| {
                self.continuous_path_from(x.index() as u32, display)
                    .unwrap()
            })
            .collect()
    }
}
