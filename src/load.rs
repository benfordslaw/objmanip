glium::implement_vertex!(ObjVertex, position, normal, texture);

#[derive(Copy, Clone, Debug, Default)]
pub struct ObjVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture: [f32; 2],
}

pub fn obj_triangles(data: &obj::ObjData) -> Vec<Vec<usize>> {
    data.objects
        .iter()
        .flat_map(move |object| object.groups.iter().flat_map(|g| g.polys.iter()))
        .map(|obj::SimplePolygon(indices)| {
            indices.iter().map(|index| index.0).collect::<Vec<usize>>()
        })
        .collect::<Vec<Vec<usize>>>()
}

pub fn get_objdata(data: &[u8]) -> Result<obj::ObjData, obj::ObjError> {
    let mut data = ::std::io::BufReader::new(data);
    obj::ObjData::load_buf(&mut data)
}
