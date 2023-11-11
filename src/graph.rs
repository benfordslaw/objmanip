use glium::{vertex::BufferCreationError, Display, VertexBuffer};
use glutin::surface::WindowSurface;
use itertools::Itertools;
use obj::ObjData;
use petgraph::{
    algo::has_path_connecting,
    graph::{self, NodeIndex},
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
    undirected: Graph<ObjVertex, usize, Undirected>,
    components: FxHashSet<graph::NodeIndex>,
}

impl VertexDag {
    pub fn new(g: Graph<ObjVertex, f32>) -> Self {
        Self {
            graph: g,
            undirected: graph::UnGraph::new_undirected(),
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

    pub fn get_paths(&self) -> Vec<Vec<NodeIndex>> {
        // sort undirected weights by y value, min by y

        let sorted_indices = self
            .graph
            .node_indices()
            .sorted_unstable_by(|&n1, &n2| {
                self.graph.node_weight(n1).unwrap().position[1]
                    .total_cmp(&self.graph.node_weight(n2).unwrap().position[1])
            })
            .collect::<Vec<NodeIndex>>();

        let mut all_out_paths = Vec::<Vec<NodeIndex>>::new();
        let mut seen_vertices = FxHashSet::<NodeIndex>::default();

        for start in &sorted_indices {
            if seen_vertices.contains(start) {
                continue;
            }
            let mut current_start_path = vec![*start];
            seen_vertices.insert(*start);

            let mut prv = *start;

            // get the earliest-occurring (i.e. lowest y-val) neighbor
            while let Some(next) = self
                .undirected
                .neighbors(prv)
                .filter(|node| !seen_vertices.contains(node))
                .min_by_key(|&node| sorted_indices.iter().position(|&n| n == node).unwrap())
            {
                seen_vertices.insert(next);
                current_start_path.push(next);
                prv = next;
            }

            if prv != *start {
                all_out_paths.push(current_start_path);
            }
        }
        all_out_paths
    }

    /// Use the undirected edges from `undirected` to add directed edges in the order determined
    /// by depth first search.
    ///
    /// Assigns the start indices of each disconnected component to `VertexGraph::components`.
    pub fn add_dag_edges(&mut self) {
        let mut start_vertices = FxHashSet::<NodeIndex>::default();

        let mut start = NodeIndex::from(2u32);
        let mut prv = NodeIndex::from(1u32);

        while prv != start {
            start = prv;
            // initialize to start, updated with previous
            let mut currently_seen_vertices = FxHashSet::<graph::NodeIndex>::default();

            // the next node is the neighbor of the previous node that
            // 1. was not previously connected by an edge
            // 2. does not have a path in this graph connecting to the previous node
            // 3. has the shortest path to the start node
            //
            // 1-2 ensure that this graph is not cyclic, 3 attempts to make this a localized
            // region in the graph
            while let Some(next) = self
                .undirected
                .neighbors(prv)
                .filter(|&node| {
                    !currently_seen_vertices.contains(&node)
                        && !has_path_connecting(&self.graph, node, prv, None)
                })
                .min_by_key(|&node| {
                    petgraph::algo::astar(
                        &self.undirected,
                        node,
                        |finish| finish == start,
                        |_| 1,
                        |_| 1,
                    )
                    .unwrap()
                    .0
                })
            {
                self.graph.add_edge(prv, next, -1.0);
                currently_seen_vertices.insert(next);
                prv = next;
            }

            start_vertices.insert(start);
        }
        self.components = start_vertices;
    }

    /// get the cartesian position of the `ObjVertex` at `idx` in this graph
    fn position_at(&self, idx: graph::NodeIndex) -> [f32; 3] {
        self.graph.node_weight(idx).unwrap().position
    }

    /// Convert the given `path` to a Vec of polar offsets from each `ObjVertex` to the next.
    pub fn path_to_polar_offs(&self, path: &[graph::NodeIndex]) -> Vec<String> {
        let mut offsets = Vec::new();

        let mut path_iter = path.iter();
        let mut prv = path_iter
            .next()
            .map(|idx| PolarCoords::from(&CartesianCoords::from(self.position_at(*idx))))
            .unwrap_or_default();
        for next in
            path_iter.map(|idx| PolarCoords::from(&CartesianCoords::from(self.position_at(*idx))))
        {
            prv.subtract_with(&next);
            offsets.push(prv.to_string());

            prv = next;
        }
        offsets
    }
}

/// Returns an undirected graph where the nodes are `ObjVertex` and connected if they are
/// connected in the obj file.
impl From<&ObjData> for VertexDag {
    fn from(data: &ObjData) -> Self {
        glium::implement_vertex!(ObjVertex, position, normal, texture);

        let mut vertex_graph: Graph<ObjVertex, f32, Directed> = graph::Graph::new();
        let mut un_vertex_graph: Graph<ObjVertex, usize, Undirected> =
            graph::Graph::new_undirected();
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

                // TODO: need some way of determining which format the `obj` is using to specify
                // normals
                //
                // this is used when the normals are specified by the faces in the obj
                let normal = v.2.map(|index| data.normal[index]);
                let normal = normal.unwrap_or([0.0; 3]);

                // this is used when the normals are specified by `vn` in the obj
                // let normal = data.normal[v.0];

                *vertex_graph
                    .node_weight_mut(NodeIndex::from(u32::try_from(v.0).unwrap()))
                    .unwrap() = ObjVertex {
                    position,
                    normal,
                    texture: [0.0; 2], // because we don't do anything meaningful yet here
                };
            }

            // add all edges between vertices in the triangle
            // (1, 2, 3) + (2, 3, 1) -> (1, 2), (2, 3), (3, 1)
            for (v1, v2) in indices.iter().zip(indices.iter().cycle().skip(1)) {
                // parent node should have more incoming edges than child
                let parent_node = NodeIndex::from(u32::try_from(v1.0).unwrap());
                let child_node = NodeIndex::from(u32::try_from(v2.0).unwrap());
                un_vertex_graph.update_edge(parent_node, child_node, 1);
            }
        }

        let mut vertex_graph = VertexDag::new(vertex_graph);
        vertex_graph.undirected = un_vertex_graph;
        vertex_graph.add_dag_edges();
        vertex_graph
    }
}
