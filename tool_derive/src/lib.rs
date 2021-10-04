extern crate proc_macro;

use crate::proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use syn;
use syn::spanned::Spanned;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyType {
    Signed,
    Unsigned,
    Float,
    Vec2,
    Vec3,
    Vec4,
    RgbColor,
    RgbaColor,
    Rotation,
    Reference,
}

impl PropertyType {
    pub fn from_type_str(ty: &str) -> Option<PropertyType> {
        match ty {
            "i32" => Some(PropertyType::Signed),
            "u32" => Some(PropertyType::Unsigned),
            "f32" => Some(PropertyType::Float),
            "Vector2" => Some(PropertyType::Vec2),
            "Vector3" => Some(PropertyType::Vec3),
            "Vector4" => Some(PropertyType::Vec4),
            "RgbColor" => Some(PropertyType::RgbColor),
            "RgbaColor" => Some(PropertyType::RgbaColor),
            "Quaternion" => Some(PropertyType::Rotation),
            "ListRef" => Some(PropertyType::Reference),
            _ => None,
        }
    }

    pub fn into_ident(self, span: Span) -> syn::Ident {
        syn::Ident::new(
            match self {
                PropertyType::Signed => "Signed",
                PropertyType::Unsigned => "Unsigned",
                PropertyType::Float => "Float",
                PropertyType::Vec2 => "Vec2",
                PropertyType::Vec3 => "Vec3",
                PropertyType::Vec4 => "Vec4",
                PropertyType::RgbColor => "RgbColor",
                PropertyType::RgbaColor => "RgbaColor",
                PropertyType::Rotation => "Rotation",
                PropertyType::Reference => "Reference",
            },
            span.into(),
        )
    }
}

#[derive(Debug)]
struct Property {
    name: String,
    ident: syn::Ident,
    ty: PropertyType,
    ty_span: Span,
}

#[derive(Debug)]
struct Pragma {
    name: String,
    prop_index: usize,
}

#[derive(Debug)]
struct Dependency {
    prop_index: usize,
}

#[derive(Debug)]
enum Expansion {
    Transform {
        translate_ident: syn::Ident,
        rotate_ident: syn::Ident,
        scale_ident: syn::Ident,
        target_ident: syn::Ident,
    },
}

