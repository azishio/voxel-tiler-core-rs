use std::fs::File;
use std::io::BufReader;

use coordinate_transformer::{JprOrigin, ZoomLv};
use criterion::{Criterion, criterion_group, criterion_main};

use voxel_tiler_core::{PlyStructs, Voxelizer};
use voxel_tiler_core::default_params::Tile;

fn create_voxel() {
    let require_zoom_lv = vec![ZoomLv::Lv17];

    require_zoom_lv.into_iter().for_each(|zoom_lv| {
        let las = BufReader::new(File::open("./examples/data-source/01JE2421.las").unwrap());

        let v = Voxelizer::<Tile>::voxelize_from_jpr_las(las, JprOrigin::One, zoom_lv, false);

        let (_tile_idx, voxel_mesh) = v.into_iter().next().unwrap();

        let ply = PlyStructs::from_voxel_mesh(voxel_mesh).to_ascii_ply_buf();

        println!("{}", ply.len());
    });
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
        group.bench_function("parallel", |b| b.iter(create_voxel));
    } else {
        group.bench_function("sequential", |b| b.iter(create_voxel));
    }

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
