#include "math.hlsl"

float3 fresnel_schlick(float cos_theta, float3 f0) {
    return f0 + (1 - f0) * pow(1 - cos_theta, 5);
}

float distribution_ggx(float3 n, float3 h, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float n_dot_h = max(dot(n, h), 0);
    float n_dot_h2 = n_dot_h * n_dot_h;

    float nom = a2;
    float denom = (n_dot_h2 * (a2 - 1) + 1);
    denom = M_PI * denom * denom;

    return nom / denom;
}

float geometry_schlick_ggx(float n_dot_v, float roughness) {
    float r = roughness + 1;
    float k = (r * r) / 8;

    float nom = n_dot_v;
    float denom = n_dot_v * (1 - k) + k;

    return nom / denom;
}

float geometry_smith(float3 n, float3 v, float3 l, float roughness) {
    float n_dot_v = max(dot(n, v), 0);
    float n_dot_l = max(dot(n, l), 0);
    float ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    float ggx1 = geometry_schlick_ggx(n_dot_l, roughness);

    return ggx1 * ggx2;
}

float3 get_light_radiance(float3 n, float3 albedo, float metallic, float roughness, float3 v, float3 l) {
    float3 h = normalize(v + l);

    float3 f0 = lerp(0.04, albedo, metallic);

    // cook-torrance brdf
    float ndf = distribution_ggx(n, h, roughness);
    float g = geometry_smith(n, v, l, roughness);
    float3 f = fresnel_schlick(max(dot(h, v), 0), f0);

    float3 k_s = f;
    float3 k_d = 1 - k_s;
    k_d *= 1 - metallic;

    float3 nominator = ndf * g * f;
    float denominator = 4 * max(dot(n, v), 0) * max(dot(n, l), 0) + 0.001;
    float3 specular = nominator / denominator;

    // add to outgoing radiance Lo
    float n_dot_l = max(dot(n, l), 0);
    return (k_d * albedo / M_PI + specular) * n_dot_l;
}
