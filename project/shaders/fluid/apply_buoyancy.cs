#include "fluid_data.hlsl"

Texture3D<float2> density_temperature_map : register(t0);
RWTexture3D<float4> velocity_map : register(u0);

static const float AMBIENT_TEMPERATURE = 0;

[numthreads(32, 32, 1)]
void main(uint3 pos : SV_DispatchThreadID) {
    float2 density_temperature = density_temperature_map[pos];

    if (density_temperature.y > AMBIENT_TEMPERATURE) {
        float4 velocity = velocity_map[pos];
        velocity.y += delta_time * (density_temperature.y - AMBIENT_TEMPERATURE) * density_buoyancy - density_temperature.x * density_weight;
        velocity_map[pos] = velocity;
    }
}
