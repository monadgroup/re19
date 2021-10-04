#include "fluid_data.hlsl"

Texture3D<int> boundary_map : register(t0);
Texture3D<float> divergence_map : register(t1);
Texture3D<float> pressure_map_in : register(t2);
RWTexture3D<float> pressure_map_out : register(u0);

float sample_pressure(int3 p, float c) {
    if (any(p < 0) || any(p >= map_size) || boundary_map[p]) {
        return c;
    } else {
        return pressure_map_in[p];
    }
}

[numthreads(32, 32, 1)]
void main(int3 pos : SV_DispatchThreadID) {
    float divergence = divergence_map[pos];

    int3 off = int3(-1, 0, 1);
    float c = pressure_map_in[pos];
    float l = sample_pressure(pos + off.xyy, c);
    float r = sample_pressure(pos + off.zyy, c);
    float b = sample_pressure(pos + off.yxy, c);
    float t = sample_pressure(pos + off.yzy, c);
    float d = sample_pressure(pos + off.yyx, c);
    float u = sample_pressure(pos + off.yyz, c);
    pressure_map_out[pos] = (l + r + b + t + u + d - divergence) / 6.;
}
