use glium::{vertex::BufferCreationError, Display, VertexBuffer};
use glutin::surface::WindowSurface;
use petgraph::{
    algo::{self, has_path_connecting},
    graph,
    visit::Dfs,
    Graph, Undirected,
};
use rustc_hash::FxHashSet;

use crate::{
    conversion::{CartesianCoords, PolarCoords},
    load::ObjVertex,
};

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

    /// get the 3D position of the `ObjVertex` at `idx` in this graph
    fn position_at(&self, idx: graph::NodeIndex) -> [f32; 3] {
        self.graph.node_weight(idx).unwrap().position
    }

    /// Convert the given `path` to a Vec of polar offsets from each `ObjVertex` to the next.
    pub fn path_to_polar_offs(&self, path: &[graph::NodeIndex]) -> Vec<String> {
        let mut normalized = Vec::new();

        let mut prv = path
            .iter()
            .next()
            .map(|idx| CartesianCoords::from(self.position_at(*idx)))
            .unwrap();
        for next in path.iter().map(|idx| self.position_at(*idx)) {
            let mut coords = CartesianCoords::from(next);
            coords.subtract_with(&prv);

            let polar_coords = PolarCoords::from(&coords);

            normalized.push(polar_coords.to_string());

            prv = CartesianCoords::from(next);
        }
        normalized
    }

    pub fn path_to_buffer(
        &self,
        path: &[graph::NodeIndex],
        display: &Display<WindowSurface>,
    ) -> Result<VertexBuffer<ObjVertex>, BufferCreationError> {
        let mut path_vertices = vec![
            ObjVertex {
                position: [0.0; 3],
                normal: [0.0; 3],
                texture: [-1.0; 2],
            };
            self.graph.node_count()
        ];

        for idx in path.iter() {
            *path_vertices.get_mut(idx.index()).unwrap() = *self.graph.node_weight(*idx).unwrap();
        }

        VertexBuffer::new(display, &path_vertices)
    }

    /// Return the longest continuous path from `start_idx` in this graph
    pub fn continuous_path_from(&self, start_idx: u32) -> Vec<graph::NodeIndex> {
        let start_node = graph::NodeIndex::from(start_idx);

        // bellman ford is able to return the longest continuous paths given negative edge
        // weights and a bit of annoying parsing
        let bellman_ford = algo::bellman_ford(&self.graph, start_node).unwrap();
        let mut prev = graph::NodeIndex::from(start_idx);

        let mut path_vertices = Vec::new();
        // parse the predecessors field to step along the path from `start_idx`
        // TODO: explain `idx` and `predecessor` relationship
        while let Some((idx, _)) = bellman_ford
            .predecessors
            .iter()
            .enumerate()
            .find(|(_, &predecessor)| predecessor == Some(prev))
        {
            path_vertices.push(prev);
            prev = graph::NodeIndex::from(idx as u32);
        }

        path_vertices
    }

    /// Return the polar offsets along the path of each connected subgraph
    pub fn connected_subgraph_polar_offs(&self) -> Vec<Vec<String>> {
        self.components
            .iter()
            .map(|idx| self.path_to_polar_offs(&self.continuous_path_from(idx.index() as u32)))
            .collect()
    }

    /// Return a `VertexBuffer` for each connected subgraph
    pub fn connected_subgraph_buffers(
        &self,
        display: &Display<WindowSurface>,
    ) -> Vec<VertexBuffer<ObjVertex>> {
        self.components
            .iter()
            .map(|idx| {
                self.path_to_buffer(&self.continuous_path_from(idx.index() as u32), display)
                    .unwrap()
            })
            .collect()
    }
}
