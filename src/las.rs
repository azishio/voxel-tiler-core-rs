use las::{Point, Read};
use num::cast::AsPrimitive;
use ordered_float::OrderedFloat;

use crate::collection::{PointCloud, VoxelCollection};
use crate::element::{Color, Point3D, UInt};

impl<W> PointCloud<OrderedFloat<f64>, W, u16>
where
    W: UInt + AsPrimitive<u16>,
    u16: AsPrimitive<W>,
{
    /// lasファイルから点群を読み込みます。
    /// 使用するには`las`featureを有効にしてください。
    pub fn from_las(mut reader: las::Reader) -> Self <> {
        let points = reader.points().flatten().map(|p| {
            let Point { x, y, z, color, .. } = p;
            let color = {
                if let Some(color) = color {
                    let las::Color { red, green, blue } = color;
                    Color::new([red, green, blue])
                } else {
                    Color::new([0, 0, 0])
                }
            };

            let point = Point3D::new([
                OrderedFloat::from(x),
                OrderedFloat::from(y),
                OrderedFloat::from(z)
            ]);

            (point, color)
        }).collect();

        PointCloud::<OrderedFloat<f64>, W, u16>::builder().points(points).build()
    }
}
