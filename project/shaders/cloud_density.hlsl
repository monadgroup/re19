#include "common/noise.hlsl"

cbuffer CloudsData : register(b2) {
    float3 map_offset;
    float cloud_y;
    float3 sky_color;
    float cloud_height;
    float3 scatter_color;
    float cloud_opacity;
    float3 light_direction;
};

void pR(inout float2 p, float a) {
    p = cos(a) * p + sin(a) * float2(p.y, -p.x);
}

float cloud_density(float3 p) {
    p.xz += map_offset.xz;
    p.y = (p.y - cloud_y) / cloud_height;

    float2 main_p = p.xz / 6000;

    pR(main_p, 2);

    float dens = fbm_procedural(float3(main_p, p.y*2));
    float cov = 0.5;
    dens *= smoothstep(cov, cov + 0.05, dens);

    float big_dens = fbm_procedural(float3(main_p/4, p.y/4));

    float m = noise_procedural(float3(p.xz/100, 0));
    return saturate(dens) * smoothstep(0.6, 0.61, big_dens) * 0.0005  * smoothstep(0, m*0.2, p.y) * smoothstep(0, m*0.2, 1-p.y);
}
