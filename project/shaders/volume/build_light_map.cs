#include "volume_data.hlsl"
#include "spherical_map.hlsl"

SamplerState directional_smp : register(s0);
SamplerState point_smp : register(s1);

Texture3D directional_shadow_map : register(t0);
Texture3D point_shadow_map : register(t1);

RWTexture3D<float2> light_map : register(u0);

[numthreads(32, 32, 1)]
void main(uint3 thread_id : SV_DispatchThreadID) {
    float3 light_map_pos = thread_id;
    float3 world_pos = mul(float4(light_map_pos, 1), scaled_light_map_to_world).xyz;

    // Directional light
    float3 directional_shadow_pos = mul(float4(world_pos, 1), world_to_directional_shadow).xyz;
    float directional_contribution = directional_shadow_map.SampleLevel(directional_smp, directional_shadow_pos, 0).x;

    // Point light
    float3 spherical_pos = world_to_spherical_pos(world_pos, point_light_world_pos, point_shadow_map_size, point_light_max_radius);
    // todo: we probably want to sample this in shadow space instead of spherical space
    float point_contribution = point_shadow_map.SampleLevel(point_smp, spherical_pos / point_shadow_map_size, 0).x;

    light_map[thread_id] = float2(directional_contribution, point_contribution);
}
