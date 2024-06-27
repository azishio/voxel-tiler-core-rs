use coordinate_transformer::{jpr2ll, JprOrigin, ll2pixel, pixel_resolution, ZoomLv};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use num::traits::AsPrimitive;

use crate::collection::{PointCloud, VoxelCollection};
use crate::element::{Point2D, Point3D, Resolution, Voxel};
use crate::voxelizer::private::PrivateVoxelizerMethod;
use crate::VoxelizerParams;

mod private {
    use num::cast::AsPrimitive;

    use crate::element::{Color, Point3D, Voxel};
    use crate::VoxelizerParams;

    pub trait PrivateVoxelizerMethod<Param: VoxelizerParams>
    {
        fn average_color(vec: Vec<(Point3D<Param::OutPoint>, Voxel<Param::ColorPool, Param::Weight>)>) -> Vec<(Point3D<Param::OutPoint>, Voxel<Param::Color, Param::Weight>)>
        where
            Param::Color: AsPrimitive<Param::ColorPool>,
            Param::ColorPool: AsPrimitive<Param::Weight> + AsPrimitive<Param::Color>,
            Param::Weight: AsPrimitive<Param::ColorPool>,
        {
            vec.into_iter().map(|(point, voxel)| {
                let color: Color<Param::Color> = (voxel.color / Color::from(voxel.weight).as_::<Param::ColorPool>()).as_();
                let voxel = Voxel::new(color);

                (point, voxel)
            }).collect::<Vec<_>>()
        }
    }
}

pub trait Voxelizer<Param: VoxelizerParams>: PrivateVoxelizerMethod<Param>
{
    fn new(resolution: Resolution) -> Self;
    fn add<T: VoxelCollection<Param::InPoint, Param::Weight, Param::Color>>(&mut self, pc: T);
    fn finish(self) -> Param::OutVC;
}

pub struct SimpleVoxelizer<Param: VoxelizerParams>
{
    field: Param::Field,
    resolution: f64,
}

impl<Param: VoxelizerParams> PrivateVoxelizerMethod<Param> for SimpleVoxelizer<Param>
where
    Param::Color: AsPrimitive<Param::ColorPool>,
    Param::ColorPool: AsPrimitive<Param::Weight> + AsPrimitive<Param::Color>,
    Param::Weight: AsPrimitive<Param::ColorPool>,
{}


impl<Param: VoxelizerParams> Default for SimpleVoxelizer<Param>
where
    Param::ColorPool: AsPrimitive<Param::Weight>,
    Param::InPoint: AsPrimitive<f64>,
    Param::Weight: AsPrimitive<Param::ColorPool>,
{
    fn default() -> Self {
        Self {
            field: Param::Field::default(),
            resolution: 1.,
        }
    }
}

impl<Param: VoxelizerParams> Voxelizer<Param> for SimpleVoxelizer<Param>
where
    Param::InPoint: AsPrimitive<f64>,
    Param::ColorPool: AsPrimitive<Param::Weight>,
    Param::Weight: AsPrimitive<Param::ColorPool>,
    Param::OutPoint: AsPrimitive<f64>,
    f64: AsPrimitive<Param::InPoint>,
    f64: AsPrimitive<Param::OutPoint>,
{
    fn new(resolution: Resolution) -> Self {
        match resolution {
            Resolution::Mater(resolution) =>
                SimpleVoxelizer {
                    field: Param::Field::default(),
                    resolution,
                },
            _ => panic!("Resolution is not mater"),
        }
    }

    fn add<T: VoxelCollection<Param::InPoint, Param::Weight, Param::Color>>(&mut self, pc: T) {
        let voxels = pc.into_vec().into_iter().map(|(point, voxel)| {
            let point = (point.as_::<f64>() / self.resolution).batch(|a| a.floor()).as_::<Param::OutPoint>();

            let color = voxel.color.as_::<Param::ColorPool>();
            let voxel = Voxel::new(color);
            (point, voxel)
        }).collect::<Vec<_>>();

        let pc = PointCloud::<Param::OutPoint, Param::Weight, Param::ColorPool>::from_voxels(voxels);

        self.field.merge(pc)
    }
    fn finish(mut self) -> Param::OutVC
    {
        let current_bounds = if self.field.has_bounds() {
            Some(self.field.get_bounds())
        } else { None };

        let points = Self::average_color(self.field.into_vec());

        match current_bounds {
            Some(bounds) => {
                Param::OutVC::with_bounds(points, bounds)
            }
            None => {
                Param::OutVC::from_voxels(points)
            }
        }
    }
}

