use crate::conversion::CartesianCoords;

#[derive(Copy, Clone, Debug, Default)]
pub struct ObjVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture: [f32; 2],
}

impl From<CartesianCoords> for ObjVertex {
    fn from(c: CartesianCoords) -> Self {
        Self {
            position: c.into(),
            normal: [0.0; 3],
            texture: [0.0; 2],
        }
    }
}

/// Return the indices from the `obj` file as a 1-directional vector where, in the case of
/// triangular meshes, the chunks of `3` from the output `Vec` correspond to the faces.
///
/// This is used to create a `glium::IndexBuffer`
pub fn get_indices(data: &obj::ObjData) -> Vec<u16> {
    data.objects
        .iter()
        .flat_map(move |object| object.groups.iter().flat_map(|g| g.polys.iter()))
        .flat_map(|obj::SimplePolygon(indices)| indices.iter().map(|x| u16::try_from(x.0).unwrap()))
        .skip(6) // investigate this skipping (mattered for teapot)
        .collect()
}

/// Parse the byte stream from the obj file to an `ObjData` result
pub fn get_objdata(data: &[u8]) -> Result<obj::ObjData, obj::ObjError> {
    let mut data = ::std::io::BufReader::new(data);
    obj::ObjData::load_buf(&mut data)
}
