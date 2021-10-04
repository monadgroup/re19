#include "volume_data.hlsl"

SamplerState smp : register(s0);
Texture3D light_map : register(t0);

RWTexture3D<float2> out_map : register(u0);

[numthreads(32, 32, 1)]
void main(uint3 thread_id : SV_DispatchThreadID) {
    float3 tex_coord = float3(thread_id) / light_map_size;

    float2 acc = 0;
    float weight = 0;
    uint level_count = 5;
    for (uint level = 0; level < level_count; level++) {
        float level_weight = 1. / (1 << level);
        acc += light_map.SampleLevel(smp, tex_coord, level).xy * level_weight;
        weight += level_weight;
    }

    out_map[thread_id] = acc / weight;
}
