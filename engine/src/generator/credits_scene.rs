use super::prelude::*;
use crate::blend_state::{BlendRenderTargetConfig, BlendState};
use crate::buffer::{Buffer, InitialData};
use crate::math::RgbaColor;
use crate::renderer::common::PostRenderer;
use crate::texture::{from_wmf, ShaderResource2D, Texture2D};
use crate::viewport::Viewport;
use core::ptr;
use winapi::um::d3d11::{
    D3D11_BIND_CONSTANT_BUFFER, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD,
    D3D11_VIEWPORT,
};

pub static CREDITS_SCENE_SCHEMA: GeneratorSchema = GeneratorSchema {
    #[cfg(debug_assertions)]
    name: "Credits Scene",
    instantiate_generator: |context| Box::new(CreditsScene::new(context)),
    groups: &[SchemaGroup {
        #[cfg(debug_assertions)]
        name: "",
        properties: &[SchemaProperty {
            #[cfg(debug_assertions)]
            name: "opacity",
            value_type: PropertyType::Float,
        }],
    }],
};

#[derive(Clone, Copy)]
#[repr(C)]
struct CreditData {
    fade: f32,
}

pub struct CreditsScene {
    tex: Texture2D,
    blend: BlendState,
    renderer: PostRenderer,
    credit_data: Buffer<CreditData>,
}

impl CreditsScene {
    pub fn new(context: &mut CreationContext) -> Self {
        let emf_bytes = include_bytes!("../../resources/credits.emf");
        CreditsScene {
            tex: from_wmf(
                context.device,
                context.devcon,
                emf_bytes,
                Viewport {
                    width: context.viewport.width * 2,
                    height: context.viewport.height * 2,
                },
                RgbaColor::new(0., 0., 0., 0.),
            ),
            blend: BlendState::new_dependent(
                context.device,
                false,
                BlendRenderTargetConfig::enabled(
                    D3D11_BLEND_ONE,
                    D3D11_BLEND_INV_SRC_ALPHA,
                    D3D11_BLEND_OP_ADD,
                    D3D11_BLEND_ONE,
                    D3D11_BLEND_INV_SRC_ALPHA,
                    D3D11_BLEND_OP_ADD,
                ),
            ),
            renderer: PostRenderer::new(context, "credits.ps"),
            credit_data: Buffer::new_dynamic(
                context.device,
                InitialData::Uninitialized(1),
                D3D11_BIND_CONSTANT_BUFFER,
            ),
        }
    }
}

impl Generator for CreditsScene {
    fn update(
        &mut self,
        io: &mut GBuffer,
        context: &mut FrameContext,
        renderers: &mut RendererCollection,
        local_frame: u32,
        properties: &[&[ClipPropertyValue]],
    ) {
        self.credit_data.upload(
            context.devcon,
            CreditData {
                fade: prop(properties, 0, 0),
            },
        );

        unsafe {
            (*context.devcon).OMSetBlendState(self.blend.ptr(), &[1., 1., 1., 1.], 0xFFFFFF);
            (*context.devcon).PSSetConstantBuffers(2, 1, &self.credit_data.ptr());
            (*context.devcon).PSSetShaderResources(0, 1, &self.tex.shader_resource_ptr());
        }

        self.renderer.render(context, io.write_output(), true, true);

        unsafe {
            (*context.devcon).OMSetBlendState(ptr::null_mut(), &[1., 1., 1., 1.], 0xFFFFFF);
            (*context.devcon).PSSetConstantBuffers(2, 1, &ptr::null_mut());
            (*context.devcon).PSSetShaderResources(0, 1, &ptr::null_mut());
        }
    }
}
