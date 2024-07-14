use std::marker::PhantomData;

use fxhash::FxBuildHasher;
use num::cast::AsPrimitive;
use ordered_float::{NotNan, OrderedFloat};

use crate::collection::{HMap3DVoxelCollection, Vec2VoxelCollection, Vec3VoxelCollection, VoxelCollection};
use crate::element::{Int, Number, UInt};
use crate::voxelizer::{MapTileVoxelizer, Resolution, SimpleVoxelizer, Voxelizer};

/// ボクセライザーを共通のインターフェースで構築するためのトレイトです。
pub trait BuildVoxelizer<V: Voxelizer<Option>, Option: VoxelizerOption>
where
    Option::Color: AsPrimitive<Option::ColorPool>,
    Option::ColorPool: AsPrimitive<Option::Weight> + AsPrimitive<Option::Color>,
    Option::Weight: AsPrimitive<Option::ColorPool>,
{
    /// 指定した分解能でボクセライザーを構築します。
    fn build_voxelizer(resolution: Resolution) -> V {
        V::new(resolution)
    }
    ///　点群をボクセライザーに追加し、ボクセル化を行います。
    fn voxelize_one<T>(pc: T, resolution: Resolution) -> Option::OutVC
    where
        T: VoxelCollection<Option::InPoint, Option::Weight, Option::Color>,
    {
        let mut voxelizer = Self::build_voxelizer(resolution);
        voxelizer.add(pc);
        voxelizer.finish()
    }
}

///　標準で用意されたオプションでボクセライザーを構築するための構造体です。
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


/// [`SimpleVoxelizer`]の標準オプションです。
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

/// [`SimpleVoxelizer`]のインスタンスを標準オプションで生成する構造体です。
pub type BuildSimpleVoxelizerDefault = BuildVoxelizerDefault<SimpleVoxelizer<SimpleVoxelizerDefaultOptions>, SimpleVoxelizerDefaultOptions>;


/// [`MapTileVoxelizer`]の標準オプションです。
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

/// [`MapTileVoxelizer`]のインスタンスを標準オプションで生成する構造体です。
pub type BuildMapTileVoxelizerDefault = BuildVoxelizerDefault<MapTileVoxelizer<MapTileVoxelizerDefaultOptions>, MapTileVoxelizerDefaultOptions>;

/// 地形データなど、同一の平面座標において複数の高さを持たない点群をボクセル化するための標準オプションです。
/// 高低差が激しい地形などはボクセルが不連続になるため、このオプションを使用することは適していません。
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

/// [`MapTileVoxelizer`]のインスタンスを[`TerrainTileVoxelizerDefaultOptions`]で生成する構造体です。
pub type BuildTerrainTileVoxelizerDefault = BuildVoxelizerDefault<MapTileVoxelizer<TerrainTileVoxelizerDefaultOptions>, TerrainTileVoxelizerDefaultOptions>;

/// ボクセライザーのオプションを表すトレイトです。
pub trait VoxelizerOption
where
    Self::Color: AsPrimitive<Self::Weight>,
    Self::Weight: AsPrimitive<Self::Color>,
    Self::Color: AsPrimitive<Self::ColorPool>,
    Self::ColorPool: AsPrimitive<Self::Color>,
{
    /// 入力点群の座標値に用いる型です。
    type InPoint: Number;

    /// 出力ボクセルの座標値に用いる型です。
    type OutPoint: Int;

    /// ボクセルの色を表す型です。
    type Color: UInt;

    /// ボクセルの重みを表す型です。
    /// ボクセルが専有する空間において、存在する超点数を表します。
    type Weight: UInt;

    ///　計算時のボクセルの色を表す型です。
    /// 頂点色の平均値を計算する際に用いられます。
    /// この型は、`Color`,`Weight`よりも大きな型を取る必要があります。
    type ColorPool: UInt;

    /// 計算時に用いるボクセルコレクションの型です。
    type CalcVC: VoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool>;

    /// 出力時に用いるボクセルコレクションの型です。
    type OutVC: VoxelCollection<Self::OutPoint, Self::Weight, Self::Color>;
}
