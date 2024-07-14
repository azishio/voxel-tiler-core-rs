use coordinate_transformer::{jpr2ll, JprOrigin, ll2pixel, pixel_resolution, ZoomLv};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use num::traits::AsPrimitive;

use crate::build_voxelizer::VoxelizerOption;
use crate::collection::{PointCloud, VoxelCollection};
use crate::element::{Point2D, Point3D, Resolution, Voxel};
use crate::voxelizer::private::PrivateVoxelizerMethod;

mod private {
    use num::cast::AsPrimitive;

    use crate::build_voxelizer::VoxelizerOption;
    use crate::element::{Color, Point3D, Voxel};

    pub trait PrivateVoxelizerMethod<Option: VoxelizerOption>
    {
        fn average_color(arg: (Point3D<Option::OutPoint>, Voxel<Option::ColorPool, Option::Weight>)) -> (Point3D<Option::OutPoint>, Voxel<Option::Color, Option::Weight>)
        where
            Option::Color: AsPrimitive<Option::ColorPool>,
            Option::ColorPool: AsPrimitive<Option::Weight> + AsPrimitive<Option::Color>,
            Option::Weight: AsPrimitive<Option::ColorPool>,
        {
            let (point, voxel) = arg;

            let color: Color<Option::Color> = (voxel.color / Color::from(voxel.weight).as_::<Option::ColorPool>()).as_();

            let voxel = Voxel::new(color);

            (point, voxel)
        }
    }
}

pub trait Voxelizer<Option: VoxelizerOption>: PrivateVoxelizerMethod<Option>
{
    fn new(resolution: Resolution) -> Self;
    fn add<T: VoxelCollection<Option::InPoint, Option::Weight, Option::Color>>(&mut self, pc: T);
    fn finish(self) -> Option::OutVC;
}

pub struct SimpleVoxelizer<Option: VoxelizerOption>
{
    field: Option::CalcVC,
    resolution: f64,
}

impl<Option: VoxelizerOption> PrivateVoxelizerMethod<Option> for SimpleVoxelizer<Option>
where
    Option::Color: AsPrimitive<Option::ColorPool>,
    Option::ColorPool: AsPrimitive<Option::Weight> + AsPrimitive<Option::Color>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
{}


impl<Option: VoxelizerOption> Default for SimpleVoxelizer<Option>
where
    Option::ColorPool: AsPrimitive<Option::Weight>,
    Option::InPoint: AsPrimitive<f64>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
{
    fn default() -> Self {
        Self {
            field: Option::CalcVC::default(),
            resolution: 1.,
        }
    }
}

impl<Option: VoxelizerOption> Voxelizer<Option> for SimpleVoxelizer<Option>
where
    Option::InPoint: AsPrimitive<f64>,
    Option::ColorPool: AsPrimitive<Option::Weight>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
    Option::OutPoint: AsPrimitive<f64>,
    f64: AsPrimitive<Option::InPoint>,
    f64: AsPrimitive<Option::OutPoint>,
{
    fn new(resolution: Resolution) -> Self {
        match resolution {
            Resolution::Mater(resolution) =>
                SimpleVoxelizer {
                    field: Option::CalcVC::default(),
                    resolution,
                },
            _ => panic!("Resolution is not mater"),
        }
    }

    fn add<T: VoxelCollection<Option::InPoint, Option::Weight, Option::Color>>(&mut self, pc: T) {
        let voxels = pc.into_vec_with_offset().into_iter().map(|(point, voxel)| {
            let point = (point.as_::<f64>() / self.resolution).batch(|a| a.floor()).as_::<Option::OutPoint>();

            let color = voxel.color.as_::<Option::ColorPool>();
            let voxel = Voxel::new(color);
            (point, voxel)
        }).collect::<Vec<_>>();

        let pc = PointCloud::<Option::OutPoint, Option::Weight, Option::ColorPool>::builder().voxels(voxels).build();

        self.field = self.field.clone().merge(pc).unwrap();
    }
    fn finish(mut self) -> Option::OutVC
    {
        let current_bounds = if self.field.has_bounds() {
            Some(self.field.get_bounds())
        } else { None };

        let offset = self.field.get_offset();

        let points = self.field.into_vec().into_iter().map(Self::average_color).collect();


        Option::OutVC::new(points, current_bounds, offset, self.resolution)
    }
}

