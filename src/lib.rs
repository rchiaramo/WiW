mod app;
mod scene;
mod sphere;
mod camera;
mod util_funcs;
mod raytracer;

pub use app::App;
pub use sphere::Sphere;
pub use camera::Camera;
pub use scene::Scene;
pub use raytracer::RayTracer;

const SPHERE_COUNT:usize = 64;