#include "fluid_data.hlsl"

Texture3D<int> boundary_map : register(t0);
Texture3D<float4> velocity_map : register(t1);
RWTexture3D<float> divergence_map : register(u0);

float3 sample_vel(int3 p) {
    if (any(p < 0) || any(p >= map_size) || boundary_map[p]) {
        return 0;
    } else {
        return velocity_map[p].xyz;
    }
}

[numthreads(32, 32, 1)]
void main(int3 pos : SV_DispatchThreadID) {
    int3 off = int3(-1, 0, 1);
    float3 l = sample_vel(pos + off.xyy);
    float3 r = sample_vel(pos + off.zyy);
    float3 b = sample_vel(pos + off.yxy);
    float3 t = sample_vel(pos + off.yzy);
    float3 d = sample_vel(pos + off.yyx);
    float3 u = sample_vel(pos + off.yyz);

    divergence_map[pos] = 0.5 * ((r.x - l.x) + (t.y - b.y) + (u.z - d.z));
}