pub struct MapTileVoxelizer<Option: VoxelizerOption>
{
    // value: (Resolution, VoxelsCollection)
    field: DashMap<Point2D<u32>, Option::CalcVC, FxBuildHasher>,
    zoom_lv: ZoomLv,
    jpr_origin: JprOrigin,
}

impl<Option: VoxelizerOption> MapTileVoxelizer<Option> {
    fn finish_tiles(self) -> Vec<(Point2D<u32>, Option::OutVC)>
    where
        Option::Weight: AsPrimitive<Option::ColorPool>,
        Option::ColorPool: AsPrimitive<Option::Weight>,
    {
        self.field.into_iter().map(|(tile, mut pc)| {
            let bounds = pc.get_bounds();
            let offset = pc.get_offset();
            let resolution = pc.get_resolution();

            let voxels = pc.into_vec().into_iter().map(Self::average_color).collect();

            (tile, Option::OutVC::new(voxels, Some(bounds), offset, resolution))
        }).collect::<Vec<_>>()
    }
}

impl<Option: VoxelizerOption> PrivateVoxelizerMethod<Option> for MapTileVoxelizer<Option>
where
    Option::Color: AsPrimitive<Option::ColorPool>,
    Option::ColorPool: AsPrimitive<Option::Weight> + AsPrimitive<Option::Color>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
{}


impl<Option: VoxelizerOption> Voxelizer<Option> for MapTileVoxelizer<Option>
where
    Option::Color: AsPrimitive<Option::ColorPool>,
    Option::ColorPool: AsPrimitive<Option::Weight> + AsPrimitive<Option::Color>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
    Option::InPoint: AsPrimitive<f64>,
    Option::OutPoint: AsPrimitive<u32>,
    u32: AsPrimitive<Option::OutPoint>,
{
    fn new(resolution: Resolution) -> Self {
        match resolution {
            Resolution::Tile { zoom_lv, jpr_origin } =>
                MapTileVoxelizer {
                    field: DashMap::with_hasher(FxBuildHasher::default()),
                    zoom_lv,
                    jpr_origin,
                },
            _ => panic!("Resolution is not tile"),
        }
    }

    fn add<T: VoxelCollection<Option::InPoint, Option::Weight, Option::Color>>(&mut self, pc: T)
    {
        pc.into_vec().into_iter().for_each(|(point, voxel)| {
            let x = point[0].as_();
            let y = point[1].as_();

            let (long, lat) = jpr2ll((y, x), self.jpr_origin);
            let (pixel_x, pixel_y) = ll2pixel((long, lat), self.zoom_lv);
            let tile = Point2D::new([pixel_x / 256, pixel_y / 256]);

            let resolution = pixel_resolution(lat, self.zoom_lv);

            let pixel_z = (point[2].as_() / resolution).floor() as u32;

            let point = Point3D::new([pixel_x, pixel_y, pixel_z]).as_();
            let voxel = Voxel::new(voxel.color.as_::<Option::ColorPool>());

            self.field.entry(tile).and_modify(|field| {
                field.insert_one(point, voxel);
            }).or_insert(
                Option::CalcVC::builder()
                    .voxels(vec![(point, voxel)])
                    .resolution(resolution)
                    .build()
            );
        });
    }


    fn finish(self) -> Option::OutVC
    {
        let (_tile, vcf_list): (Vec<_>, Vec<_>) = self.field.into_iter().unzip();

        let min_resolution = vcf_list.iter().map(|vcf| vcf.get_resolution()).reduce(|a, b| a.min(b)).unwrap();
        let max_resolution = vcf_list.iter().map(|vcf| vcf.get_resolution()).reduce(|a, b| a.max(b)).unwrap();
        let average_resolution = (min_resolution + max_resolution) / 2.;

        let voxels = vcf_list.into_iter().flat_map(|v| { v.into_vec_with_offset() }).map(Self::average_color).collect();

        Option::OutVC::builder()
            .voxels(voxels)
            .resolution(average_resolution)
            .build()
    }
}


