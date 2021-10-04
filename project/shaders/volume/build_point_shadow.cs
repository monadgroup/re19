#include "volume_data.hlsl"
#include "spherical_map.hlsl"

SamplerState smp : register(s0);
Texture3D density : register(t0);

RWTexture3D<float> shadow_map : register(u0);

[numthreads(32, 32, 1)]
void main(uint3 thread_id : SV_DispatchThreadID) {
    float3 last_world_pos = spherical_to_world_pos(
        float3(thread_id.xy, -1),
        point_light_world_pos,
        point_shadow_map_size,
        point_light_max_radius
    );
    float shadow_light = 1;
    for (uint i = 0; i < point_shadow_map_size.z; i++) {
        uint3 spherical_pos = uint3(thread_id.xy, i);
        float3 world_pos = spherical_to_world_pos(
            spherical_pos,
            point_light_world_pos,
            point_shadow_map_size,
            point_light_max_radius
        );
        float norm_dist = distance(world_pos, point_light_world_pos) * point_light_radius;
        float att = 1. / (1 + 0.1 * norm_dist + 0.01 * norm_dist * norm_dist);
        shadow_map[spherical_pos] = shadow_light * att;

        float3 density_pos = mul(float4(world_pos, 1), world_to_density).xyz;
        float sigma_s = density.SampleLevel(smp, density_pos, 0).x;
        float sigma_e = max(0.000000001, sigma_s);
        shadow_light *= exp(-sigma_e * distance(world_pos, last_world_pos));
        last_world_pos = world_pos;
    }
}
