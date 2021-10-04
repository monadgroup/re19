#include "common/camera.hlsl"
#include "common/ia_out.hlsl"
#include "common/vs_out.hlsl"
#include "common/noise.hlsl"

cbuffer Object : register(b2) {
    matrix modelMatrix;
    matrix normModelMatrix;
}

float heightmap(float2 pos) {
    pos += 600;
    float n = (fbm_procedural(float3(pos/200, 100)) - 0.5) * min(2, (pos.y + 200)/120);
    n += fbm_procedural(float3(pos/2, 200)) * 0.01;
    //n *= n;

    return n*100;
}

VSOut main(IAOut input) {
    VSOut output;

    output.world_pos = mul(float4(input.position, 1), modelMatrix).xyz;
    output.world_pos.y += heightmap(output.world_pos.xz);
    output.position = mul(float4(output.world_pos, 1), viewProjMatrix);
    output.normal = 0;
    output.uv = input.uv;

    return output;
}
