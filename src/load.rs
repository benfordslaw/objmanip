#![allow(dead_code)]

use petgraph::{
    algo::has_path_connecting,
    graph::{self, NodeIndex},
    visit::{Dfs, IntoEdges},
    Directed,
    Direction::Incoming,
    Graph, Undirected,
};
use rustc_hash::FxHashSet;

#[derive(Copy, Clone, Debug, Default)]
pub struct ObjVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture: [f32; 2],
}

pub fn get_indices(data: &obj::ObjData) -> Vec<u16> {
    data.objects
        .iter()
        .flat_map(move |object| object.groups.iter().flat_map(|g| g.polys.iter()))
        .flat_map(|obj::SimplePolygon(indices)| indices.iter().map(|x| x.0 as u16))
        .skip(6)
        .collect()
}

pub fn get_objdata(data: &[u8]) -> Result<obj::ObjData, obj::ObjError> {
    let mut data = ::std::io::BufReader::new(data);
    obj::ObjData::load_buf(&mut data)
}

/// Returns an undirected graph where the nodes are `ObjVertex` and connected if they are
/// connected in the obj file.
pub fn load_wavefront(data: &obj::ObjData) -> petgraph::Graph<ObjVertex, f32, Directed> {
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
            let normal = v.2.map(|index| data.normal[index]);

            let texture = texture.unwrap_or([0.0; 2]);
            let normal = normal.unwrap_or([0.0; 3]);
            let normal = data.normal[v.0];

            *vertex_graph
                .node_weight_mut(NodeIndex::from(v.0 as u32))
                .unwrap() = ObjVertex {
                position,
                normal,
                texture,
            };
        }

        // add all edges, avoiding duplicates
        // (1, 2, 3) + (2, 3, 1) -> (1, 2), (2, 3), (3, 1)
        for (v1, v2) in indices.iter().zip(indices.iter().cycle().skip(1)) {
            // parent node should have more incoming edges than child
            let parent_node = NodeIndex::from(v1.0 as u32);
            let child_node = NodeIndex::from(v2.0 as u32);
            un_vertex_graph.update_edge(parent_node, child_node, 0.0);
        }
    }
    let mut seen_vertices = FxHashSet::<NodeIndex>::default();
    let mut start_vertices = FxHashSet::<NodeIndex>::default();
    for start in un_vertex_graph.node_indices() {
        if seen_vertices.contains(&start) {
            continue;
        }
        start_vertices.insert(start);
        let mut dfs = Dfs::new(&vertex_graph, start);
        let mut prv = start;
        while let Some(visited) = dfs.next(&un_vertex_graph) {
            if !has_path_connecting(&vertex_graph, visited, prv, None)
                && un_vertex_graph.contains_edge(prv, visited)
            {
                seen_vertices.insert(visited);
                vertex_graph.update_edge(prv, visited, -1.0);
            }
            prv = visited;
        }
    }
    println!("{:?}", start_vertices);

    vertex_graph
}