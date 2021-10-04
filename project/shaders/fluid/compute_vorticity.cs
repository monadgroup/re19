#include "fluid_data.hlsl"

Texture3D<float4> velocity_map : register(t0);
RWTexture3D<float4> vorticity_map : register(u0);

[numthreads(32, 32, 1)]
void main(int3 pos : SV_DispatchThreadID) {
    int3 off = int3(-1, 0, 1);
    float3 l = velocity_map[clamp(pos + off.xyy, 0, map_size-1)].xyz;
    float3 r = velocity_map[clamp(pos + off.zyy, 0, map_size-1)].xyz;
    float3 b = velocity_map[clamp(pos + off.yxy, 0, map_size-1)].xyz;
    float3 t = velocity_map[clamp(pos + off.yzy, 0, map_size-1)].xyz;
    float3 d = velocity_map[clamp(pos + off.yyx, 0, map_size-1)].xyz;
    float3 u = velocity_map[clamp(pos + off.yyz, 0, map_size-1)].xyz;

    vorticity_map[pos] = 0.5 * float4((t.z - b.z) - (u.y - d.y), (u.x - d.x) - (r.z - l.z), (r.y - l.y) - (t.x - b.x), 0);
}
