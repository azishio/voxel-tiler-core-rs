use coordinate_transformer::{pixel_resolution, ZoomLv};
use fxhash::FxBuildHasher;
use image::{DynamicImage, Rgb};

use crate::collection::{HMap2DVoxelCollection, VoxelCollection};
use crate::element::{Color, Point3D};

struct DefaultColorIter;

impl Iterator for DefaultColorIter {
    type Item = Rgb<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Rgb::from([0, 0, 0]))
    }
}

pub enum AltitudeResolutionCriteria {
    ZoomLv(ZoomLv),
    Lat(f64, ZoomLv),
}

pub struct JTerrainImageSampler;

impl JTerrainImageSampler {
    pub fn sampling(
        resolution: AltitudeResolutionCriteria,
        altitude_image: DynamicImage,
        color_image: Option<DynamicImage>,
    ) -> Result<HMap2DVoxelCollection<u32, u8, u8, FxBuildHasher>, anyhow::Error> {
        let resolution = match resolution {
            AltitudeResolutionCriteria::ZoomLv(zoom_lv) => {
                // 日本経緯度原点の緯度
                let japan_origin_lat = (35_f64 + (39. / 64.) + (29.1572 / 3600.)).to_radians();

                pixel_resolution(japan_origin_lat, zoom_lv)
            }
            AltitudeResolutionCriteria::Lat(lat, zoom_lv) => pixel_resolution(lat, zoom_lv),
        };

        const TILE_SIZE: u32 = 256;

        let color_image = color_image
            .unwrap_or_else(|| {
                DynamicImage::ImageRgb8(image::ImageBuffer::from_fn(
                    TILE_SIZE,
                    TILE_SIZE,
                    |_, _| Rgb::from([0, 0, 0]),
                ))
            })
            .into_rgb8();

        let points = altitude_image
            .into_rgb8()
            .pixels()
            .zip(color_image.pixels())
            .collect::<Vec<_>>()
            .chunks(TILE_SIZE as usize)
            .enumerate()
            .flat_map(|(y, line)| {
                line.iter()
                    .enumerate()
                    .filter_map(move |(x, (&height, &color))| {
                        let z = {
                            let r = height[0] as f64;
                            let g = height[1] as f64;
                            let b = height[2] as f64;

                            let x = 2_f64.powi(16) * r + 2_f64.powi(8) * g + b;
                            let u = 0.01;

                            if x < 2_f64.powi(23) {
                                Some(x * u)
                            } else if x > 2_f64.powi(23) {
                                Some((x - 2_f64.powi(24)) * u)
                            } else {
                                None
                            }
                        };

                        if let Some(z) = z {
                            let z = (z / resolution) as u32;
                            let point = Point3D::new([x as u32, y as u32, z]);
                            let color = Color::new(color.0);

                            Some((point, color))
                        } else {
                            None
                        }
                    })
            })
            .collect();

        Ok(HMap2DVoxelCollection::builder()
            .points(points)
            .resolution(resolution)
            .build())
    }
}
