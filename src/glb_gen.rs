use std::borrow::Cow::Owned;
use std::collections::BTreeMap;
use std::default::Default;
use std::mem;

use anyhow::anyhow;
use gltf::{Glb, Semantic};
use gltf::binary::Header;
use gltf::buffer::Target::{ArrayBuffer, ElementArrayBuffer};
use gltf::json::{Accessor, Asset, Buffer, Material, Mesh, Node, Root, Scene, Value};
use gltf::json::accessor::{ComponentType, GenericComponentType, Type};
use gltf::json::buffer::{Stride, View};
use gltf::json::material::{PbrBaseColorFactor, PbrMetallicRoughness};
use gltf::json::mesh::Primitive;
use gltf::json::validation::Checked::Valid;
use gltf::json::validation::USize64;
use gltf::mesh::Mode;
use num::cast::AsPrimitive;

use crate::element;
use crate::element::{Int, UInt};
use crate::glb_gen::private::GlbGenPrivateMethod;
use crate::mesher::VoxelMesh;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct Vertex([f32; 3]);

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct Color([f32; 4]);

impl Color {
    pub fn from_srgb<C: UInt>(color: element::Color<C>) -> Self
    where
        C: UInt + AsPrimitive<f32>,
        f32: AsPrimitive<C>,
    {
        let color = color.as_::<f32>();
        let max: f32 = C::max_value().as_();

        // sRGBからリニアRGBに近似
        let r = (color[0] / max).powf(2.2);
        let g = (color[1] / max).powf(2.2);
        let b = (color[2] / max).powf(2.2);
        let a = 1.;

        Self([r, g, b, a])
    }
}

mod private {
    use std::mem;

    pub trait GlbGenPrivateMethod {
        fn convert_to_byte_vec<T>(vec: Vec<T>) -> Vec<u8> {
            let byte_length = vec.len() * mem::size_of::<T>();
            let alloc = vec.into_boxed_slice();
            let ptr = Box::<[T]>::into_raw(alloc) as *mut u8;
            //　`Vec::into_boxed_slice`によって、余分な容量を破棄しているので、`byte_capacity`は`byte_length`と同じ
            unsafe { Vec::from_raw_parts(ptr, byte_length, byte_length) }
        }

        // 要素数が4の倍数になるようにdefault値で埋める
        fn pad_to_mul_of_four<T: Default + Clone>(mut vec: Vec<T>) -> Vec<T> {
            let remainder = vec.len() % 4;

            if remainder != 0 {
                vec.append(&mut vec![T::default(); 4 - remainder])
            }

            vec
        }

        // n以上の最小の4の倍数に切り上げる
        // bit演算は使わない
        fn round_up_to_mul_of_four(n: usize) -> usize {
            let remainder = n % 4;
            if remainder == 0 {
                n
            } else {
                n + 4 - remainder
            }
        }
    }
}


