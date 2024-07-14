use std::borrow::Cow::Owned;
use std::collections::BTreeMap;
use std::default::Default;
use std::mem;

use anyhow::anyhow;
use gltf::{Glb, Semantic};
use gltf::binary::Header;
use gltf::buffer::Target::{ArrayBuffer, ElementArrayBuffer};
use gltf::json::{Accessor, Buffer, Image, Material, Mesh, Node, Root, Scene, Texture, Value};
use gltf::json::accessor::{ComponentType, GenericComponentType, Type};
use gltf::json::buffer::{Stride, View};
use gltf::json::image::MimeType;
use gltf::json::material::{PbrBaseColorFactor, PbrMetallicRoughness};
use gltf::json::mesh::Primitive;
use gltf::json::texture::{Info, Sampler};
use gltf::json::validation::Checked::Valid;
use gltf::json::validation::USize64;
use gltf::mesh::Mode;
use gltf::texture::{MagFilter, MinFilter};
use num::cast::AsPrimitive;

use crate::element::{Int, UInt};
use crate::glb_gen::private::GlbGenPrivateMethod;
use crate::voxel_mesh::VoxelMesh;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct Vertex([f32; 3]);

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct UV([f32; 2]);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mime {
    ImageJpeg,
    ImagePng,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureInfo {
    pub buf: Option<Vec<u8>>,
    pub uri: Option<String>,
    pub mime_type: Mime,
}


mod private {
    use std::mem;

    use num::cast::AsPrimitive;

    use crate::element::{Color, UInt};

    pub trait GlbGenPrivateMethod {
        fn srgb_to_liner_rgba<C>(color: Color<C>) -> [f32; 4]
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

            [r, g, b, a]
        }
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

            vec.shrink_to_fit();

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
    fn from_voxel_mesh<P, C>(voxel_mesh: VoxelMesh<P, C>) -> Result<Glb<'a>, anyhow::Error>
    where
        P: Int + AsPrimitive<f32>,
        C: UInt + AsPrimitive<f32>,
        f32: AsPrimitive<P> + AsPrimitive<C>,
    {
        let mut root = Root::default();

        let vertices = voxel_mesh.points.into_iter().map(|point| {
            let [x, y, z] = point.as_().data;
            // gltfの座標系に合わせる
            Vertex([x, z, -y])
        }).collect::<Vec<_>>();

        let (colors, indices): (Vec<_>, Vec<_>) = voxel_mesh.faces.into_iter().map(|(color, vertex_ids)| {
            let color = Self::srgb_to_liner_rgba(color);
            let vertex_ids = vertex_ids.into_iter().map(|vertex_id| {
                vertex_id as u32
            }).collect::<Vec<_>>();

            (color, vertex_ids)
        }).unzip();

        let padded_vertices_length = Self::round_up_to_mul_of_four(vertices.len()) * mem::size_of::<Vertex>();
        let padded_indices_length = indices.iter().map(|v| Self::round_up_to_mul_of_four(v.len()) * mem::size_of::<u32>()).collect::<Vec<_>>();

        let buffer_length = padded_vertices_length + padded_indices_length.iter().sum::<usize>();
        let buffer = root.push(Buffer {
            byte_length: USize64::from(buffer_length),
            name: None,
            uri: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        let vertices_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(padded_vertices_length),
            byte_offset: None,
            byte_stride: Some(Stride(mem::size_of::<Vertex>())),
            name: None,
            target: Some(Valid(ArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        });

        let indices_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(padded_indices_length.iter().sum::<usize>()),
            byte_offset: Some(USize64::from(padded_vertices_length)),
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
            let offset = padded_indices_length[0..i].iter().sum::<usize>();

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
                base_color_factor: PbrBaseColorFactor(color),
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
            scale: Some([voxel_mesh.resolution as f32; 3]),
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

    fn from_voxel_mesh_with_texture_projected_z<P, C>(voxel_mesh: VoxelMesh<P, C>, texture: TextureInfo) -> Result<Glb<'a>, anyhow::Error>
    where
        P: Int + AsPrimitive<f32> + AsPrimitive<isize>,
        C: UInt + AsPrimitive<f32>,
        isize: AsPrimitive<P>,
        f32: AsPrimitive<P> + AsPrimitive<C>,
    {
        let vertices = voxel_mesh.points.iter().map(|point| {
            let [x, y, z] = point.as_().data;
            // gltfの座標系に合わせる
            Vertex([x, z, -y])
        }).collect::<Vec<_>>();

        let vertex_indices = voxel_mesh.faces.into_iter().flat_map(|(_color, vertex_ids)| {
            vertex_ids.into_iter().map(|vertex_id| vertex_id as u32)
        }).collect::<Vec<_>>();

        let uv = {
            let (min, max) = voxel_mesh.bounds;
            let offset = min + voxel_mesh.offset;

            println!("min: {:?}, max: {:?}", min, max);
            println!("size: {:?}", max - min);
            println!("offset: {:?}", offset);

            //vertex_indices.iter().map(|uv_id| {
            //    let p = (voxel_mesh.points[*uv_id as usize] - offset).as_::<isize>();

            //    let normalized = p.as_::<f32>() / (max - min).as_::<f32>();


            //    UV(normalized.fit::<2>().data)
            //}).collect::<Vec<_>>()

            voxel_mesh.points.iter().map(|&point| {
                let p = (point - offset).as_::<isize>();

                let normalized = p.as_::<f32>() / (max - min).as_::<f32>();

                UV(normalized.fit::<2>().data)
            }).collect::<Vec<_>>()
        };

        let padded_vertices_length = Self::round_up_to_mul_of_four(vertices.len()) * mem::size_of::<Vertex>();
        let padded_indices_length = Self::round_up_to_mul_of_four(vertex_indices.len()) * mem::size_of::<u32>();

        let padded_uv_length = Self::round_up_to_mul_of_four(uv.len()) * mem::size_of::<UV>();

        let texture_length = if let Some(buf) = &texture.buf {
            buf.len() * mem::size_of::<u8>()
        } else {
            0
        };

        let mut root = Root::default();

        let buffer_length = padded_vertices_length + padded_indices_length + padded_uv_length + texture_length;
        let buffer = root.push(Buffer {
            byte_length: USize64::from(buffer_length),
            name: None,
            uri: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        let vertices_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(padded_vertices_length),
            byte_offset: None,
            byte_stride: Some(Stride(mem::size_of::<Vertex>())),
            name: None,
            target: Some(Valid(ArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        });

        let (min, max) = {
            let min = voxel_mesh.bounds.0.as_::<f32>();
            let max = voxel_mesh.bounds.1.as_::<f32>();

            (min.data, max.data)
        };

        let positions_accessor = root.push(Accessor {
            buffer_view: Some(vertices_buffer_view),
            byte_offset: None,
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

        let vertex_indices_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(padded_indices_length),
            byte_offset: Some(USize64::from(padded_vertices_length)),
            byte_stride: None,
            name: None,
            target: Some(Valid(ElementArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        });

        let vertex_indices_accessor = root.push(Accessor {
            buffer_view: Some(vertex_indices_buffer_view),
            byte_offset: None,
            count: USize64::from(vertex_indices.len()),
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

        let uv_buffer_view = root.push(View {
            buffer,
            byte_length: USize64::from(padded_uv_length),
            byte_offset: Some(USize64::from(padded_vertices_length + padded_indices_length)),
            byte_stride: Some(Stride(mem::size_of::<UV>())),
            name: None,
            target: Some(Valid(ArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        });

        let uv_accessor = root.push(Accessor {
            buffer_view: Some(uv_buffer_view),
            byte_offset: Some(USize64(0)),
            count: USize64::from(uv.len()),
            component_type: Valid(GenericComponentType(ComponentType::F32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(Type::Vec2),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        let texture_buffer_view = if texture.buf.is_some() {
            let view = root.push(View {
                buffer,
                byte_length: USize64::from(texture_length),
                byte_offset: Some(USize64::from(padded_vertices_length + padded_indices_length + padded_uv_length)),
                byte_stride: None,
                name: None,
                target: None,
                extensions: Default::default(),
                extras: Default::default(),
            });
            Some(view)
        } else {
            None
        };

        let mime_type = match texture.mime_type {
            Mime::ImageJpeg => "image/jpeg",
            Mime::ImagePng => "image/png",
        };

        let image = root.push(Image {
            buffer_view: texture_buffer_view,
            mime_type: Some(MimeType(mime_type.to_string())),
            name: None,
            uri: texture.uri,
            extensions: None,
            extras: Default::default(),
        });

        let sampler = root.push(Sampler {
            mag_filter: Some(Valid(MagFilter::Nearest)),
            min_filter: Some(Valid(MinFilter::Nearest)),
            name: None,
            wrap_s: Default::default(),
            wrap_t: Default::default(),
            extensions: None,
            extras: Default::default(),
        });

        let textures = root.push(Texture {
            sampler: Some(sampler),
            source: image,
            name: None,
            extensions: None,
            extras: Default::default(),
        });

        let tex_info = Info {
            index: textures,
            tex_coord: 0,
            extensions: None,
            extras: Default::default(),
        };

        let pbr_metallic_roughness = PbrMetallicRoughness {
            base_color_factor: PbrBaseColorFactor::default(),
            base_color_texture: Some(tex_info),
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


        let primitives = vec![Primitive {
            attributes: BTreeMap::from([
                (Valid(Semantic::Positions), positions_accessor),
                (Valid(Semantic::TexCoords(0)), uv_accessor)
            ]),
            extensions: None,
            extras: Default::default(),
            indices: Some(vertex_indices_accessor),
            material: Some(material),
            mode: Valid(Mode::Triangles),
            targets: None,
        }];

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
            scale: Some([voxel_mesh.resolution as f32; 3]),
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

        let mut bin = [
            Self::convert_to_byte_vec(Self::pad_to_mul_of_four(vertices)),
            Self::convert_to_byte_vec(Self::pad_to_mul_of_four(vertex_indices)),
            Self::convert_to_byte_vec(Self::pad_to_mul_of_four(uv)),
        ].concat();

        if let Some(buf) = texture.buf {
            bin.extend(buf);
        }

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

#[cfg(test)]
mod test {
    #[test]
    fn test_round_up_to_mul_of_four() {
        use crate::glb_gen::private::GlbGenPrivateMethod;
        struct TestStruct;
        impl GlbGenPrivateMethod for TestStruct {}

        assert_eq!(TestStruct::round_up_to_mul_of_four(0), 0);
        assert_eq!(TestStruct::round_up_to_mul_of_four(1), 4);
        assert_eq!(TestStruct::round_up_to_mul_of_four(2), 4);
        assert_eq!(TestStruct::round_up_to_mul_of_four(3), 4);
        assert_eq!(TestStruct::round_up_to_mul_of_four(4), 4);
        assert_eq!(TestStruct::round_up_to_mul_of_four(5), 8);
        assert_eq!(TestStruct::round_up_to_mul_of_four(6), 8);
        assert_eq!(TestStruct::round_up_to_mul_of_four(7), 8);
        assert_eq!(TestStruct::round_up_to_mul_of_four(8), 8);
    }

    #[test]
    fn test_pad_to_mul_of_four() {
        use crate::glb_gen::private::GlbGenPrivateMethod;
        struct TestStruct;
        impl GlbGenPrivateMethod for TestStruct {}

        assert_eq!(TestStruct::pad_to_mul_of_four(vec![1, 2, 3]), vec![1, 2, 3, 0]);
        assert_eq!(TestStruct::pad_to_mul_of_four(vec![1, 2, 3, 4]), vec![1, 2, 3, 4]);
        assert_eq!(TestStruct::pad_to_mul_of_four(vec![1, 2, 3, 4, 5]), vec![1, 2, 3, 4, 5, 0, 0, 0]);
    }
}
