#include "fluid_data.hlsl"

Texture3D<int> boundary_map : register(t0);
Texture3D<float4> velocity_map_in : register(t1);
Texture3D<float2> density_temperature_map_in : register(t2);

RWTexture3D<float4> velocity_map_out : register(u0);
RWTexture3D<float2> density_temperature_map_out : register(u1);

#define GEN_TAKE_SAMPLE(Ty)                             \
Ty take_sample(Texture3D<Ty> tex, float3 pos) {         \
    int2 off = int2(0, 1);                              \
    float3 fp = frac(pos);                              \
    int3 paaa = floor(pos);                             \
    int3 pbaa = min(pos + off.yxx, map_size-1);         \
    int3 paba = min(pos + off.xyx, map_size-1);         \
    int3 pbba = min(pos + off.yyx, map_size-1);         \
    int3 paab = min(pos + off.xxy, map_size-1);         \
    int3 pbab = min(pos + off.yxy, map_size-1);         \
    int3 pabb = min(pos + off.xyy, map_size-1);         \
    int3 pbbb = min(pos + off.yyy, map_size-1);         \
                                                        \
    Ty x0 = tex[paaa] * (1 - fp.x) + tex[pbaa] * fp.x;  \
    Ty x1 = tex[paab] * (1 - fp.x) + tex[pbab] * fp.x;  \
                                                        \
    Ty x2 = tex[paba] * (1 - fp.x) + tex[pbba] * fp.x;  \
    Ty x3 = tex[pabb] * (1 - fp.x) + tex[pbbb] * fp.x;  \
                                                        \
    Ty z0 = x0 * (1 - fp.z) + x1 * fp.z;                \
    Ty z1 = x2 * (1 - fp.z) + x3 * fp.z;                \
                                                        \
    return z0 * (1 - fp.y) + z1 * fp.y;                 \
}

GEN_TAKE_SAMPLE(float2)
GEN_TAKE_SAMPLE(float4)

[numthreads(32, 32, 1)]
void main(uint3 pos : SV_DispatchThreadID) {
    if (boundary_map[pos]) {
        density_temperature_map_out[pos] = 0;
        velocity_map_out[pos] = 0;
        return;
    }

    float3 advected_pos = pos - delta_time * velocity_map_in[pos].xyz;

    density_temperature_map_out[pos] = max(
        0,
        take_sample(density_temperature_map_in, advected_pos) * float2(density_dissipation, temperature_dissipation)
    );
    velocity_map_out[pos] = take_sample(velocity_map_in, advected_pos) * velocity_dissipation;
}
