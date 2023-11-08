#[derive(Default, Clone, Debug)]
pub struct PolarCoords {
    r: f32,
    long: f32,
    lat: f32,
}

impl PolarCoords {
    pub fn sum_with(&mut self, other: &PolarCoords) {
        self.r += other.r;
        self.long += other.long;
        self.lat += other.lat;
    }
}

impl From<&CartesianCoords> for PolarCoords {
    fn from(cartesian_coords: &CartesianCoords) -> Self {
        let calc_r = f32::sqrt(
            cartesian_coords.x.powi(2) + cartesian_coords.y.powi(2) + cartesian_coords.z.powi(2),
        );

        let mut calc_long = f32::atan2(cartesian_coords.y, cartesian_coords.x);
        if calc_long.is_nan() {
            calc_long = 0.0;
        }

        let mut calc_lat = cartesian_coords.z / calc_r;
        if calc_lat.is_nan() {
            calc_lat = 0.0;
        }

        Self {
            r: calc_r,
            long: calc_long,
            lat: calc_lat,
        }
    }
}

impl From<&String> for PolarCoords {
    fn from(str: &String) -> Self {
        let mut split_string = str.split(' ');
        Self {
            r: split_string.next().unwrap().parse::<f32>().unwrap(),
            long: split_string.next().unwrap().parse::<f32>().unwrap(),
            lat: split_string.next().unwrap().parse::<f32>().unwrap(),
        }
    }
}

impl ToString for PolarCoords {
    fn to_string(&self) -> String {
        [
            self.r.to_string(),
            self.long.to_string(),
            self.lat.to_string(),
        ]
        .join(" ")
    }
}

#[derive(Default, Clone, Debug)]
pub struct CartesianCoords {
    x: f32,
    y: f32,
    z: f32,
}

impl CartesianCoords {
    pub fn sum_with(&mut self, other: &CartesianCoords) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }

    pub fn subtract_with(&mut self, other: &CartesianCoords) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl From<&PolarCoords> for CartesianCoords {
    fn from(polar_coords: &PolarCoords) -> Self {
        Self {
            x: polar_coords.r * polar_coords.long.sin() * polar_coords.lat.cos(),
            y: polar_coords.r * polar_coords.long.sin() * polar_coords.lat.sin(),
            z: polar_coords.r * polar_coords.long.sin(),
        }
    }
}

impl From<[f32; 3]> for CartesianCoords {
    fn from(s: [f32; 3]) -> Self {
        Self {
            x: s[0],
            y: s[1],
            z: s[2],
        }
    }
}

impl From<CartesianCoords> for [f32; 3] {
    fn from(c: CartesianCoords) -> Self {
        [c.x, c.y, c.z]
    }
}