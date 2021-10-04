#include "fluid_data.hlsl"

Texture3D<int> boundary_map : register(t0);
Texture3D<float> pressure_map : register(t1);
RWTexture3D<float4> velocity_map : register(u0);

float sample_pressure(int3 p, float c, inout float mask) {
    if (any(p < 0) || any(p >= map_size) || boundary_map[p]) {
        mask = 0;
        return c;
    } else {
        return pressure_map[p];
    }
}

[numthreads(32, 32, 1)]
void main(int3 pos : SV_DispatchThreadID) {
    int3 off = int3(-1, 0, 1);
    float3 mask = 1;
    float c = pressure_map[pos];
    float l = sample_pressure(pos + off.xyy, c, mask.x);
    float r = sample_pressure(pos + off.zyy, c, mask.x);
    float b = sample_pressure(pos + off.yxy, c, mask.y);
    float t = sample_pressure(pos + off.yzy, c, mask.y);
    float d = sample_pressure(pos + off.yyx, c, mask.z);
    float u = sample_pressure(pos + off.yyz, c, mask.z);

    float3 v = velocity_map[pos].xyz - float3(r - l, t - b, u - d) * 0.5;
    velocity_map[pos] = float4(v * mask, 0);
}
