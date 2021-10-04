#include "volume_data.hlsl"
#include "../common/noise.hlsl"

SamplerState smp : register(s0);
Texture3D noise_map : register(t0);

RWTexture3D<float> density : register(u0);

float density_map(float3 p) {
    float3 original_p = p;
    float3 perturb_p = p;
    perturb_p *= 3;
    perturb_p += float3(400, 10, 13);
    float3 perturb = float3(
        fbm(perturb_p.xyz, smp, noise_map),
        fbm(perturb_p.yzx, smp, noise_map),
        fbm(perturb_p.zxy, smp, noise_map)
    );
    p += perturb * 2 - 1;

    float amount = 0;

    float3 back_wall_p = p;
    back_wall_p.y += 0.5;
    back_wall_p.z -= 0.25;
    back_wall_p.z *= 0.8;
    float front_layer = step(back_wall_p.x, -0) * step(back_wall_p.y-(back_wall_p.x+2)/8, back_wall_p.z*back_wall_p.z);
    amount += front_layer;

    float3 left_wall_p = p;
    left_wall_p += (perturb * 2 - 1);
    left_wall_p.x -= 1.3;
    left_wall_p.z -= 1.1;
    float left_wall = (1-step(pow(10, left_wall_p.z), saturate(left_wall_p.y-1)-left_wall_p.x)) * step(left_wall_p.y/2, left_wall_p.x+2);
    amount += left_wall;

    float depth_amount = max(0, -original_p.y)/5;
    amount = lerp(depth_amount, 1, step(1, amount))/2;

    float top_plate = step(2.1, original_p.y)*step(original_p.x, 0);
    amount += top_plate;

    return (amount + 0.01) * 10;
}

[numthreads(32, 32, 1)]
void main(uint3 thread_id : SV_DispatchThreadID) {
    float3 p = mul(float4(thread_id, 1), scaled_density_to_world).xyz;
    float3 d = float3(-1, 0, 1) * 0.01;
    float blur_weight = 0.1;

    float weight = 1;
    float amount = density_map(p);

    amount += density_map(p + d.xyy) * blur_weight;
    weight += blur_weight;

    amount += density_map(p + d.yxy) * blur_weight;
    weight += blur_weight;

    amount += density_map(p + d.yyx) * blur_weight;
    weight += blur_weight;

    amount += density_map(p + d.zyy) * blur_weight;
    weight += blur_weight;

    amount += density_map(p + d.yzy) * blur_weight;
    weight += blur_weight;

    amount += density_map(p + d.yyz) * blur_weight;
    weight += blur_weight;

    density[thread_id] = amount / weight;
}
