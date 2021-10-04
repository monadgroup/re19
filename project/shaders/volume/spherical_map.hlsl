#include "../common/math.hlsl"

float3 spherical_to_world_pos(float3 spherical_pos, float3 light_world_pos, uint3 shadow_map_size, float max_radius) {
    float denorm_azimuth = spherical_pos.x / shadow_map_size.x * M_PI * 2;
    float denorm_polar = spherical_pos.y / shadow_map_size.y * M_PI;
    float radius = spherical_pos.z / shadow_map_size.z;
    float3 cartesian_pos = radius * float3(
        sin(denorm_polar) * cos(denorm_azimuth),
        sin(denorm_polar) * sin(denorm_azimuth),
        cos(denorm_polar)
    );
    return light_world_pos + max_radius * cartesian_pos;
}

float3 world_to_spherical_pos(float3 world_pos, float3 light_world_pos, uint3 shadow_map_size, float max_radius) {
    float3 cartesian_pos = (world_pos - light_world_pos) / max_radius;
    float radius = length(cartesian_pos);
    if (radius == 0) return float3(0, 0, 0);

    float denorm_polar = acos(cartesian_pos.z / radius);
    float denorm_azimuth = atan2(cartesian_pos.y, cartesian_pos.x);
    if (denorm_azimuth < 0) denorm_azimuth = 2 * M_PI + denorm_azimuth;

    return float3(
        denorm_azimuth / (M_PI * 2) * shadow_map_size.x,
        denorm_polar / M_PI * shadow_map_size.y,
        (radius) * shadow_map_size.z
    );
}
