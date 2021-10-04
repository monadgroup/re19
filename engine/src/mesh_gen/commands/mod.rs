mod cube_command;
mod cylinder_command;
mod duplicate_command;
mod flat_normals_command;
mod quad_command;
mod remap_uv_command;
mod smooth_normals_command;
mod sphere_command;
mod subdivide_command;
mod sublist_command;

use super::{Executor, MeshHandle, Selection};
use crate::animation::property::PropertyType;
use crate::math::{Quaternion, RgbColor, RgbaColor, Vector2, Vector3, Vector4};
use crate::mesh_gen::ListRef;
use alloc::boxed::Box;

#[cfg(feature = "tool")]
type BoxedCommand = Box<dyn EditorCommand>;

#[cfg(not(feature = "tool"))]
type BoxedCommand = Box<dyn Command>;

macro_rules! commands {
    ($($enum_type:ident => $module:ident::$struct_name:ident),*) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum CommandType {
            $($enum_type,)*
        }

        pub static COMMAND_TYPES: &[CommandType] = &[
            $(CommandType::$enum_type,)*
        ];

        impl CommandType {
            pub fn instantiate(self) -> BoxedCommand {
                match self {
                    $(
                    CommandType::$enum_type => Box::new($module::$struct_name::default()),
                    )*
                }
            }

            #[cfg(feature = "tool")]
            pub fn name(self) -> &'static str {
                match self {
                    $(
                    CommandType::$enum_type => stringify!($enum_type),
                    )*
                }
            }

            #[cfg(feature = "tool")]
            pub fn schema(self) -> &'static EditorCommandSchema {
                match self {
                    $(
                    CommandType::$enum_type => $module::$struct_name::schema(),
                    )*
                }
            }
        }
    }
}

commands! {
    Quad => quad_command::QuadCommand,
    Cube => cube_command::CubeCommand,
    Cylinder => cylinder_command::CylinderCommand,
    Sphere => sphere_command::SphereCommand,

    Sublist => sublist_command::SublistCommand,
    Duplicate => duplicate_command::DuplicateCommand,

    FlatNormals => flat_normals_command::FlatNormalsCommand,
    SmoothNormals => smooth_normals_command::SmoothNormalsCommand,
    RemapUv => remap_uv_command::RemapUvCommand,
    Subdivide => subdivide_command::SubdivideCommand
}

pub trait Command {
    fn run(&self, mesh: &mut MeshHandle, selection: &mut Selection, executor: &dyn Executor);
}

macro_rules! property_types {
    ($($enum_type:ident: $type:ident => ($val:ident, $val_mut:ident)),*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum EditorCommandPropertyType {
            $($enum_type,)*
        }

        impl EditorCommandProperty {
            $(
            pub unsafe fn $val<'r>(&self, obj: &'r [u8]) -> &'r $type {
                assert_eq!(self.val_type, EditorCommandPropertyType::$enum_type);
                &*(&obj[self.byte_offset] as *const u8 as *const $type)
            }

            pub unsafe fn $val_mut<'r>(&self, obj: &'r mut [u8]) -> &'r mut $type {
                assert_eq!(self.val_type, EditorCommandPropertyType::$enum_type);
                &mut *(&mut obj[self.byte_offset] as *mut u8 as *mut $type)
            }
            )*
        }
    }
}

pub struct EditorCommandProperty {
    pub name: &'static str,
    pub val_type: EditorCommandPropertyType,
    pub byte_offset: usize,
}

property_types! {
    Signed: i32 => (signed, signed_mut),
    Unsigned: u32 => (unsigned, unsigned_mut),
    Float: f32 => (float, float_mut),
    Vec2: Vector2 => (vec2, vec2_mut),
    Vec3: Vector3 => (vec3, vec3_mut),
    Vec4: Vector4 => (vec4, vec4_mut),
    RgbColor: RgbColor => (rgb_color, rgb_color_mut),
    RgbaColor: RgbaColor => (rgba_color, rgba_color_mut),
    Rotation: Quaternion => (rotation, rotation_mut),
    Reference: ListRef => (reference, reference_mut)
}

pub struct EditorCommandPragma {
    pub name: &'static str,
    pub property: usize,
}

pub struct EditorCommandDependency {
    pub property: usize,
}

pub struct EditorCommandSchema {
    pub properties: &'static [EditorCommandProperty],
    pub pragmas: &'static [EditorCommandPragma],
    pub dependencies: &'static [EditorCommandDependency],
}

pub trait EditorCommand: Command {
    fn update(&mut self);
    fn run(&self, mesh: &mut MeshHandle, selection: &mut Selection, executor: &dyn Executor);
    fn clone(&self) -> Box<dyn EditorCommand>;
    unsafe fn as_bytes_mut(&mut self) -> &mut [u8];
}
