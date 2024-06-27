use std::io::{BufReader, Read};

use num::cast::AsPrimitive;
use ordered_float::OrderedFloat;
use ply_rs::parser::Parser;
use ply_rs::ply::{Addable, DefaultElement, ElementDef, Encoding, Ply, Property, PropertyAccess, PropertyDef, PropertyType, ScalarType};
use ply_rs::ply::Property::{Float, ListUInt, UChar};
use ply_rs::writer::Writer;

use crate::element::{Color, Int, Point3D, UInt};
use crate::mesher::VoxelMesh;

/// Structure representing a single vertex in a Ply file
///
/// Plyファイルにおける1つの頂点を表す構造体
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
            ("x", Float(v)) => self.x = v,
            ("y", Float(v)) => self.y = v,
            ("z", Float(v)) => self.z = v,
            ("red", UChar(v)) => self.r = v,
            ("green", UChar(v)) => self.g = v,
            ("blue", UChar(v)) => self.b = v,
            (k, _) => if cfg!(feature = "print-log") { println!("[warn] Vertex: Unexpected key/value combination: key: {}", k) },
        }
    }
}



/// Structure representing a single face in a Ply file
///
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

/// Structure with information necessary to generate Ply format data
///
/// Ply形式のデータを生成するために必要な情報を持つ構造体
#[derive(Clone, Debug, Default)]
pub struct PlyStructs {
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
}

impl PlyStructs {
    /// Generate PlyStructs with vertices and faces
    ///
    /// 頂点と面を指定してPlyStructsを生成
    pub fn new(vertices: Vec<Vertex>, faces: Vec<Face>) -> Self {
        Self {
            vertices,
            faces,
        }
    }


    pub fn from_ply<T: Read>(file: T) -> Self {
        let mut buf_reader = BufReader::new(file);

        let vertex_parser = Parser::<Vertex>::new();
        let face_parser = Parser::<Face>::new();

        let header = vertex_parser.read_header(&mut buf_reader).unwrap();

        let mut vertex_list = Vec::new();
        let mut face_list = Vec::new();

        header.elements.iter().for_each(|(_, element)| {
            match element.name.as_ref() {
                "vertex" => vertex_list = vertex_parser.read_payload_for_element(&mut buf_reader, element, &header).unwrap(),
                "face" => face_list = face_parser.read_payload_for_element(&mut buf_reader, element, &header).unwrap(),
                _ => {}
            }
        });

        Self::new(vertex_list, face_list)
    }

    /// Generate PlyStructs from VoxelMesh
    ///
    /// VoxelMeshからPlyStructsを生成
    pub fn from_voxel_mesh<P:Int,C:UInt>(voxel_mesh: VoxelMesh<P,C>) -> Self
    where
        P: Int + AsPrimitive<f32>,
        C: UInt + AsPrimitive<f32>,
        f32: AsPrimitive<P> + AsPrimitive<C>,
    {
        let VoxelMesh { vertices, faces,offset,.. } = voxel_mesh;

        let (vertex_list, face_list):(Vec<_>,Vec<_>) = faces.into_iter().map(|(color, vertex_ids)| {
            let color = color.as_::<f32>();
            let max: f32 = C::max_value().as_();

            let r = ((color[0] / max)*u8::MAX as f32) as u8;
            let g = ((color[1] / max)*u8::MAX as f32) as u8;
            let b = ((color[2] / max)*u8::MAX as f32) as u8;

            let vertex_list = vertex_ids.iter()
                .filter_map(|&vertex_ids| {
                    if let Some(&vertex) = vertices.get_index(vertex_ids){
                        let x = (vertex[0]+offset[0]).as_();
                        let y = (vertex[1]+offset[1]).as_();
                        let z = (vertex[2]+offset[2]).as_();
                        
                        Some(Vertex {
                            x,
                            y,
                            z,
                            r,
                            g,
                            b,
                        })
                    }else {
                        None
                    }
                }).collect::<Vec<_>>();
            
                let face_list = vertex_ids.chunks(3).map(|rect| {
                    Face{
                        vertex_indices: rect.iter().map(|&v|v as u32).collect::<Vec<_>>()
                    }
                }).collect::<Vec<_>>();


            (vertex_list, face_list)
        }).unzip();
        
        let vertices = vertex_list.into_iter().flatten().collect::<Vec<_>>();
        let faces = face_list.into_iter().flatten().collect::<Vec<_>>();

        Self {
            vertices,
            faces,
        }
    }
    
    pub fn into_points(self)->Vec<(Point3D<OrderedFloat<f32>>,Color<u8>)>{
        self.vertices.into_iter().map(|Vertex{x,y,z,r,g,b}|{
            let x= OrderedFloat::from(x);
            let y= OrderedFloat::from(y);
            let z= OrderedFloat::from(z);
            
            let point = Point3D::new([x,y,z]);
            let color = Color::new([r,g,b]);
            (point,color)
        }).collect::<Vec<_>>()
    }

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

            // データの追加
            let vertex = self.vertices.into_iter().map(
                |Vertex { x, y, z, r, g, b }| {
                    DefaultElement::from_iter([
                        ("x".to_string(), Float(x)), ("y".to_string(), Float(y)), ("z".to_string(), Float(z)), ("red".to_string(), UChar(r)), ("green".to_string(), UChar(g)), ("blue".to_string(), UChar(b))])
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
