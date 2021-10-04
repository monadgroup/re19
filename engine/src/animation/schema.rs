use super::property::PropertyType;
use crate::animation::timeline::ClipSource;
use crate::animation::timeline::{Clip, PropertyDefault, PropertyGroup};
use crate::creation_context::CreationContext;
use crate::generator::Generator;
use alloc::boxed::Box;
use alloc::string::ToString;

#[derive(Clone, Copy)]
pub struct GeneratorSchema {
    #[cfg(debug_assertions)]
    pub name: &'static str,
    pub instantiate_generator: fn(&mut CreationContext) -> Box<dyn Generator>,
    pub groups: &'static [SchemaGroup],
}

#[derive(Clone, Copy)]
pub struct SchemaGroup {
    #[cfg(debug_assertions)]
    pub name: &'static str,
    pub properties: &'static [SchemaProperty],
}

impl SchemaGroup {
    pub fn instantiate(&self) -> PropertyGroup {
        PropertyGroup {
            defaults: self
                .properties
                .iter()
                .map(|property| property.instantiate())
                .collect(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SchemaProperty {
    #[cfg(debug_assertions)]
    pub name: &'static str,
    pub value_type: PropertyType,
}

impl SchemaProperty {
    pub fn instantiate(&self) -> PropertyDefault {
        PropertyDefault {
            value: self.value_type.default_value(),
            is_override: false,
        }
    }
}

impl GeneratorSchema {
    pub fn instantiate(
        &'static self,
        id: u32,
        offset_frames: u32,
        duration_frames: u32,
        context: &mut CreationContext,
    ) -> Clip {
        Clip {
            id,
            #[cfg(debug_assertions)]
            name: self.name.to_string(),
            schema: self,
            source: ClipSource::Generator((self.instantiate_generator)(context)),
            offset_frames,
            duration_frames,
            property_groups: self
                .groups
                .iter()
                .map(|group| group.instantiate())
                .collect(),
            is_selected: false,
        }
    }
}