pub trait GlbGen<'a>: GlbGenPrivateMethod {
    fn generate<P, C>(voxel_mesh: VoxelMesh<P, C>) -> Result<Glb<'a>, anyhow::Error>
    where
        P: Int + AsPrimitive<f32>,
        C: UInt + AsPrimitive<f32>,
        f32: AsPrimitive<P> + AsPrimitive<C>,
    {
        let mut root = Root::default();

        root.asset = Asset{
            copyright: None,
            extensions: None,
            extras: Default::default(),
            generator: None,
            min_version: None,
            version: "2.0".to_string(),
        };

        let vertices = voxel_mesh.vertices.into_iter().map(|point| {
            let point = point.as_();
            Vertex([point[0], point[1], point[2]])
        }).collect::<Vec<_>>();
        
        
        
        let vertices_length = Self::round_up_to_mul_of_four(vertices.len()) * mem::size_of::<Vertex>();

        let (colors, indices): (Vec<_>, Vec<_>) = voxel_mesh.faces.into_iter().map(|(color, vertex_ids)| {
            
            let color = Color::from_srgb(color);
            let vertex_ids = vertex_ids.into_iter().map(|vertex_id| {
                vertex_id as u32
            }).collect::<Vec<_>>();

            (color, vertex_ids)
        }).unzip();
        let indices_length = indices.iter().fold(0, |len, vec| len + Self::round_up_to_mul_of_four(vec.len()) * mem::size_of::<u32>());

        let buffer_length = vertices_length + indices_length;
        let buffer = root.push(Buffer {
            byte_length: USize64::from(buffer_length),
            name: None,
            uri: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        let vertices_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(vertices_length),
            byte_offset: None,
            byte_stride: Some(Stride(mem::size_of::<Vertex>())),
            name: None,
            target: Some(Valid(ArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        });

        let indices_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(indices_length),
            byte_offset: Some(USize64::from(vertices_length)),
            byte_stride: None,
            name: None,
            target: Some(Valid(ElementArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        });

        let (min, max) = {
            let min = voxel_mesh.bounds.0.as_::<f32>();
            let max = voxel_mesh.bounds.1.as_::<f32>();

            let min = [min[0], min[1], min[2]];
            let max = [max[0], max[1], max[2]];

            (min, max)
        };
        

        let positions_accessor = root.push(Accessor {
            buffer_view: Some(vertices_buffer_view),
            byte_offset: Some(USize64(0)),
            count: USize64::from(vertices.len()),
            component_type: Valid(GenericComponentType(ComponentType::F32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(Type::Vec3),
            min: Some(Value::from(Vec::from(min))),
            max: Some(Value::from(Vec::from(max))),
            name: None,
            normalized: false,
            sparse: None,
        });

        let primitives = colors.into_iter().enumerate().map(|(i, color)| {
            let offset = (0..i).fold(0, |offset, i| offset + Self::round_up_to_mul_of_four(indices[i].len()));

            let indices_accessor = root.push(Accessor {
                buffer_view: Some(indices_buffer_view),
                byte_offset: Some(USize64::from(offset)),
                count: USize64::from(indices[i].len()),
                component_type: Valid(GenericComponentType(ComponentType::U32)),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Valid(Type::Scalar),
                min: None,
                max: None,
                name: None,
                normalized: false,
                sparse: None,
            });

            let pbr_metallic_roughness = PbrMetallicRoughness {
                base_color_factor: PbrBaseColorFactor(color.0),
                base_color_texture: None,
                metallic_factor: Default::default(),
                roughness_factor: Default::default(),
                metallic_roughness_texture: None,
                extensions: Default::default(),
                extras: Default::default(),
            };

            let material = root.push(Material {
                alpha_cutoff: None,
                alpha_mode: Default::default(),
                double_sided: false,
                name: None,
                pbr_metallic_roughness,
                normal_texture: None,
                occlusion_texture: None,
                emissive_texture: None,
                emissive_factor: Default::default(),
                extensions: Default::default(),
                extras: Default::default(),
            });


            Primitive {
                attributes: BTreeMap::from([(Valid(Semantic::Positions), positions_accessor)]),
                extensions: None,
                extras: Default::default(),
                indices: Some(indices_accessor),
                material: Some(material),
                mode: Valid(Mode::Triangles),
                targets: None,
            }
        }).collect::<Vec<_>>();

        let mesh = root.push(Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            primitives,
            weights: None,
        });

        let node = root.push(Node {
            mesh: Some(mesh),
            translation: Some(voxel_mesh.offset.as_::<f32>().data),
            ..Default::default()
        });

        root.push(Scene {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            nodes: vec![node],
        });


        let json = root.to_string().map_err(|_| anyhow!("Serialization error"))?.into_bytes();
        let json_offset = Self::round_up_to_mul_of_four(json.len());

        let bin = [
            Self::convert_to_byte_vec(Self::pad_to_mul_of_four(vertices)),
            indices.into_iter().flat_map(|v| Self::convert_to_byte_vec(Self::pad_to_mul_of_four(v))).collect::<Vec<_>>(),
        ].concat();

        Ok(Glb {
            header: Header {
                magic: *b"glTF",
                version: 2,
                length: (json_offset + buffer_length).try_into().map_err(|_| anyhow!("file size exceeds binary glTF limit"))?,
            },
            json: Owned(json),
            bin: Some(Owned(bin)),
        })
    }
}

impl GlbGenPrivateMethod for Glb<'_> {}

impl GlbGen<'_> for Glb<'_> {}