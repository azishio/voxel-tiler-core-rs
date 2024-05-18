use std::io::{BufReader, Read};

use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use ordered_float::NotNan;
use ply_rs::parser::Parser;
use ply_rs::ply::{Addable, DefaultElement, ElementDef, Encoding, Ply, Property, PropertyAccess, PropertyDef, PropertyType, ScalarType};
use ply_rs::ply::Property::{Float, ListUInt, UChar};
use ply_rs::writer::Writer;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::{Point, VertexIndices, VoxelMesh};

/// Structure representing a single vertex in a Ply file
///
/// Plyファイルにおける1つの頂点を表す構造体
#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    r: u8,
    g: u8,
    b: u8,
}

impl FromIterator<HashableVertex> for Vec<Vertex> {
    fn from_iter<T: IntoIterator<Item=HashableVertex>>(iter: T) -> Vec<Vertex> {
        iter.into_iter().map(|v| Vertex {
            x: v.x.into_inner(),
            y: v.y.into_inner(),
            z: v.z.into_inner(),
            r: v.r,
            g: v.g,
            b: v.b,
        }).collect()
    }
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

impl From<Point<f32>> for Vertex {
    fn from((coord, material_index): Point<f32>) -> Self {
        Vertex {
            x: coord[0],
            y: coord[1],
            z: coord[2],
            r: material_index[0],
            g: material_index[1],
            b: material_index[2],
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
struct HashableVertex {
    x: NotNan<f32>,
    y: NotNan<f32>,
    z: NotNan<f32>,
    r: u8,
    g: u8,
    b: u8,
}

impl From<Vertex> for HashableVertex {
    fn from(Vertex { x, y, z, r, g, b }: Vertex) -> Self {
        HashableVertex {
            x: NotNan::new(x).expect("x is NaN"),
            y: NotNan::new(y).expect("y is NaN"),
            z: NotNan::new(z).expect("z is NaN"),
            r,
            g,
            b,
        }
    }
}

/// Structure representing a single face in a Ply file
///
/// Plyファイルにおける1つの面を表す構造体
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Face {
    vertex_indices: Vec<u32>,
}

impl PropertyAccess for Face {
    fn new() -> Self {
        Face::default()
    }

    fn set_property(&mut self, key: String, property: Property) {
        match (key.as_ref(), property) {
            ("vertex_indices", ListUInt(v)) => self.vertex_indices = v,
            (k, _) => if cfg!(feature = "print-log") { println!("[warn] Face: Unexpected key/value combination: key: {}", k) },
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

    /// Merge multiple PlyStructs into a single PlyStruct
    ///
    /// 複数のPlyStructsをマージして1つのPlyStructsを生成
    ///
    /// # Example
    ///
    /// ```
    /// use std::fs::File;
    /// use voxel_tiler_core::PlyStructs;
    ///
    /// let file = File::open("examples/data-source/box.ply").unwrap();
    /// let box_ply = PlyStructs::from_ply(file);
    ///
    /// let file = File::open("examples/data-source/cone.ply").unwrap();
    /// let cone_ply = PlyStructs::from_ply(file);
    ///
    /// let merged_ply = PlyStructs::marge(vec![box_ply, cone_ply]);
    /// ```
    pub fn marge(ply_list: Vec<Self>) -> Self {
        let mut vertex_set = IndexSet::<HashableVertex, FxBuildHasher>::with_hasher(FxBuildHasher::default());
        let mut face_set = IndexSet::<Face, FxBuildHasher>::with_hasher(FxBuildHasher::default());

        ply_list.into_iter().for_each(|ply| {
            ply.faces.into_iter().for_each(|face| {
                let vertex_indices = face.vertex_indices.into_iter().map(|i| {
                    let vertex = ply.vertices[i as usize].into();

                    vertex_set.insert_full(vertex).0 as u32
                }).collect::<Vec<_>>();

                face_set.insert(Face { vertex_indices });
            });
        });

        PlyStructs {
            vertices: vertex_set.into_iter().collect(),
            faces: face_set.into_iter().collect(),
        }
    }

    /// Generate PlyStructs from data according to Ply format
    ///
    /// Plyのフォーマットに沿ったデータからPlyStructsを生成
    ///
    /// Unsupported properties and elements are ignored, with warnings printed.
    /// If you set the `print-warning` feature flag, it will explicitly print that it has been ignored.
    ///
    /// 対応していないプロパティや要素は警告をプリントして無視されます。
    /// `print-warning`featureフラグ立てると無視したことを明示的にprintします。
    ///
    /// For actual behavior, see `examples/read_ply.rs`.
    ///
    /// 実際の動作は`examples/read_ply.rs`を参照してください。
    ///
    /// # Example
    ///
    /// ```
    ///  // Ascii PLY
    ///  use std::fs::File;
    ///  use voxel_tiler_core::PlyStructs;
    ///
    ///  let file = File::open("examples/data-source/box.ply").unwrap();
    ///  let ascii_ply = PlyStructs::from_ply(file);
    ///
    ///  // Binary PLY
    ///  let file = File::open("examples/data-source/binary_box.ply").unwrap();
    ///  let binary_ply = PlyStructs::from_ply(file);
    ///
    ///  // from buffer
    ///  let file: Vec<u8> = std::fs::read("examples/data-source/box.ply").unwrap();
    ///  let ply_by_buf = PlyStructs::from_ply(file.as_slice());
    /// ```
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
                _ => if cfg!(feature = "print-log") { println!("[warn] PlyStructs::from_ply: Unexpected element name: {}", element.name) },
            }
        });

        Self::new(vertex_list, face_list)
    }

    /// Generate PlyStructs from VoxelMesh
    ///
    /// VoxelMeshからPlyStructsを生成
    #[cfg(not(feature = "rayon"))]
    pub fn from_voxel_mesh(voxel_mesh: VoxelMesh<f32>) -> Self {
        let VoxelMesh {
            vertices,
            face,
            ..
        } = voxel_mesh;

        let vertices = vertices.into_iter().map(Vertex::from).collect::<Vec<_>>();

        let faces = face.into_iter().map(
            |vertex_indices| {
                let vertex_indices = vertex_indices.into_iter().map(|i| i as u32).collect::<Vec<_>>();

                Face {
                    vertex_indices,
                }
            }).collect::<Vec<_>>();

        Self {
            vertices,
            faces,
        }
    }

    /// Generate PlyStructs from VoxelMesh
    ///
    /// VoxelMeshからPlyStructsを生成
    #[cfg(feature = "rayon")]
    pub fn from_voxel_mesh(voxel_mesh: VoxelMesh<f32>) -> Self {
        let VoxelMesh {
            vertices,
            face,
            ..
        } = voxel_mesh;

        let vertices = vertices.into_par_iter().map(Vertex::from).collect::<Vec<_>>();

        let faces = face.into_par_iter().map(
            |vertex_indices| {
                let vertex_indices = vertex_indices.into_iter().map(|i| i as u32).collect::<Vec<_>>();

                Face {
                    vertex_indices,
                }
            }
        ).collect::<Vec<_>>();

        Self {
            vertices,
            faces,
        }
    }

    /// Generate buffer for Ply file in Ascii format.
    /// See `examples/write_voxel.rs` for an example of writing a buffer as a file.
    ///
    /// Ascii形式のPlyファイルのバッファを生成。
    /// バッファをファイルとして書き込む例は`examples/write_voxel.rs`を参照してください。
    pub fn to_ascii_ply_buf(self) -> Vec<u8> {
        self.into_buf(Encoding::Ascii)
    }

    /// Generate a buffer for a Ply file in binary (little edian) format.
    /// The method of writing to the file is the same as `to_ascii_ply_buf`.
    ///
    /// バイナリ(リトルエディアン)形式のPlyファイルのバッファを生成する。
    /// ファイルへの書き込み方法は`to_ascii_ply_buf`と同様です。
    pub fn to_binary_little_endian_ply_buf(self) -> Vec<u8> {
        self.into_buf(Encoding::BinaryLittleEndian)
    }

    /// Generate a buffer for a Ply file in binary (bigedian) format.
    /// The method of writing to the file is the same as `to_ascii_ply_buf`.
    ///
    /// バイナリ(ビッグエディアン)形式のPlyファイルのバッファを生成する。
    /// ファイルへの書き込み方法は`to_ascii_ply_buf`と同様です。
    pub fn to_binary_big_endian_ply_buf(self) -> Vec<u8> {
        self.into_buf(Encoding::BinaryBigEndian)
    }

    #[cfg(not(feature = "rayon"))]
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

        let writer = Writer::new();
        writer.write_ply(&mut buf, &mut ply).unwrap();

        buf
    }

    #[cfg(feature = "rayon")]
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
                PropertyDef::new("vertex_indices".to_string(), PropertyType::List(ScalarType::UChar, ScalarType::UInt)),
            ].into_iter().for_each(|p| face_element.properties.add(p));

            [vertex_element, face_element]
                .into_iter().for_each(|e| ply.header.elements.add(e));

            // データの追加
            let vertex = self.vertices.into_par_iter().map(
                |Vertex { x, y, z, r, g, b }| {
                    DefaultElement::from_iter([
                        ("x".to_string(), Float(x)), ("y".to_string(), Float(y)), ("z".to_string(), Float(z)), ("red".to_string(), UChar(r)), ("green".to_string(), UChar(g)), ("blue".to_string(), UChar(b))])
                }).collect::<Vec<_>>();

            ply.payload.insert("vertex".to_string(), vertex);

            let face = self.faces.into_par_iter().map(
                |Face { vertex_indices }| {
                    DefaultElement::from_iter([("vertex_indices".to_string(), ListUInt(vertex_indices))])
                }).collect::<Vec<_>>();

            ply.payload.insert("face".to_string(), face);

            ply
        };

        let writer = Writer::new();
        writer.write_ply(&mut buf, &mut ply).unwrap();

        buf
    }
}
