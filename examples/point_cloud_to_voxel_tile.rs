use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};

use coordinate_transformer::{jpr2ll, JprOrigin, ZoomLv};
use las::Reader;
use ordered_float::OrderedFloat;

use voxel_tiler_core::build_voxelizer::{BuildMapTileVoxelizerDefault, BuildVoxelizer};
use voxel_tiler_core::collection::{PointCloud, VoxelCollection};
use voxel_tiler_core::element::Point3D;
use voxel_tiler_core::mesh::{Mesher, ValidSide};
use voxel_tiler_core::ply::PlyStructs;
use voxel_tiler_core::voxelizer::{Resolution, Voxelizer};

fn main() {
    let file = File::open("examples/data-source/point_cloud.laz").unwrap();
    let reader = Reader::new(BufReader::new(file)).unwrap();
    let point_cloud = PointCloud::<OrderedFloat<f64>, u8, u16>::from_las(reader);

    let ll_point_cloud = {
        let transformed = point_cloud.into_points().into_iter().map(|(point, color)| {
            let x = point[0].into_inner();
            let y = point[1].into_inner();
            let (long, lat) = jpr2ll((y, x), JprOrigin::One);

            let long = OrderedFloat::from(long);
            let lat = OrderedFloat::from(lat);

            let ll_point = Point3D::new([long, lat, point[2]]);

            (ll_point, color)
        }).collect::<Vec<_>>();

        PointCloud::builder().points(transformed).build()
    };

    let resolution = Resolution::Tile {
        zoom_lv: ZoomLv::Lv17,
    };

    let mut voxelizer = BuildMapTileVoxelizerDefault::build_voxelizer(resolution);

    voxelizer.add(ll_point_cloud);

    let tiles = voxelizer.finish_tiles();

    create_dir_all("examples/exports").expect("I/O error");
    tiles.into_iter().for_each(|(tile, vc)| {
        let [tile_x, tile_y] = tile.data;

        let mesh = Mesher::meshing(vc, ValidSide::all());

        let ply = PlyStructs::from_voxel_mesh(mesh.clone());

        let buf = ply.into_ascii_buf();

        let mut writer = File::create(format!("examples/exports/point_cloud_tile_{}-{}.ply", tile_x, tile_y)).expect("I/O error");
        writer.write_all(&buf).expect("I/O error");
        writer.flush().expect("I/O error");

        // 私が試した環境では、多数の頂点カラーを持つglbファイルを生成すると、レンダリングが非常に高コストになります。
        // 同等のplyは何故か軽量です
        // 現在調査中です

        //let glb = Glb::from_voxel_mesh(mesh).unwrap();
        //
        //let writer = File::create(format!("examples/exports/point_cloud_tile_{}-{}.glb", tile_x, tile_y)).expect("I/O error");
        //glb.to_writer(writer).expect("glTF binary output error");
        //println!("Generated examples/point_cloud_tile_{}-{}.glb", tile_x, tile_y);
    });
}
