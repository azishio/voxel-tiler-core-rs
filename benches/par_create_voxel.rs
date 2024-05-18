use std::fs::File;
use std::io::BufReader;

use coordinate_transformer::{JprOrigin, ZoomLv};
use criterion::{Criterion, criterion_group, criterion_main};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

use voxel_tiler_core::{PlyStructs, Voxelizer};
use voxel_tiler_core::default_params::{Fit, Tile};

fn create_voxel() -> Vec<Vec<u8>> {
    let las = BufReader::new(File::open("./examples/data-source/01JE2423.las").unwrap());

    let v = Voxelizer::<Fit>::voxelize_from_jpr_las(las, JprOrigin::One, ZoomLv::Lv17, false);

    v.into_par_iter().map(|(_, voxel_mesh)| PlyStructs::from_voxel_mesh(voxel_mesh).to_ascii_ply_buf()).collect::<Vec<_>>()
}

fn create_voxel_tile() -> Vec<Vec<u8>> {
    let las = BufReader::new(File::open("./examples/data-source/01JE2423.las").unwrap());

    let v = Voxelizer::<Tile>::voxelize_from_jpr_las(las, JprOrigin::One, ZoomLv::Lv17, false);

    v.into_par_iter().map(|(_, voxel_mesh)| PlyStructs::from_voxel_mesh(voxel_mesh).to_ascii_ply_buf()).collect::<Vec<_>>()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    if cfg!(feature = "rayon") {
        println!("rayon enabled");
    } else {
        println!("rayon disabled");
    }

    let mut group = c.benchmark_group("create_voxel");
    group.sample_size(10);

    if cfg!(feature = "rayon") {
        group.bench_function("fit", |b| b.iter(create_voxel));
        group.bench_function("tile", |b| b.iter(create_voxel_tile));
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
