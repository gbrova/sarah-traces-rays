mod camera;
mod hit;
mod material;
mod ray;
mod rectanglexy;
mod sphere;
mod vec;

use camera::CameraSettings;
use rand::Rng;
use rayon::prelude::*;
use rectanglexy::RectangleXY;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::io::Write;
use std::{env, fs::File};
use vec::Vec3;

use material::{Dielectric, Lambertian, Metal};
use ray::Ray;
use rayon::iter::IntoParallelIterator;
use std::sync::Arc;
use vec::{Color, Point3};

use camera::Camera;
use hit::{Hit, World};
use sphere::Sphere;

#[derive(Serialize, Deserialize, Debug)]
struct Preset {
    // aspect_ratio: f64,
    image_width: u64,
    samples_per_pixel: u64,
    max_depth: u64,
    camera: CameraSettings,
}

fn ray_color(r: &Ray, world: &World, depth: u64) -> Color {
    if depth <= 0 {
        // If we've exceeded the ray bounce limit, no more light is gathered
        return Color::new(0.0, 0.0, 0.0);
    }
    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
            attenuation * ray_color(&scattered, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}
fn random_scene() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();

    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat);

    world.push(Box::new(ground_sphere));

    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat: f64 = rng.gen();
            let center = Point3::new(
                (a as f64) + rng.gen_range(0.0..0.9),
                0.2,
                (b as f64) + rng.gen_range(0.0..0.9),
            );

            if choose_mat < 0.8 {
                // Diffuse
                let albedo = Color::random(0.0..1.0) * Color::random(0.0..1.0);
                let sphere_mat = Arc::new(Lambertian::new(albedo));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else if choose_mat < 0.95 {
                // Metal
                let albedo = Color::random(0.4..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Arc::new(Metal::new(albedo, fuzz));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else {
                // Glass
                let sphere_mat = Arc::new(Dielectric::new(1.5));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            }
        }
    }

    let diffuse_mat = Arc::new(Dielectric::new(1.5));
    let rect = RectangleXY::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 5.0, 5.0),
        diffuse_mat,
    );
    world.push(Box::new(rect));

    let mat1 = Arc::new(Dielectric::new(1.5));
    let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    let mat3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1);
    let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2);
    let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3);

    world.push(Box::new(sphere1));
    world.push(Box::new(sphere2));
    world.push(Box::new(sphere3));

    world
}

fn load_preset_from_file(path_to_file: &str) -> Preset {
    let file = File::open(path_to_file).unwrap();
    let reader = BufReader::new(file);

    let u: Preset = serde_json::from_reader(reader).unwrap();
    u
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let preset: Preset = load_preset_from_file(&args[1]);

    // image
    let image_height: u64 = ((preset.image_width as f64) / preset.camera.aspect_ratio) as u64;

    // World
    let world = random_scene();

    let cam = Camera::new(preset.camera);

    let mut output = File::create(&args[2]).unwrap();
    writeln!(output, "P3").unwrap();
    writeln!(output, "{} {}", preset.image_width, image_height).unwrap();
    writeln!(output, "255").unwrap();

    for j in (0..image_height).rev() {
        eprintln!("Scanlines remaining: {}", j + 1);

        let scanline: Vec<Color> = (0..preset.image_width)
            .into_par_iter()
            .map(|i| {
                let mut rng = rand::thread_rng();

                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..preset.samples_per_pixel {
                    let random_u: f64 = rng.gen();
                    let random_v: f64 = rng.gen();

                    let u = ((i as f64) + random_u) / ((preset.image_width - 1) as f64);
                    let v = ((j as f64) + random_v) / ((image_height - 1) as f64);

                    let r = cam.get_ray(u, v);
                    pixel_color += ray_color(&r, &world, preset.max_depth);
                }

                pixel_color
            })
            .collect();

        for pixel_color in scanline {
            writeln!(
                output,
                "{}",
                pixel_color.format_color(preset.samples_per_pixel)
            )
            .unwrap();
        }
    }
    eprintln!("Done.");
}
