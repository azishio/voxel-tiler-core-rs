use num::cast::AsPrimitive;

use crate::collection::VoxelCollection;
use crate::element::{Int, Number, UInt};

pub trait VoxelizerParams
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
    type Field: VoxelCollection<Self::OutPoint, Self::Weight, Self::ColorPool>;
    type OutVC: VoxelCollection<Self::OutPoint, Self::Weight, Self::Color>;
}

