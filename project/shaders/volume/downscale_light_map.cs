#include "volume_data.hlsl"

cbuffer DownscalePassData : register(b0) {
    uint size_divisor;
}

SamplerState smp : register(s0);
Texture3D source_light_map : register(t0);
RWTexture3D<float2> target_light_map : register(u0);

static const int BLUR_RANGE = 2;
static const float COEFFICIENTS[5] = {0.06136, 0.24477, 0.38774, 0.24477, 0.06136};

[numthreads(4, 4, 1)]
void main(uint3 thread_id : SV_DispatchThreadID) {
    float3 target_tex_pos = thread_id;
    float3 light_map_pos = target_tex_pos * size_divisor;
    float3 world_pos = mul(float4(light_map_pos, 1), scaled_light_map_to_world).xyz;

    float2 acc = 0;
    for (int z = -BLUR_RANGE; z <= BLUR_RANGE; z++) {
        for (int y = -BLUR_RANGE; y <= BLUR_RANGE; y++) {
            for (int x = -BLUR_RANGE; x <= BLUR_RANGE; x++) {
                float3 sample_world_pos = world_pos + float3(x, y, z) * light_blur_size;
                float3 sample_light_map_pos = mul(float4(sample_world_pos, 1), world_to_light_map).xyz;

                float weight = COEFFICIENTS[z + BLUR_RANGE] * COEFFICIENTS[y + BLUR_RANGE] * COEFFICIENTS[x + BLUR_RANGE];
                acc += source_light_map.SampleLevel(smp, sample_light_map_pos, 0).xy * weight;
            }
        }
    }
    target_light_map[thread_id] = acc;
}
