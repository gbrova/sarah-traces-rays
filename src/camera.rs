use serde::{Deserialize, Serialize};

use super::ray::Ray;
use super::vec::{Point3, Vec3};

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    cu: Vec3,
    cv: Vec3,
    lens_radius: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CameraSettings {
    lookfrom: Point3,
    lookat: Point3,
    vup: Vec3,
    vfov: f64,
    pub aspect_ratio: f64,
    aperture: f64,
    focus_dist: f64,
}

impl Camera {
    pub fn new(settings: CameraSettings) -> Camera {
        // Vertical field-of-view in degrees
        let theta = std::f64::consts::PI / 180.0 * settings.vfov;
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = settings.aspect_ratio * viewport_height;

        let cw = (settings.lookfrom - settings.lookat).normalized();
        let cu = settings.vup.cross(cw).normalized();
        let cv = cw.cross(cu);

        let h = settings.focus_dist * viewport_width * cu;
        let v = settings.focus_dist * viewport_height * cv;
        let llc = settings.lookfrom - h / 2.0 - v / 2.0 - settings.focus_dist * cw;

        Camera {
            origin: settings.lookfrom,
            horizontal: h,
            vertical: v,
            lower_left_corner: llc,
            cu,
            cv,
            lens_radius: settings.aperture / 2.0,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.cu * rd.x() + self.cv * rd.y();

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}
