use std::marker::PhantomData;

use fxhash::FxBuildHasher;
use num::cast::AsPrimitive;
use ordered_float::{NotNan, OrderedFloat};

use crate::{MapTileVoxelizer, SimpleVoxelizer, Voxelizer};
use crate::collection::{HMap3DVoxelCollection, Vec2VoxelCollection, Vec3VoxelCollection, VoxelCollection};
use crate::element::{Int, Number, Resolution, UInt};

pub trait BuildVoxelizer<V: Voxelizer<Option>, Option: VoxelizerOption>
where
    Option::Color: AsPrimitive<Option::ColorPool>,
    Option::ColorPool: AsPrimitive<Option::Weight> + AsPrimitive<Option::Color>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
{
    fn build_voxelizer(resolution: Resolution) -> V {
        V::new(resolution)
    }
    fn voxelize_one<T>(pc: T, resolution: Resolution) -> Option::OutVC
    where
        T: VoxelCollection<Option::InPoint, Option::Weight, Option::Color>,
    {
        let mut voxelizer = Self::build_voxelizer(resolution);
        voxelizer.add(pc);
        voxelizer.finish()
    }
}

pub struct BuildVoxelizerDefault<V: Voxelizer<Option>, Option: VoxelizerOption>
where
    Option::ColorPool: AsPrimitive<Option::Weight>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
{
    _phantom: PhantomData<V>,
    _phantom2: PhantomData<Option>,
}

impl<V: Voxelizer<Option>, Option: VoxelizerOption> BuildVoxelizer<V, Option> for BuildVoxelizerDefault<V, Option>
where
    Option::ColorPool: AsPrimitive<Option::Weight>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
    Option::OutPoint: AsPrimitive<f64>,
    f64: AsPrimitive<Option::OutPoint>,
{}


pub struct SimpleVoxelizerDefaultOptions {}

impl VoxelizerOption for SimpleVoxelizerDefaultOptions
{
    type InPoint = OrderedFloat<f32>;
    type OutPoint = i32;
    type Color = u8;
    type Weight = u8;
    type ColorPool = u16;
    type CalcVC = HMap3DVoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool, FxBuildHasher>;
    type OutVC = HMap3DVoxelCollection<Self::OutPoint, Self::Weight, Self::Color, FxBuildHasher>;
}

pub type BuildSimpleVoxelizerDefault = BuildVoxelizerDefault<SimpleVoxelizer<SimpleVoxelizerDefaultOptions>, SimpleVoxelizerDefaultOptions>;


pub struct MapTileVoxelizerDefaultOptions {}

impl VoxelizerOption for MapTileVoxelizerDefaultOptions
where
    NotNan<f32>: AsPrimitive<f64>,
{
    type InPoint = OrderedFloat<f32>;
    type OutPoint = i32;
    type Color = u8;
    type Weight = u8;
    type ColorPool = u16;
    type CalcVC = Vec3VoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool>;
    type OutVC = Vec3VoxelCollection<Self::OutPoint, Self::Weight, Self::Color>;
}

pub type BuildMapTileVoxelizerDefault = BuildVoxelizerDefault<MapTileVoxelizer<MapTileVoxelizerDefaultOptions>, MapTileVoxelizerDefaultOptions>;

pub struct TerrainTileVoxelizerDefaultOptions {}

impl VoxelizerOption for TerrainTileVoxelizerDefaultOptions
{
    type InPoint = OrderedFloat<f64>;
    type OutPoint = i32;
    type Color = u8;
    type Weight = u8;
    type ColorPool = u16;
    type CalcVC = Vec2VoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool>;
    type OutVC = Vec2VoxelCollection<Self::OutPoint, Self::Weight, Self::Color>;
}

pub type BuildTerrainTileVoxelizerDefault = BuildVoxelizerDefault<MapTileVoxelizer<TerrainTileVoxelizerDefaultOptions>, TerrainTileVoxelizerDefaultOptions>;

pub trait VoxelizerOption
where
    Self::Color: AsPrimitive<Self::Weight>,
    Self::Weight: AsPrimitive<Self::Color>,
    Self::Color: AsPrimitive<Self::ColorPool>,
    Self::ColorPool: AsPrimitive<Self::Color>,
{
    type InPoint: Number;
    type OutPoint: Int;
    type Color: UInt;
    type Weight: UInt;
    type ColorPool: UInt;
    type CalcVC: VoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool>;
    type OutVC: VoxelCollection<Self::OutPoint, Self::Weight, Self::Color>;
}