#include "camera.hlsl"

struct Ray {
    float3 origin;
    float3 direction;
};

float3 project_clip_to_view(float3 clip_pos) {
    float4 view_pos_unorm = mul(float4(clip_pos, 1), invProjMatrix);
    return view_pos_unorm.xyz / view_pos_unorm.w;
}

float3 project_clip_to_world(float3 clip_pos) {
    return mul(float4(project_clip_to_view(clip_pos), 1), invViewMatrix).xyz;
}

Ray project_ray(float2 tex_coord) {
    float3 p0 = project_clip_to_world(float3(tex_coord, 0));
    float3 p1 = project_clip_to_world(float3(tex_coord, 1));

    Ray ray;
    ray.origin = p0;
    ray.direction = normalize(p1 - p0);
    return ray;
}

float2 clip_to_tex_coord(float2 clip_coord) {
    float2 tex_coord = clip_coord * 0.5 + 0.5;
    tex_coord.y = 1 - tex_coord.y;
    return tex_coord;
}

float2 tex_to_clip_coord(float2 tex_coord) {
    tex_coord.y = 1 - tex_coord.y;
    return tex_coord * 2 - 1;
}
