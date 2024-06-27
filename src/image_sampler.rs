use std::io::Read;

use anyhow::anyhow;
use ordered_float::NotNan;
use png::OutputInfo;

use crate::element::{Color, Point3D};

pub struct GIAJTerrainImageSet<T:Read>{
    height:T,
    color:T,
}

pub struct JTerrainImageSampler{}

impl JTerrainImageSampler {
    fn check_image_info(info:OutputInfo)->Result<(),anyhow::Error>{
        let OutputInfo { width, height, color_type, bit_depth, .. } = info;

        if width != 256 || height != 256 {
            return Err(anyhow!("width and height must be 256"));
        }

        if color_type != png::ColorType::Rgb {
            return Err(anyhow!("color_type must be RGB"));
        }

        if bit_depth != png::BitDepth::Eight {
            return Err(anyhow!("bit_depth must be 8"));
        }

        Ok(())
    }
    pub fn sampling<T:Read>(image_set:GIAJTerrainImageSet<T>) -> Result<Vec<(Point3D<NotNan<f64>>,Color<u8>)>, anyhow::Error> {
        let height = png::Decoder::new(image_set.height);
        let mut height_reader = height.read_info()?;
        let mut height_buf = vec![0;height_reader.output_buffer_size()];
        Self::check_image_info(height_reader.next_frame(&mut height_buf)?)?;

        let color = png::Decoder::new(image_set.color);
        let mut color_reader = color.read_info()?;
        let mut color_buf = vec![0;color_reader.output_buffer_size()];
        Self::check_image_info(color_reader.next_frame(&mut color_buf)?)?;

        let points = height_buf.chunks(3).zip(color_buf.chunks(3)).enumerate().filter_map(|(i,(height, color))| {

            let z = {
                let r = height[0] as f64;
                let g = height[1] as f64;
                let b = height[2] as f64;

                let x = 2_f64.powi(16) * r + 2_f64.powi(8) * g + b;
                let u = 0.01;

                if x < 2_f64.powi(23){
                    Some(x * u)
                }else if x > 2_f64.powi(23){
                    Some((x - 2_f64.powi(24)) * u)
                } else{
                    None
                }
            };

            let point =                 if let Some(z) = z {
                let x = NotNan::new((i % 256) as f64).unwrap();
                let y = NotNan::new((i / 256) as f64).unwrap();
                let z = NotNan::new(z).unwrap();
                Some(Point3D::new([x, y, z]))
                }else{
                    None
                };

            let color = if let &[r,g,b] = color {
                Some(Color::new([r,g,b]))
            } else {
                None
            };

            if let (Some(point), Some(color)) = (point, color) {
                Some((point, color))
            } else {
                None
            }
        }).collect::<Vec<_>>();

        Ok(points)
    }
}
