use std::marker::PhantomData;

use fxhash::FxBuildHasher;
use num::cast::AsPrimitive;
use ordered_float::{NotNan, OrderedFloat};

use crate::{MapTileVoxelizer, SimpleVoxelizer, Voxelizer, VoxelizerParams};
use crate::collection::{HMap3DVoxelCollection, Vec2VoxelCollection, Vec3VoxelCollection, VoxelCollection};
use crate::element::Resolution;

pub trait BuildVoxelizer<V: Voxelizer<Param>, Param: VoxelizerParams>
where
    Param::Color: AsPrimitive<Param::ColorPool>,
    Param::ColorPool: AsPrimitive<Param::Weight> + AsPrimitive<Param::Color>,
    Param::Weight: AsPrimitive<Param::ColorPool>,
{
    fn build_voxelizer(resolution: Resolution) -> V {
        V::new(resolution)
    }
    fn voxelize_one<T>(pc: T, resolution: Resolution) -> Param::OutVC
    where
        T: VoxelCollection<Param::InPoint, Param::Weight, Param::Color>,
    {
        let mut voxelizer = Self::build_voxelizer(resolution);
        voxelizer.add(pc);
        voxelizer.finish()
    }
}

pub struct BuildVoxelizerDefault<V: Voxelizer<Param>, Param: VoxelizerParams>
where
    Param::ColorPool: AsPrimitive<Param::Weight>,
    Param::Weight: AsPrimitive<Param::ColorPool>,
{
    _phantom: PhantomData<V>,
    _phantom2: PhantomData<Param>,
}

impl<V: Voxelizer<Param>, Param: VoxelizerParams> BuildVoxelizer<V, Param> for BuildVoxelizerDefault<V, Param>
where
    Param::ColorPool: AsPrimitive<Param::Weight>,
    Param::Weight: AsPrimitive<Param::ColorPool>,
    Param::OutPoint: AsPrimitive<f64>,
    f64: AsPrimitive<Param::OutPoint>,
{}


pub struct SimpleVoxelizerDefaultParams {}

impl VoxelizerParams for SimpleVoxelizerDefaultParams
{
    type InPoint = OrderedFloat<f32>;
    type OutPoint = i32;
    type Color = u8;
    type Weight = u8;
    type ColorPool = u16;
    type Field = HMap3DVoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool,FxBuildHasher>;
    type OutVC = HMap3DVoxelCollection<Self::OutPoint, Self::Weight, Self::Color,FxBuildHasher>;
}

pub type BuildSimpleVoxelizerDefault = BuildVoxelizerDefault<SimpleVoxelizer<SimpleVoxelizerDefaultParams>, SimpleVoxelizerDefaultParams>;


pub struct MapTileVoxelizerDefaultParams {}

impl VoxelizerParams for MapTileVoxelizerDefaultParams
where
    NotNan<f32>: AsPrimitive<f64>,
{
    type InPoint = OrderedFloat<f32>;
    type OutPoint = i32;
    type Color = u8;
    type Weight = u8;
    type ColorPool = u16;
    type Field = Vec3VoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool>;
    type OutVC = Vec3VoxelCollection<Self::OutPoint, Self::Weight, Self::Color>;
}

pub type BuildMapTileVoxelizerDefault = BuildVoxelizerDefault<MapTileVoxelizer<MapTileVoxelizerDefaultParams>, MapTileVoxelizerDefaultParams>;

pub struct TerrainTileVoxelizerDefaultParams {}

impl VoxelizerParams for TerrainTileVoxelizerDefaultParams
{
    type InPoint = OrderedFloat<f32>;
    type OutPoint = i32;
    type Color = u8;
    type Weight = u8;
    type ColorPool = u16;
    type Field = Vec2VoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool>;
    type OutVC = Vec2VoxelCollection<Self::OutPoint, Self::Weight, Self::Color>;
}

pub type BuildTerrainTileVoxelizerDefault = BuildVoxelizerDefault<MapTileVoxelizer<TerrainTileVoxelizerDefaultParams>, TerrainTileVoxelizerDefaultParams>;
