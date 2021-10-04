#include "project.hlsl"
#include "camera.hlsl"
#include "frame_data.hlsl"
#include "math.hlsl"

// To be implemented by the scene:
//static const int MAX_ITERATIONS;
//SceneOutput fScene(float3 p);
//float3 lightScene(float3 p, float3 n, float3 d, float ao);
//float3 lightSky(float3 d);

//#define USE_ENHANCED_TRACING

#ifdef USE_ENHANCED_TRACING
bool raymarch_loop(inout float3 p, inout float t, inout float ao, float3 d, float pixel_radius, float clip_far) {
    bool force_hit = false;
    float omega = 1.2;
    float candidate_error = INFINITY;
    float candidate_t = t;
    float candidate_ao = 0;
    float3 o = p;
    float3 candidate_p = o;
    float previous_radius = 0;
    float step_length = 0;
    float function_sign = sign(fScene(o).dist);

    int i = 0;
    for (i = 0; i < MAX_ITERATIONS; ++i) {
        p = d*t + o;
        SceneOutput scene_out = fScene(p);
        float signed_radius = function_sign * scene_out.dist;
        float radius = abs(signed_radius);

        bool sor_fail = omega > 1 && (radius + previous_radius) < step_length;
        if (sor_fail) {
            step_length -= omega * step_length;
            omega = 1;
        } else {
            step_length = signed_radius * omega;
        }

        previous_radius = radius;
        float error = radius / t;

        if (!sor_fail && error < candidate_error) {
            candidate_t = t;
            candidate_p = p;
            candidate_ao = scene_out.ao;
            candidate_error = error;
        }
        if (!sor_fail && error < pixel_radius || t > clip_far) {
            break;
        }
        t += step_length;
    }

    if ((t > clip_far || candidate_error > pixel_radius) && !force_hit) {
        return true;
    }

    p = candidate_p;
    t = candidate_t;
    ao = candidate_ao;
    return false;
}
#else
static const float MIN_RADIUS = 0.00005;
bool raymarch_loop(inout float3 p, inout float t, inout float ao, float3 d, float pixel_radius, float clip_far) {
    float3 o = p;
    int i = 0;
    for (i = 0; i < MAX_ITERATIONS && t < clip_far; ++i) {
        p = d * t + o;
        SceneOutput scene_out = fScene(p);
        if (scene_out.dist < MIN_RADIUS) {
            ao = scene_out.ao;
            return false;
        }
        t += scene_out.dist;
    }
    return true;
}
#endif

float3 raymarch_normal(float3 p) {
    float2 e = float2(0.00005, -0.00005);
    float4 o = float4(
        fScene(p + e.xyy).dist,
        fScene(p + e.yyx).dist,
        fScene(p + e.yxy).dist,
        fScene(p + e.xxx).dist
    );
    return normalize(o.wzy + o.xww - o.zxz - o.yyx);
}

void raymarch(float2 tex_coord, float max_depth, out float4 color, out float4 normal, out float4 world_pos, out float depth) {
    Ray ray = project_ray(tex_to_clip_coord(tex_coord));

    float3 d = ray.direction;
    float3 p = ray.origin;
    float pixel_size = 2 * sin(camFovRadians) / viewportSize.y;
    float t = zRange.x;
    float ao = 0;
    bool is_sky = raymarch_loop(p, t, ao, d, pixel_size, max_depth);

    if (is_sky) {
        color = float4(lightSky(d), 1);
        normal = float4(d, 0);
        world_pos = float4(d * max_depth, max_depth);
        depth = 1;
    } else {
        normal = float4(raymarch_normal(p), 1);
        color = float4(lightScene(p, normal.xyz, d, ao), 1);
        world_pos = float4(p, distance(ray.origin, p));

        float4 clip_space_pos = mul(float4(p, 1), viewProjMatrix);
        depth = clip_space_pos.z / clip_space_pos.w;
    }
}
