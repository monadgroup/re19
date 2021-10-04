#include "project.hlsl"

struct BoxIntersection {
    float nearest_dist;
    float furthest_dist;
};

BoxIntersection find_intersection(Ray ray, float3 box_min, float3 box_max) {
    float3 t_min = (box_min - ray.origin) / ray.direction;
    float3 t_max = (box_max - ray.origin) / ray.direction;
    float3 real_min = min(t_min, t_max);
    float3 real_max = max(t_min, t_max);

    BoxIntersection intersection;
    intersection.nearest_dist = max(max(real_min.x, real_min.y), real_min.z);
    intersection.furthest_dist = min(min(real_max.x, real_max.y), real_max.z);
    return intersection;
}

float plane_intersection(Ray ray, float3 center, float3 normal) {
    float denom = dot(normal, ray.direction);
    if (abs(denom) > 0.0001) {
        return dot(center - ray.origin, normal) / denom;
    } else {
        return 0;
    }
}
