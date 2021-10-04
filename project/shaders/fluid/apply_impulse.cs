#include "fluid_data.hlsl"

RWTexture3D<float2> density_temperature_map : register(u0);
RWTexture3D<float4> velocity_map : register(u1);

[numthreads(32, 32, 1)]
void main(uint3 pos : SV_DispatchThreadID) {
    float3 rel_pos = float3(pos) / map_size - input_pos;
    rel_pos /= input_radius;

    //float mag = rel_pos.x*rel_pos.x + rel_pos.y*rel_pos.y + rel_pos.z*rel_pos.z;
    //float m = exp(-mag) * delta_time;
    float m = all(abs(rel_pos) <= 1);
    density_temperature_map[pos] += float2(density_amount, temperature_amount) * m;
    velocity_map[pos] += float4(velocity_amount, 0) * m;
}
