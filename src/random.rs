use nalgebra::{Vector3, Vector4};

/// Wang hash
pub fn random_u32(seed: &mut u32) -> u32 {
    *seed = (*seed ^ 61) ^ (*seed >> 16);
    *seed *= 9;
    *seed = *seed ^ (*seed >> 4);
    *seed *= 0x27d4eb2d;
    *seed = *seed ^ (*seed >> 15);
    *seed
}

const INVERSE_U32MAX: f64 = 1.0 / u32::MAX as f64;

pub fn random_f64(seed: &mut u32) -> f64 {
    *seed = random_u32(seed);
    *seed as f64 * INVERSE_U32MAX
}

pub fn random_vec3f64(seed: &mut u32) -> Vector3<f64> {
    nalgebra_glm::vec3(
        random_f64(seed) * 2.0 - 1.0,
        random_f64(seed) * 2.0 - 1.0,
        random_f64(seed) * 2.0 - 1.0,
    )
}

pub fn random_vec4f64(seed: &mut u32) -> Vector4<f64> {
    nalgebra_glm::vec4(
        random_f64(seed) * 2.0 - 1.0,
        random_f64(seed) * 2.0 - 1.0,
        random_f64(seed) * 2.0 - 1.0,
        random_f64(seed) * 2.0 - 1.0,
    )
}
