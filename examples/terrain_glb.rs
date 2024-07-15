use std::fs::File;
use std::io::{Read, Write};

use coordinate_transformer::ZoomLv;
use gltf::Glb;
use image::io::Reader;

use voxel_tiler_core::giaj_terrain::{AltitudeResolutionCriteria, GIAJTerrainImageSampler};
use voxel_tiler_core::glb::{GlbGen, Mime, TextureInfo};
use voxel_tiler_core::mesh::{Mesher, ValidSide};

fn main() -> Result<(), anyhow::Error> {
    let altitude = Reader::open("examples/data-source/altitude.png")?.decode()?;
    let mut color_file = File::open("examples/data-source/color.jpg")?;
    let mut color_buf = Vec::<u8>::new();
    color_file.read_to_end(&mut color_buf)?;

    let resolution = AltitudeResolutionCriteria::ZoomLv(ZoomLv::Lv15);
    let sampler = GIAJTerrainImageSampler::sampling(resolution, altitude, None)?;

    let mesh = Mesher::meshing(sampler, ValidSide::all() - ValidSide::BOTTOM - ValidSide::BORDER).simplify();

    let texture = TextureInfo {
        buf: Some(color_buf),
        uri: None,
        mime_type: Mime::ImageJpeg,
    };

    let glb = Glb::from_voxel_mesh_with_texture_projected_z(mesh, texture)?;

    let mut writer = File::create("examples/exports/terrain.glb")?;
    glb.to_writer(&mut writer)?;
    writer.flush()?;

    println!("Generated terrain.glb");
    Ok(())
}
