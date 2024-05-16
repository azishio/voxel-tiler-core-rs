use std::io::{BufReader, Read};

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use ordered_float::NotNan;
use ply_rs::parser::Parser;
use ply_rs::ply::{Addable, DefaultElement, ElementDef, Encoding, Ply, Property, PropertyAccess, PropertyDef, PropertyType, ScalarType};
use ply_rs::ply::Property::{Float, Int, ListInt, UChar};
use ply_rs::writer::Writer;
use vec_x::VecX;

use crate::VoxelMesh;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
struct Vertex {
    x: NotNan<f32>,
    y: NotNan<f32>,
    z: NotNan<f32>,
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
            (k, _) => panic!("Vertex: Unexpected key/value combination: key: {}", k),
        }
    }
}

impl From<VecX<f32, 3>> for Vertex {
    fn from(v: VecX<f32, 3>) -> Self {
        Vertex {
            x: NotNan::new(v[0]).expect("Vertex: x is NaN"),
            y: NotNan::new(v[1]).expect("Vertex: x is NaN"),
            z: NotNan::new(v[2]).expect("Vertex: x is NaN"),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
struct Material {
    r: u8,
    g: u8,
    b: u8,
}

impl PropertyAccess for Material {
    fn new() -> Self {
        Material::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("ambient_red", UChar(v)) => self.r = v,
            ("ambient_green", UChar(v)) => self.g = v,
            ("ambient_blue", UChar(v)) => self.b = v,
            (k, _) => panic!("RGB: Unexpected key/value combination: key: {}", k),
        }
    }
}

impl From<VecX<u8, 3>> for Material {
    fn from(v: VecX<u8, 3>) -> Self {
        Material {
            r: v[0],
            g: v[1],
            b: v[2],
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
struct Face {
    vertex_indices: Vec<i32>,
    material_index: i32,
}

impl PropertyAccess for Face {
    fn new() -> Self {
        Face::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("vertex_index", ListInt(v)) => self.vertex_indices = v,
            ("material_index", Int(v)) => self.material_index = v,
            (k, _) => panic!("Face: Unexpected key/value combination: key: {}", k),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct PlyStructs {
    vertices: Vec<Vertex>,
    materials: Vec<Material>,
    faces: Vec<Face>,
}

impl PlyStructs {
    pub fn new(vertices: Vec<Vertex>, materials: Vec<Material>, faces: Vec<Face>) -> Self {
        Self {
            vertices,
            materials,
            faces,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        let Self { vertices: vertexes, materials: colors, faces } = self;
        let mut vertex_set = IndexSet::<Vertex, FxBuildHasher>::from_iter(vertexes);
        let mut material_set = IndexSet::<Material, FxBuildHasher>::from_iter(colors);
        let mut face_set = IndexSet::<Face, FxBuildHasher>::from_iter(faces);

        other.faces.into_iter().for_each(|face| {
            let vertex_index = face.vertex_indices.into_iter().map(|i| {
                let vertex = other.vertices[i as usize];

                vertex_set.insert_full(vertex).0 as i32
            }).collect::<Vec<_>>();

            let color_index = material_set.insert_full(other.materials[face.material_index as usize]).0 as i32;

            face_set.insert(Face { vertex_indices: vertex_index, material_index: color_index });
        });

        PlyStructs {
            vertices: vertex_set.into_iter().collect(),
            materials: material_set.into_iter().collect(),
            faces: face_set.into_iter().collect(),
        }
    }

    pub fn from_ply<T: Read>(file: T) -> Self {
        let mut buf_reader = BufReader::new(file);

        let vertex_parser = Parser::<Vertex>::new();
        let face_parser = Parser::<Face>::new();
        let material_parser = Parser::<Material>::new();

        let header = vertex_parser.read_header(&mut buf_reader).unwrap();

        let mut vertex_list = Vec::new();
        let mut face_list = Vec::new();
        let mut material_list = Vec::new();

        header.elements.iter().for_each(|(_, element)| {
            match element.name.as_ref() {
                "vertex" => vertex_list = vertex_parser.read_payload_for_element(&mut buf_reader, &element, &header).unwrap(),
                "face" => face_list = face_parser.read_payload_for_element(&mut buf_reader, &element, &header).unwrap(),
                "material" => material_list = material_parser.read_payload_for_element(&mut buf_reader, &element, &header).unwrap(),
                _ => {}
            }
        });

        Self::new(vertex_list, material_list, face_list)
    }

    pub fn from_voxel_mesh(voxel_mesh: VoxelMesh<f32>) -> Self {
        let VoxelMesh {
            vertices,
            materials,
            face,
            ..
        } = voxel_mesh;

        let vertices = vertices.into_iter().map(Vertex::from).collect::<Vec<_>>();
        let materials = materials.into_iter().map(Material::from).collect::<Vec<_>>();
        let faces = face.into_iter().map(|(vertex_indices, material_index)| {
            let vertex_indices = vertex_indices.into_iter().map(|i| i as i32).collect::<Vec<_>>();
            let material_index = material_index as i32;

            Face {
                vertex_indices,
                material_index,
            }
        }).collect::<Vec<_>>();


        Self {
            vertices,
            materials,
            faces,
        }
    }

    pub fn to_buf(self) -> Vec<u8> {
        let mut buf = Vec::<u8>::new();

        let mut ply = {
            let mut ply = Ply::<DefaultElement>::new();
            ply.header.encoding = Encoding::BinaryLittleEndian;

            // 要素の定義
            let mut vertex_element = ElementDef::new("vertex".to_string());
            [
                PropertyDef::new("x".to_string(), PropertyType::Scalar(ScalarType::Double)),
                PropertyDef::new("y".to_string(), PropertyType::Scalar(ScalarType::Double)),
                PropertyDef::new("z".to_string(), PropertyType::Scalar(ScalarType::Double)),
            ].into_iter().for_each(|p| vertex_element.properties.add(p));

            let mut face_element = ElementDef::new("face".to_string());
            [
                PropertyDef::new("vertex_indices".to_string(), PropertyType::List(ScalarType::UChar, ScalarType::Int)),
                PropertyDef::new("material_index".to_string(), PropertyType::Scalar(ScalarType::Int)),
            ].into_iter().for_each(|p| face_element.properties.add(p));

            let mut material_element = ElementDef::new("material".to_string());
            [
                PropertyDef::new("ambient_red".to_string(), PropertyType::Scalar(ScalarType::UChar)),
                PropertyDef::new("ambient_green".to_string(), PropertyType::Scalar(ScalarType::UChar)),
                PropertyDef::new("ambient_blue".to_string(), PropertyType::Scalar(ScalarType::UChar)),
            ].into_iter().for_each(|p| material_element.properties.add(p));

            [vertex_element, face_element, material_element]
                .into_iter().for_each(|e| ply.header.elements.add(e));

            // データの追加
            let vertex = self.vertices.into_iter().map(|Vertex { x, y, z }| {
                DefaultElement::from_iter([("x".to_string(), Float(x.into_inner())), ("y".to_string(), Float(y.into_inner())), ("z".to_string(), Float(z.into_inner()))])
            }).collect::<Vec<_>>();
            ply.payload.insert("vertex".to_string(), vertex);

            let face = self.faces.into_iter().map(|Face { vertex_indices: vertex_index, material_index }| {
                DefaultElement::from_iter([("vertex_indices".to_string(), ListInt(vertex_index)), ("material_index".to_string(), Int(material_index))])
            }).collect::<Vec<_>>();
            ply.payload.insert("face".to_string(), face);

            let material = self.materials.into_iter().map(|Material { r, g, b }| {
                DefaultElement::from_iter([("ambient_red".to_string(), UChar(r)), ("ambient_green".to_string(), UChar(g)), ("ambient_blue".to_string(), UChar(b))])
            }).collect::<Vec<_>>();
            ply.payload.insert("material".to_string(), material);

            ply
        };

        let writer = Writer::new();
        writer.write_ply(&mut buf, &mut ply).unwrap();

        buf
    }
}
