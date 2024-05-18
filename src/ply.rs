use std::io::{BufReader, Read};

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use ordered_float::NotNan;
use ply_rs::parser::Parser;
use ply_rs::ply::{Addable, DefaultElement, ElementDef, Encoding, Ply, Property, PropertyAccess, PropertyDef, PropertyType, ScalarType};
use ply_rs::ply::Property::{Float, ListInt, UChar};
use ply_rs::writer::Writer;
use vec_x::VecX;

use crate::{Point, VoxelMesh};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Vertex {
    x: NotNan<f32>,
    y: NotNan<f32>,
    z: NotNan<f32>,
    r: u8,
    g: u8,
    b: u8,
}

impl PropertyAccess for Vertex {
    fn new() -> Self {
        Vertex::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("x", Float(v)) => self.x = NotNan::new(v).expect("Vertex: x is NaN"),
            ("y", Float(v)) => self.y = NotNan::new(v).expect("Vertex: x is NaN"),
            ("z", Float(v)) => self.z = NotNan::new(v).expect("Vertex: x is NaN"),
            ("red", UChar(v)) => self.r = v,
            ("green", UChar(v)) => self.g = v,
            ("blue", UChar(v)) => self.b = v,
            (k, _) => panic!("Vertex: Unexpected key/value combination: key: {}", k),
        }
    }
}

impl From<Point<f32>> for Vertex {
    fn from((coord, material_index): Point<f32>) -> Self {
        Vertex {
            x: NotNan::new(coord[0]).expect("Vertex: x is NaN"),
            y: NotNan::new(coord[1]).expect("Vertex: x is NaN"),
            z: NotNan::new(coord[2]).expect("Vertex: x is NaN"),
            r: material_index[0],
            g: material_index[1],
            b: material_index[2],
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Face {
    vertex_indices: Vec<i32>,
}

impl PropertyAccess for Face {
    fn new() -> Self {
        Face::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("vertex_index", ListInt(v)) => self.vertex_indices = v,
            (k, _) => panic!("Face: Unexpected key/value combination: key: {}", k),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PlyStructs {
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
}

impl PlyStructs {
    pub fn new(vertices: Vec<Vertex>, faces: Vec<Face>) -> Self {
        Self {
            vertices,
            faces,
        }
    }

    pub fn marge(ply_list: Vec<Self>) -> Self {
        let mut vertex_set = IndexSet::<Vertex, FxBuildHasher>::with_hasher(FxBuildHasher::default());
        let mut face_set = IndexSet::<Face, FxBuildHasher>::with_hasher(FxBuildHasher::default());

        ply_list.into_iter().for_each(|ply| {
            ply.faces.into_iter().for_each(|face| {
                let vertex_index = face.vertex_indices.into_iter().map(|i| {
                    let vertex = ply.vertices[i as usize];

                    vertex_set.insert_full(vertex).0 as i32
                }).collect::<Vec<_>>();

                face_set.insert(Face { vertex_indices: vertex_index });
            });
        });

        PlyStructs {
            vertices: vertex_set.into_iter().collect(),
            faces: face_set.into_iter().collect(),
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
                "vertex" => vertex_list = vertex_parser.read_payload_for_element(&mut buf_reader, &element, &header).unwrap(),
                "face" => face_list = face_parser.read_payload_for_element(&mut buf_reader, &element, &header).unwrap(),
                _ => {}
            }
        });

        Self::new(vertex_list, face_list)
    }

    pub fn from_voxel_mesh(voxel_mesh: VoxelMesh<f32>) -> Self {
        let VoxelMesh {
            vertices,
            face,
            ..
        } = voxel_mesh;

        let vertices = vertices.into_iter().map(Vertex::from).collect::<Vec<_>>();
        let faces = face.into_iter().map(|vertex_indices| {
            let vertex_indices = vertex_indices.into_iter().map(|i| i as i32).collect::<Vec<_>>();

            Face {
                vertex_indices,
            }
        }).collect::<Vec<_>>();


        Self {
            vertices,
            faces,
        }
    }

    pub fn to_ascii_ply_buf(self) -> Vec<u8> {
        self.into_buf(Encoding::Ascii)
    }

    pub fn to_binary_little_endian_ply_buf(self) -> Vec<u8> {
        self.into_buf(Encoding::BinaryLittleEndian)
    }

    pub fn to_binary_big_endian_ply_buf(self) -> Vec<u8> {
        self.into_buf(Encoding::BinaryBigEndian)
    }

    fn into_buf(self, encoding: Encoding) -> Vec<u8> {
        let mut buf = Vec::<u8>::new();

        let mut ply = {
            let mut ply = Ply::<DefaultElement>::new();
            ply.header.encoding = encoding;

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
                PropertyDef::new("vertex_indices".to_string(), PropertyType::List(ScalarType::UChar, ScalarType::Int)),
            ].into_iter().for_each(|p| face_element.properties.add(p));

            [vertex_element, face_element]
                .into_iter().for_each(|e| ply.header.elements.add(e));

            // データの追加
            let vertex = self.vertices.into_iter().map(|Vertex { x, y, z, r, g, b }| {
                DefaultElement::from_iter([
                    ("x".to_string(), Float(x.into_inner())), ("y".to_string(), Float(y.into_inner())), ("z".to_string(), Float(z.into_inner())), ("red".to_string(), UChar(r)), ("green".to_string(), UChar(g)), ("blue".to_string(), UChar(b))])
            }).collect::<Vec<_>>();
            ply.payload.insert("vertex".to_string(), vertex);

            let face = self.faces.into_iter().map(|Face { vertex_indices: vertex_index }| {
                DefaultElement::from_iter([("vertex_indices".to_string(), ListInt(vertex_index))])
            }).collect::<Vec<_>>();
            ply.payload.insert("face".to_string(), face);

            ply
        };

        let writer = Writer::new();
        writer.write_ply(&mut buf, &mut ply).unwrap();

        buf
    }
}
