#include "common/camera.hlsl"
#include "common/ia_out.hlsl"
#include "common/vs_out.hlsl"

cbuffer Object : register(b2) {
    matrix modelMatrix;
    matrix normModelMatrix;
}

VSOut main(IAOut input) {
    VSOut output;

    output.world_pos = mul(float4(input.position, 1), modelMatrix).xyz;
    output.position = mul(float4(output.world_pos, 1), viewProjMatrix);

    output.normal = normalize(mul(float4(input.normal, 0), normModelMatrix).xyz);
    output.uv = input.uv;

    return output;
}