#[proc_macro_attribute]
pub fn param_list(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ast: syn::ItemStruct = match syn::parse_macro_input!(item as syn::Item) {
        syn::Item::Struct(s) => s,
        _ => {
            return syn::Error::new(
                Span::call_site().into(),
                "#[param_list] can only be used on structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut property_list = Vec::new();
    let mut pragma_list = Vec::new();
    let mut dependency_list = Vec::new();
    let mut expansion_list = Vec::new();

    match &mut ast.fields {
        syn::Fields::Named(fields) => {
            let fields = &mut fields.named;

            for field_index in 0..fields.len() {
                let field = &mut fields[field_index];

                // Skip the field if it's not public
                match field.vis {
                    syn::Visibility::Public(_) => {}
                    _ => continue,
                };

                let field_ident = field.ident.clone().unwrap();
                let field_name = field_ident.to_string();
                let type_val = field.ty.clone();
                let type_str = type_val.clone().into_token_stream().to_string();

                let pragma = field
                    .attrs
                    .iter()
                    .filter_map(|attr| match attr.parse_meta() {
                        Ok(syn::Meta::List(meta_list)) => Some(Ok(meta_list)),
                        Ok(_) => None,
                        Err(err) => return Some(Err(err)),
                    })
                    .filter(|meta| match meta {
                        Ok(meta) => meta.ident.to_string() == "pragma",
                        Err(_) => true,
                    })
                    .map(|meta| {
                        meta.map(|meta| meta.nested[0].clone().into_token_stream().to_string())
                    })
                    .next();

                let pragma = match pragma {
                    Some(Ok(pragma)) => Some(pragma),
                    Some(Err(err)) => return err.to_compile_error().into(),
                    None => None,
                };

                // Clear the attributes now, so the compiler doesn't throw an error
                field.attrs.clear();

                // Matrix4 is special: we 'expand' it into translate/scale/rotate components
                if type_str == "Matrix4" {
                    // add pragmas if 'transform' pragma is on the main property
                    if let Some("transform") = pragma.as_ref().map(|r| r as &str) {
                        pragma_list.push(Pragma {
                            name: "translation".to_string(),
                            prop_index: property_list.len(),
                        });
                        pragma_list.push(Pragma {
                            name: "rotation".to_string(),
                            prop_index: property_list.len() + 1,
                        });
                        pragma_list.push(Pragma {
                            name: "scale".to_string(),
                            prop_index: property_list.len() + 2,
                        });
                    }

                    let translate_ident = syn::Ident::new(
                        &format!("_{}_translate", field_name),
                        Span::call_site().into(),
                    );
                    let rotate_ident = syn::Ident::new(
                        &format!("_{}_rotate", field_name),
                        Span::call_site().into(),
                    );
                    let scale_ident = syn::Ident::new(
                        &format!("_{}_scale", field_name),
                        Span::call_site().into(),
                    );
                    let extra_fields_struct: syn::ItemStruct = match syn::parse2(quote! {
                        struct ExtraFields {
                            pub #translate_ident: crate::math::Vector3,
                            pub #rotate_ident: crate::math::Quaternion,
                            pub #scale_ident: crate::math::Vector3
                        }
                    }) {
                        Ok(s) => s,
                        Err(err) => return err.to_compile_error().into(),
                    };
                    for field in extra_fields_struct.fields.iter() {
                        fields.push(field.clone());
                    }

                    property_list.push(Property {
                        name: format!("{}.translate", field_name),
                        ident: translate_ident.clone(),
                        ty: PropertyType::Vec3,
                        ty_span: Span::call_site(),
                    });
                    property_list.push(Property {
                        name: format!("{}.rotate", field_name),
                        ident: rotate_ident.clone(),
                        ty: PropertyType::Rotation,
                        ty_span: Span::call_site(),
                    });
                    property_list.push(Property {
                        name: format!("{}.scale", field_name),
                        ident: scale_ident.clone(),
                        ty: PropertyType::Vec3,
                        ty_span: Span::call_site(),
                    });

                    expansion_list.push(Expansion::Transform {
                        translate_ident,
                        rotate_ident,
                        scale_ident,
                        target_ident: field_ident,
                    });
                } else {
                    let field_type = match PropertyType::from_type_str(&type_str) {
                        Some(field_type) => field_type,
                        None => {
                            return syn::Error::new(
                                type_val.span(),
                                format!(
                                    "Property {} has unknown field type: {}",
                                    field_ident.to_string(),
                                    type_str
                                ),
                            )
                            .to_compile_error()
                            .into();
                        }
                    };

                    if field_type == PropertyType::Reference {
                        dependency_list.push(Dependency {
                            prop_index: property_list.len(),
                        });
                    }

                    if let Some(pragma_name) = pragma {
                        pragma_list.push(Pragma {
                            name: pragma_name,
                            prop_index: property_list.len(),
                        });
                    }

                    property_list.push(Property {
                        name: field_ident.to_string(),
                        ident: field_ident,
                        ty: field_type,
                        ty_span: type_val.span().into(),
                    });
                }
            }
        }
        _ => {}
    }
    let struct_ident = ast.ident.clone();

    let mut result_stream = syn::Item::Struct(ast).into_token_stream();

    let mut property_stream = proc_macro2::TokenStream::new();
    for property in &property_list {
        let prop_name_ident = &property.ident;
        let prop_name_str = &property.name;
        let prop_type_ident = property.ty.into_ident(property.ty_span);

        property_stream.extend(quote! {
            crate::mesh_gen::commands::EditorCommandProperty {
                name: #prop_name_str,
                val_type: crate::mesh_gen::commands::EditorCommandPropertyType::#prop_type_ident,
                byte_offset: offset_of!(#struct_ident => #prop_name_ident).get_byte_offset(),
            },
        });
    }

    let mut pragmas_stream = proc_macro2::TokenStream::new();
    for pragma in &pragma_list {
        let pragma_name = &pragma.name;
        let pragma_prop = pragma.prop_index;

        pragmas_stream.extend(quote! {
            crate::mesh_gen::commands::EditorCommandPragma {
                name: #pragma_name,
                property: #pragma_prop,
            },
        });
    }

    let mut dependency_stream = proc_macro2::TokenStream::new();
    for dependency in &dependency_list {
        let dependency_prop = dependency.prop_index;

        dependency_stream.extend(quote! {
            crate::mesh_gen::commands::EditorCommandDependency {
                property: #dependency_prop,
            },
        });
    }

    let mut update_stream = proc_macro2::TokenStream::new();
    for expansion in &expansion_list {
        match expansion {
            Expansion::Transform {
                translate_ident,
                rotate_ident,
                scale_ident,
                target_ident,
            } => {
                update_stream.extend(quote! {
                    self.#target_ident = crate::math::Matrix4::translate(self.#translate_ident) * self.#rotate_ident.as_matrix() * crate::math::Matrix4::scale(self.#scale_ident);
                });
            }
        }
    }

    let property_list_ident = syn::Ident::new(
        &(struct_ident.to_string().to_uppercase() + "_EDITOR_COMMAND_PROP_LIST"),
        struct_ident.span(),
    );
    let pragma_list_ident = syn::Ident::new(
        &(struct_ident.to_string().to_uppercase() + "_EDITOR_COMMAND_PRAGMA_LIST"),
        struct_ident.span(),
    );
    let dependency_list_ident = syn::Ident::new(
        &(struct_ident.to_string().to_uppercase() + "_EDITOR_COMMAND_DEPENDENCY_LIST"),
        struct_ident.span(),
    );
    let command_schema_ident = syn::Ident::new(
        &(struct_ident.to_string().to_uppercase() + "_EDITOR_COMMAND_SCHEMA"),
        struct_ident.span(),
    );

    let prop_stream_count = property_list.len();
    let pragma_stream_count = pragma_list.len();
    let dependency_stream_count = dependency_list.len();

    let prop_streams = quote! {
        static ref #property_list_ident: [crate::mesh_gen::commands::EditorCommandProperty; #prop_stream_count] = [#property_stream];
        static ref #pragma_list_ident: [crate::mesh_gen::commands::EditorCommandPragma; #pragma_stream_count] = [#pragmas_stream];
        static ref #dependency_list_ident: [crate::mesh_gen::commands::EditorCommandDependency; #dependency_stream_count] = [#dependency_stream];
    };

    result_stream.extend(quote! {
        lazy_static::lazy_static! {
            #prop_streams
            static ref #command_schema_ident: crate::mesh_gen::commands::EditorCommandSchema = crate::mesh_gen::commands::EditorCommandSchema {
                properties: &*#property_list_ident,
                pragmas: &*#pragma_list_ident,
                dependencies: &*#dependency_list_ident,
            };
        }
    });

    let sub_stream_a = quote! {
        fn update(&mut self) {
            #update_stream
        }

        fn run(&self, mesh: &mut crate::mesh_gen::MeshHandle, selection: &mut crate::mesh_gen::Selection, executor: &crate::mesh_gen::Executor) {
            Command::run(self, mesh, selection, executor);
        }
    };
    let sub_stream_b = quote! {
        fn clone(&self) -> alloc::boxed::Box<dyn crate::mesh_gen::commands::EditorCommand> {
            alloc::boxed::Box::new(Clone::clone(self))
        }

        unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
            core::slice::from_raw_parts_mut(self as *mut #struct_ident as *mut u8, core::mem::size_of::<#struct_ident>())
        }
    };

    result_stream.extend(quote! {
        impl #struct_ident {
            pub fn schema() -> &'static crate::mesh_gen::commands::EditorCommandSchema {
                &*#command_schema_ident
            }
        }

        impl crate::mesh_gen::commands::EditorCommand for #struct_ident {
            #sub_stream_a
            #sub_stream_b
        }
    });

    result_stream.into()
}
