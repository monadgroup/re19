#include "common/vs_post_out.hlsl"
#include "common/project.hlsl"
#include "common/frame_data.hlsl"

struct IAPostOut {
    float2 position : POSITION0;
};

VSPostOut main(IAPostOut input) {
    VSPostOut output;

    output.position =  float4(input.position, 0, 1);
    output.tex = clip_to_tex_coord(input.position);

    return output;
}
