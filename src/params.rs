/// ボクセルデータを3D空間に配置する際の原点を設定するためのルール
///
/// Rules for setting the origin when placing voxel data in 3D space
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Offset {
    /// 各タイル原点を3D空間の原点と一致させる。
    /// `VoxelizerParams::TILING`が`false`の場合の挙動は`MinTile`と同じ。
    ///
    /// The origin of each tile is aligned with the origin of the 3D space.
    /// If `VoxelizerParams::TILING` is `false`, the behavior is the same as `MinTile`.
    Tile,

    /// ボクセルデータを含む最小タイルの原点を3D空間の原点と一致させる。
    /// `VoxelizerParams::TILING`が`false`の場合の挙動は`Tile`と同じ。
    ///
    /// The origin of the smallest tile containing voxel data is matched to the origin in 3D space.
    /// If `VoxelizerParams::TILING` is `false`, the behavior is the same as `Tile`.
    MinTile,

    /// XY平面上のAABBの最小座標を3D空間のXY原点と一致させる。
    /// Zは保持される。
    ///
    /// Align the minimum coordinates of AABB on the XY plane with the XY origin in 3D space.
    /// Z is preserved.
    Pixel,

    /// ボクセルデータのAABBの最小座標を3D空間の原点と一致させる。
    ///
    /// Align the minimum coordinates of AABB in the voxel data with the origin in 3D space.
    Voxel,

    /// 3D空間の原点をそのまま使用する。
    /// 単精度浮動小数点数で表現した際に十分な精度が得られることを確認して使用してください。
    ///
    /// Use the origin of 3D space as it is.
    /// Make sure that sufficient precision is available when expressed as a single-precision floating-point number before use.
    None,
}

/// `Voxelizer`のパラメータを定義するトレイト。
/// このトレイトを実装した構造体を使用することで、`Voxelizer`の挙動をカスタマイズできます。
///
/// A trace that defines the parameters of the `Voxelizer`.
/// Structures implementing this trace can be used to customize the behavior of the `Voxelizer`.
///
/// # Examples
///
/// ```
/// use voxel_tiler::{VoxelizerParams, Offset};
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
    /// ボクセルデータをタイルごとに分割するかどうか。
    ///
    /// Whether voxel data should be split into tiles or not.
    const TILING: bool;

    /// ボクセルを生成するために必要な点の数。
    /// 1ボクセル空間に含まれる点の数がこの値を超えるとボクセルが生成される。
    ///
    /// The number of points required to generate a voxel.
    /// A voxel is generated when the number of points in one voxel space exceeds this value.
    const THRESHOLD: usize;

    /// ボクセルデータを3D空間に配置する際の基準。
    ///
    /// Criteria for placing voxel data in 3D space.
    const OFFSET: Offset;
}

/// 汎用的なパラメータをデフォルトで提供しています。
/// このモジュール内の構造体を参考に、独自のパラメータを定義することもできます。
///
/// Generic parameters are provided by default.
/// You can also define your own parameters by referring to the structures in this module.
pub mod default_params {
    use super::*;

    /// タイルごとに分割したボクセルデータを生成するためのパラメータ。
    /// 3Dデータをタイルデータとして配信したい場合は、このパラメータを使用してデータを生成します。
    /// このパラメータの使用例は`examples/write_voxel_tiles`にあります。
    ///
    /// Parameter for generating voxel data divided into tiles.
    /// If you want to deliver 3D data as tile data, use this parameter to generate the data.
    /// Examples of using this parameter can be found in `examples/write_voxel_tiles`.
    pub struct Tile;

    impl VoxelizerParams for Tile {
        const TILING: bool = true;
        const THRESHOLD: usize = 1;
        const OFFSET: Offset = Offset::Tile;
    }

    /// ボクセルデータを分割せずに生成するためのパラメータ。
    /// 点群をそのまま1つのボクセルデータに変換でき、かつAABBの最小を原点に合わせるため、単純なボクセルデータの可視化に便利です。
    /// このパラメータの使用例は`examples/write_voxel`にあります。
    ///
    /// Parameters for generating voxel data without splitting.
    /// This parameter is useful for visualization of simple voxel data because it can convert a point cloud directly into a single voxel data and also aligns the minimum of AABB to the origin.
    /// An example of using this parameter can be found in `examples/write_voxel`.
    pub struct Fit;

    impl VoxelizerParams for Fit {
        const TILING: bool = false;
        const THRESHOLD: usize = 1;
        const OFFSET: Offset = Offset::Voxel;
    }
}
