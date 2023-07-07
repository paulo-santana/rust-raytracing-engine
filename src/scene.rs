use nalgebra::{Vector3, Vector4};

pub struct Material {
    pub albedo: Vector4<f64>,
    pub roughness: f64,
    pub metallic: f64,
    pub emission_color: Vector4<f64>,
    pub emission_power: f64,
}

pub struct Sphere {
    pub position: Vector3<f64>,
    pub radius: f64,
    pub material_index: usize,
}

pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
}

impl Sphere {
    pub fn new(position: Vector3<f64>, radius: f64) -> Sphere {
        Sphere {
            position,
            radius,
            material_index: 0,
        }
    }
}

impl Material {
    pub fn get_emission(&self) -> Vector4<f64> {
        self.emission_color * self.emission_power
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
            roughness: 1.0,
            metallic: 0.0,
            emission_color: Vector4::new(0.0, 0.0, 0.0, 1.0),
            emission_power: 0.0,
        }
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            position: Default::default(),
            radius: 0.5,
            material_index: 0,
        }
    }
}
