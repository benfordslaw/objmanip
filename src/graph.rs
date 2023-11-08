use glium::{vertex::BufferCreationError, Display, VertexBuffer};
use glutin::surface::WindowSurface;
use obj::ObjData;
use petgraph::{
    algo::{self, has_path_connecting},
    graph::{self, NodeIndex},
    visit::Dfs,
    Directed, Graph, Undirected,
};
use rustc_hash::FxHashSet;

use crate::{
    conversion::{CartesianCoords, PolarCoords},
    load::ObjVertex,
};

/// Directed graph
pub struct VertexDag {
    graph: Graph<ObjVertex, f32>,
    components: FxHashSet<graph::NodeIndex>,
}

impl VertexDag {
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
                .copied()
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
            prev = graph::NodeIndex::from(u32::try_from(idx).unwrap());
        }

        path_vertices
    }

    /// Return the polar offsets along the path of each connected subgraph
    pub fn connected_subgraph_polar_offs(&self) -> Vec<Vec<String>> {
        self.components
            .iter()
            .map(|idx| {
                self.path_to_polar_offs(
                    &self.continuous_path_from(u32::try_from(idx.index()).unwrap()),
                )
            })
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
                self.path_to_buffer(
                    &self.continuous_path_from(u32::try_from(idx.index()).unwrap()),
                    display,
                )
                .unwrap()
            })
            .collect()
    }
}

/// Returns an undirected graph where the nodes are `ObjVertex` and connected if they are
/// connected in the obj file.
impl From<&ObjData> for VertexDag {
    fn from(data: &ObjData) -> Self {
        glium::implement_vertex!(ObjVertex, position, normal, texture);

        let mut vertex_graph: Graph<ObjVertex, f32, Directed> = graph::Graph::new();
        let mut un_vertex_graph: Graph<ObjVertex, f32, Undirected> = graph::Graph::new_undirected();
        let mut seen_vertices = FxHashSet::<usize>::default();

        // initialize empty nodes
        for _ in 0..data.position.len() {
            vertex_graph.add_node(ObjVertex::default());
            un_vertex_graph.add_node(ObjVertex::default());
        }

        for obj::SimplePolygon(indices) in data
            .objects
            .iter()
            .flat_map(|object| object.groups.iter().flat_map(|g| g.polys.iter()))
        {
            // add unseen positions as new nodes in vertex_graph
            for v in indices.iter().filter(|v| seen_vertices.insert(v.0)) {
                let position = data.position[v.0];

                let texture = v.1.map(|index| data.texture[index]);
                let texture = texture.unwrap_or([0.0; 2]);

                // TODO: need some way of determining which format the `obj` is using to specify
                // normals
                //
                // this is used when the normals are specified by the faces in the obj
                let normal = v.2.map(|index| data.normal[index]);
                let _normal = normal.unwrap_or([0.0; 3]);

                // this is used when the normals are specified by `vn` in the obj
                let normal = data.normal[v.0];

                *vertex_graph
                    .node_weight_mut(NodeIndex::from(u32::try_from(v.0).unwrap()))
                    .unwrap() = ObjVertex {
                    position,
                    normal,
                    texture,
                };
            }

            // add all edges between vertices in the triangle
            // (1, 2, 3) + (2, 3, 1) -> (1, 2), (2, 3), (3, 1)
            for (v1, v2) in indices.iter().zip(indices.iter().cycle().skip(1)) {
                // parent node should have more incoming edges than child
                let parent_node = NodeIndex::from(u32::try_from(v1.0).unwrap());
                let child_node = NodeIndex::from(u32::try_from(v2.0).unwrap());
                un_vertex_graph.update_edge(parent_node, child_node, 0.0);
            }
        }

        let mut vertex_graph = VertexDag::new(vertex_graph);
        vertex_graph.add_dag_edges(&un_vertex_graph);
        vertex_graph
    }
}
