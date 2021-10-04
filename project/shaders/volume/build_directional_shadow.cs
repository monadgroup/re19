#include "volume_data.hlsl"

SamplerState smp : register(s0);
Texture3D density : register(t0);

RWTexture3D<float> shadow_map : register(u0);

[numthreads(32, 32, 1)]
void main(uint3 thread_id : SV_DispatchThreadID) {
    float3 last_world_pos = mul(float4(thread_id.xy, -1, 1), scaled_directional_shadow_to_world).xyz;
    float shadow_light = 1;
    for (uint i = 0; i < directional_shadow_map_size.z; i++) {
        uint3 shadow_pos = uint3(thread_id.xy, i);
        shadow_map[shadow_pos] = shadow_light;

        float3 world_pos = mul(float4(shadow_pos, 1), scaled_directional_shadow_to_world).xyz;
        float3 density_pos = mul(float4(world_pos, 1), world_to_density).xyz;
        float sigma_s = density.SampleLevel(smp, density_pos, 0).x;
        float sigma_e = max(0.000000001, sigma_s);
        shadow_light *= exp(-sigma_e * distance(world_pos, last_world_pos));
        last_world_pos = world_pos;
    }
}
