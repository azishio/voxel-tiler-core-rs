use std::io::{BufReader, Read};

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use num::cast::AsPrimitive;
use ordered_float::OrderedFloat;
use ply_rs::parser::Parser;
use ply_rs::ply::{Addable, DefaultElement, ElementDef, Encoding, Ply, Property, PropertyAccess, PropertyDef, PropertyType, ScalarType};
use ply_rs::ply::Property::{Float, ListUInt, UChar};
use ply_rs::writer::Writer;

use crate::collection::{PointCloud, VoxelCollection};
use crate::element::{Color, Int, Point3D, UInt};
use crate::voxel_mesh::VoxelMesh;

/// Plyファイルにおける1つの頂点を表す構造体
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Vertex {
    pub x: OrderedFloat<f32>,
    pub y: OrderedFloat<f32>,
    pub z: OrderedFloat<f32>,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PropertyAccess for Vertex {
    fn new() -> Self {
        Vertex::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("x", Float(v)) => self.x = OrderedFloat::from(v),
            ("y", Float(v)) => self.y = OrderedFloat::from(v),
            ("z", Float(v)) => self.z = OrderedFloat::from(v),
            ("red", UChar(v)) => self.r = v,
            ("green", UChar(v)) => self.g = v,
            ("blue", UChar(v)) => self.b = v,
            _ => {}
        }
    }
}


/// Plyファイルにおける1つの面を表す構造体
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Face {
    pub vertex_indices: Vec<u32>,
}

impl PropertyAccess for Face {
    fn new() -> Self {
        Face::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("vertex_indices", ListUInt(v)) => self.vertex_indices = v,
            _ => {}
        }
    }
}

impl<W> PointCloud<OrderedFloat<f32>, W, u8>
where
    W: UInt + AsPrimitive<u8>,
    u8: AsPrimitive<W>,
{
    /// plyファイルから点群を読み込みます。
    /// 使用するには`ply`featureを有効にしてください。
    pub fn from_ply<T: Read>(file: T) -> Self {
        let mut buf_reader = BufReader::new(file);

        let vertex_parser = Parser::<Vertex>::new();

        let header = vertex_parser.read_header(&mut buf_reader).unwrap();


        let points = header.elements.iter()
            .filter(|(_, element)| element.name == "vertex")
            .flat_map(|(_, element)| {
                vertex_parser.read_payload_for_element(&mut buf_reader, element, &header).unwrap()
            }).map(|Vertex { x, y, z, r, g, b }| {
            let point = Point3D::new([x, y, z]);
            let color = Color::new([r, g, b]);
            (point, color)
        }).collect::<Vec<_>>();

        Self::builder().points(points).build()
    }
}

/// Ply形式のデータを生成するために必要な情報を持つ構造体
/// 使用するには`ply`featureを有効にしてください。
#[derive(Clone, Debug, Default)]
pub struct PlyStructs {
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
}

impl PlyStructs {
    /// 頂点と面を指定してインスタンスを生成します。
    pub fn new(vertices: Vec<Vertex>, faces: Vec<Face>) -> Self {
        Self {
            vertices,
            faces,
        }
    }


    /// [`VoxelMesh`]からインスタンスを生成
    pub fn from_voxel_mesh<P: Int, C: UInt>(voxel_mesh: VoxelMesh<P, C>) -> Self
    where
        P: Int + AsPrimitive<f32>,
        C: UInt + AsPrimitive<f32>,
        f32: AsPrimitive<P> + AsPrimitive<C>,
        u8: AsPrimitive<C>,
    {
        let VoxelMesh { points, faces, offset, resolution, .. } = voxel_mesh;

        let points = points.into_iter().map(|p| (p + offset).as_() * resolution as f32).collect::<Vec<_>>();

        let mut vertex_set = IndexSet::<Vertex, FxBuildHasher>::with_hasher(Default::default());

        let faces = faces.into_iter().flat_map(|(color, vertex_ids)| {
            let color = (color.as_::<f32>() / C::max_value().as_() * u8::MAX as f32).as_::<u8>();

            let [r, g, b] = color.data;

            let vertex_ids = vertex_ids.into_iter().map(|id| {
                let point = points[id];
                let x = OrderedFloat::from(point[0]);
                let y = OrderedFloat::from(point[1]);
                let z = OrderedFloat::from(point[2]);

                let vertex = Vertex { x, y, z, r, g, b };

                vertex_set.insert_full(vertex).0 as u32
            }).collect::<Vec<_>>();

            vertex_ids.chunks(3).map(|chunk| {
                Face { vertex_indices: chunk.to_vec() }
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();


        Self {
            vertices: vertex_set.into_iter().collect(),
            faces,
        }
    }

    /// ASCII形式のplyファイルのバッファを返します。
    pub fn into_ascii_buf(self) -> Vec<u8> {
        let mut ply = {
            let mut ply = Ply::<DefaultElement>::new();
            ply.header.encoding = Encoding::Ascii;

            // 要素の定義
            let mut vertex_element = ElementDef::new("vertex".to_string());
            [
                PropertyDef::new("x".to_string(), PropertyType::Scalar(ScalarType::Float)),
                PropertyDef::new("y".to_string(), PropertyType::Scalar(ScalarType::Float)),
                PropertyDef::new("z".to_string(), PropertyType::Scalar(ScalarType::Float)),
                PropertyDef::new("red".to_string(), PropertyType::Scalar(ScalarType::UChar)),
                PropertyDef::new("green".to_string(), PropertyType::Scalar(ScalarType::UChar)),
                PropertyDef::new("blue".to_string(), PropertyType::Scalar(ScalarType::UChar)),
            ].into_iter().for_each(|p| vertex_element.properties.add(p));

            let mut face_element = ElementDef::new("face".to_string());
            [
                PropertyDef::new("vertex_indices".to_string(), PropertyType::List(ScalarType::UChar, ScalarType::UInt)),
            ].into_iter().for_each(|p| face_element.properties.add(p));

            [vertex_element, face_element]
                .into_iter().for_each(|e| ply.header.elements.add(e));

            let vertex = self.vertices.into_iter().map(
                |Vertex { x, y, z, r, g, b }| {
                    DefaultElement::from_iter([
                        ("x".to_string(), Float(x.into_inner())),
                        ("y".to_string(), Float(y.into_inner())),
                        ("z".to_string(), Float(z.into_inner())),
                        ("red".to_string(), UChar(r)),
                        ("green".to_string(), UChar(g)),
                        ("blue".to_string(), UChar(b))])
                }).collect::<Vec<_>>();

            ply.payload.insert("vertex".to_string(), vertex);

            let face = self.faces.into_iter().map(
                |Face { vertex_indices }| {
                    DefaultElement::from_iter([("vertex_indices".to_string(), ListUInt(vertex_indices))])
                }).collect::<Vec<_>>();

            ply.payload.insert("face".to_string(), face);

            ply
        };

        let mut buf = Vec::<u8>::new();
        let writer = Writer::new();
        writer.write_ply(&mut buf, &mut ply).unwrap();

        buf
    }
}