pub struct MapTileVoxelizer<Param: VoxelizerParams>
{
    field: DashMap<Point2D<u32>, Param::Field, FxBuildHasher>,
    zoom_lv: ZoomLv,
    jpr_origin: JprOrigin,
}

impl<Param: VoxelizerParams> MapTileVoxelizer<Param> {
    fn finish_tiles(self) -> Vec<(Point2D<u32>, Param::OutVC)>
    where
        Param::Weight: AsPrimitive<Param::ColorPool>,
        Param::ColorPool: AsPrimitive<Param::Weight>,
    {
        self.field.into_iter().map(|(tile, mut pc)| {
            let bounds = pc.get_bounds();
            let points = Self::average_color(pc.into_vec());

            (tile, Param::OutVC::with_bounds(points, bounds))
        }).collect::<Vec<_>>()
    }
}

impl<Param: VoxelizerParams> PrivateVoxelizerMethod<Param> for MapTileVoxelizer<Param>
where
    Param::Color: AsPrimitive<Param::ColorPool>,
    Param::ColorPool: AsPrimitive<Param::Weight> + AsPrimitive<Param::Color>,
    Param::Weight: AsPrimitive<Param::ColorPool>,
{}


impl<Param: VoxelizerParams> Voxelizer<Param> for MapTileVoxelizer<Param>
where
    Param::Color: AsPrimitive<Param::ColorPool>,
    Param::ColorPool: AsPrimitive<Param::Weight> + AsPrimitive<Param::Color>,
    Param::Weight: AsPrimitive<Param::ColorPool>,
    Param::InPoint: AsPrimitive<f64>,
    Param::OutPoint: AsPrimitive<u32>,
    u32: AsPrimitive<Param::OutPoint>,
{
    fn new(resolution: Resolution) -> Self {
        match resolution {
            Resolution::Tile {zoom_lv,jpr_origin} =>
                MapTileVoxelizer {
                    field: DashMap::with_hasher(FxBuildHasher::default()),
                    zoom_lv,
                    jpr_origin,
                },
            _ => panic!("Resolution is not tile"),
        }
    }

    fn add<T: VoxelCollection<Param::InPoint, Param::Weight, Param::Color>>(&mut self, pc: T)
    {
        pc.into_vec().into_iter().for_each(|(point, voxel)| {
            let x = point[0].as_();
            let y = point[1].as_();

            let (long, lat) = jpr2ll((y, x), self.jpr_origin);
            let (pixel_x, pixel_y) = ll2pixel((long, lat), self.zoom_lv);
            let tile = Point2D::new([pixel_x / 256, pixel_y / 256]);

            let pixel_z = {
                let resolution = pixel_resolution(lat, self.zoom_lv);

                (point[2].as_() / resolution).floor() as u32
            };

            let point = Point3D::new([pixel_x, pixel_y, pixel_z]).as_();
            let voxel = Voxel::new(voxel.color.as_::<Param::ColorPool>());

            self.field.entry(tile).and_modify(|field| {
                field.insert_one(point, voxel);
            }).or_insert(
                Param::Field::from_voxels(vec![(point, voxel)])
            );
        });
    }


    fn finish(self) -> Param::OutVC
    {
        let (_tile, vcf_list): (Vec<_>, Vec<_>) = self.field.into_iter().unzip();

        let voxels = Self::average_color(vcf_list.into_iter().flat_map(|v| { v.into_vec() }).collect());

        Param::OutVC::from_voxels(voxels)
    }
}


