use nalgebra::Vector3;

pub struct Sphere {
    pub position: Vector3<f64>,
    pub radius: f64,
    pub albedo: Vector3<f64>,
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
}

impl Sphere {
    pub fn new(position: Vector3<f64>, radius: f64, albedo: Vector3<f64>) -> Sphere {
        Sphere {
            position,
            radius,
            albedo,
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            position: Default::default(),
            radius: 0.5,
            albedo: nalgebra_glm::vec3(1.0, 1.0, 1.0),
        }
    }
}
