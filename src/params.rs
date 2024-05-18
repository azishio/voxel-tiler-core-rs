/// Rules for setting the origin when placing voxel data in 3D space
///
/// ボクセルデータを3D空間に配置する際の原点を設定するためのルール
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Offset {
    /// If `VoxelizerParams::TILING` is `false`, the behavior is the same as `MinTile`.
    /// The origin of each tile is aligned with the origin of the 3D space.
    ///
    /// `VoxelizerParams::TILING`が`false`の場合の挙動は`MinTile`と同じ。
    /// 各タイル原点を3D空間の原点と一致させる。
    Tile,

    /// The origin of the smallest tile containing voxel data is matched to the origin in 3D space.
    /// If `VoxelizerParams::TILING` is `false`, the behavior is the same as `Tile`.
    ///
    /// ボクセルデータを含む最小タイルの原点を3D空間の原点と一致させる。
    /// `VoxelizerParams::TILING`が`false`の場合の挙動は`Tile`と同じ。
    MinTile,

    /// Align the minimum coordinates of AABB on the XY plane with the XY origin in 3D space.
    /// Z is preserved.
    ///
    /// XY平面上のAABBの最小座標を3D空間のXY原点と一致させる。
    /// Zは保持される。
    Pixel,

    /// Align the minimum coordinates of AABB in the voxel data with the origin in 3D space.
    ///
    /// ボクセルデータのAABBの最小座標を3D空間の原点と一致させる。
    Voxel,

    /// Use the origin of 3D space as it is.
    /// Make sure that sufficient precision is available when expressed as a single-precision floating-point number before use.
    ///
    /// 3D空間の原点をそのまま使用する。
    /// 単精度浮動小数点数で表現した際に十分な精度が得られることを確認して使用してください。
    None,
}

/// A trace that defines the parameters of the `Voxelizer`.
/// Structures implementing this trace can be used to customize the behavior of the `Voxelizer`.
///
/// `Voxelizer`のパラメータを定義するトレイト。
/// このトレイトを実装した構造体を使用することで、`Voxelizer`の挙動をカスタマイズできます。
///
/// # Examples
///
/// ```
/// use voxel_tiler_core::{VoxelizerParams, Offset};
///
/// pub struct MyParams;
///
/// impl VoxelizerParams for MyParams {
///    const TILING: bool = true;
///    const THRESHOLD: usize = 1;
///    const OFFSET: Offset = Offset::Tile;
/// }
/// ```
///
pub trait VoxelizerParams {
    /// Whether voxel data should be split into tiles or not.
    ///
    /// ボクセルデータをタイルごとに分割するかどうか。
    const TILING: bool;

    /// The number of points required to generate a voxel.
    /// A voxel is generated when the number of points in one voxel space exceeds this value.
    ///
    /// ボクセルを生成するために必要な点の数。
    /// 1ボクセル空間に含まれる点の数がこの値を超えるとボクセルが生成される。
    const THRESHOLD: usize;

    /// Criteria for placing voxel data in 3D space.
    ///
    /// ボクセルデータを3D空間に配置する際の基準。
    const OFFSET: Offset;
}

/// Generic parameters are provided by default.
/// You can also define your own parameters by referring to the structures in this module.
///
/// 汎用的なパラメータをデフォルトで提供しています。
/// このモジュール内の構造体を参考に、独自のパラメータを定義することもできます。
pub mod default_params {
    use super::*;

    /// Parameter for generating voxel data divided into tiles.
    /// If you want to deliver 3D data as tile data, use this parameter to generate the data.
    /// Examples of using this parameter can be found in `examples/write_voxel_tiles`.
    ///
    /// タイルごとに分割したボクセルデータを生成するためのパラメータ。
    /// 3Dデータをタイルデータとして配信したい場合は、このパラメータを使用してデータを生成します。
    /// このパラメータの使用例は`examples/write_voxel_tiles`にあります。
    pub struct Tile;

    impl VoxelizerParams for Tile {
        const TILING: bool = true;
        const THRESHOLD: usize = 1;
        const OFFSET: Offset = Offset::Tile;
    }

    /// Parameters for generating voxel data without splitting.
    /// This parameter is useful for visualization of simple voxel data because it can convert a point cloud directly into a single voxel data and also aligns the minimum of AABB to the origin.
    /// An example of using this parameter can be found in `examples/write_voxel`.
    ///
    /// ボクセルデータを分割せずに生成するためのパラメータ。
    /// 点群をそのまま1つのボクセルデータに変換でき、かつAABBの最小を原点に合わせるため、単純なボクセルデータの可視化に便利です。
    /// このパラメータの使用例は`examples/write_voxel`にあります。
    pub struct Fit;

    impl VoxelizerParams for Fit {
        const TILING: bool = false;
        const THRESHOLD: usize = 1;
        const OFFSET: Offset = Offset::Voxel;
    }
}
