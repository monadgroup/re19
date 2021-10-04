#include "fluid_data.hlsl"

Texture3D<float4> vorticity_map : register(t0);
RWTexture3D<float4> velocity_map : register(u0);

[numthreads(32, 32, 1)]
void main(int3 pos : SV_DispatchThreadID) {
    int3 off = int3(-1, 0, 1);
    float3 omega = vorticity_map[pos].xyz;
    float omega_l = length(vorticity_map[clamp(pos + off.xyy, 0, map_size-1)].xyz);
    float omega_r = length(vorticity_map[clamp(pos + off.zyy, 0, map_size-1)].xyz);
    float omega_b = length(vorticity_map[clamp(pos + off.yxy, 0, map_size-1)].xyz);
    float omega_t = length(vorticity_map[clamp(pos + off.yzy, 0, map_size-1)].xyz);
    float omega_d = length(vorticity_map[clamp(pos + off.yyx, 0, map_size-1)].xyz);
    float omega_u = length(vorticity_map[clamp(pos + off.yyz, 0, map_size-1)].xyz);

    float3 eta = 0.5 * float3(omega_r - omega_l, omega_t - omega_b, omega_u - omega_d);
    eta = normalize(eta + 0.001);
    float3 force = delta_time * vorticity_strength * (eta.yzx * omega.zxy - eta.zxy * omega.yzx);
    velocity_map[pos] += float4(force, 0);
}
