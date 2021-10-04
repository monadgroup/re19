#include "math.hlsl"
#include "light.hlsl"

static const float EARTH_RS = 6360.0e3;
static const float EARTH_RA = 6420.0e3;
static const float3 EARTH_BETA_R = float3(5.5e-6, 13.0e-6, 22.1e-6);
static const float3 EARTH_BETA_M = 18.0e-6;
static const float EARTH_SH_R = 7994;
static const float EARTH_SH_M = 1200;

static const float SUN_MC = 0.76;
static const float SUN_AD = radians(0.53);

static const uint SKYLIGHT_NB_VIEWDIR_SAMPLES = 12;
static const uint SKYLIGHT_NB_SUNDIR_SAMPLES = 6;

bool intersect_with_atmosphere(float3 ro, float3 rd, out float tr) {
    float c = length(ro);
    float3 up_dir = ro / max(c, 0.0001);
    float beta = M_PI - acos(dot(rd, up_dir));
    float sb = sin(beta);
    float b = EARTH_RA;
    float bt = EARTH_RS - 10;

    tr = sqrt((b * b) - (c * c) * (sb * sb)) + c * cos(beta);

    return sqrt((bt * bt) - (c * c) * (sb * sb)) + c * cos(beta) <= 0;
}

float compute_sun_visibility(float alt) {
    float vap = 0;
    float h, a;
    float vvp = saturate(0.5 + alt / SUN_AD);
    if (vvp == 0) return 0;
    else if (vvp == 1) return 1;

    bool is_sup;
    if (vvp > 0.5) {
        is_sup = true;
        h = (vvp - 0.5) * 2;
    } else {
        is_sup = false;
        h = (0.5 - vvp) * 2;
    }

    float alpha = acos(h) * 2;
    a = (alpha - sin(alpha)) / M_PI2;

    if (is_sup) vap = 1 - a;
    else vap = a;

    return vap;
}

float3 compute_sky_light(float3 ro, float3 rd) {
    float t1;
    if (!intersect_with_atmosphere(ro, rd, t1) || t1 < 0) {
        return 0;
    }

    float s1 = t1 / SKYLIGHT_NB_VIEWDIR_SAMPLES;
    float t = 0;

    float3 sun_dir = worldLightDirection.xyz;
    float mu = dot(rd, sun_dir);

    float mu2 = mu * mu;
    float mc2 = SUN_MC * SUN_MC;

    // rayleigh
    float3 sumr = 0;
    float odr = 0;
    float phase_r = (3 / (16 * M_PI)) * (1 + mu2);

    // mie
    float3 summ = 0;
    float odm = 0;
    float phase_m = ((3 / (8 * M_PI)) * ((1 - mc2) * (1 + mu2))) /
                    ((2 + mc2) * pow(1 + mc2 - 2 * SUN_MC * mu, 1.5));

    for (uint i = 0; i < SKYLIGHT_NB_VIEWDIR_SAMPLES; ++i) {
        float3 sp = ro + rd * (t + 0.5 * s1);
        float h = length(sp) - EARTH_RS;
        float hr = exp(-h / EARTH_SH_R) * s1;
        odr += hr;
        float hm = exp(-h / EARTH_SH_M) * s1;
        odm += hm;
        float tm;
        float sp_alt = M_PI / 2 - asin(EARTH_RS / max(length(sp), 0.0001));
        sp_alt += acos(normalize(sp).y);
        float coef = compute_sun_visibility(sp_alt);
        if (intersect_with_atmosphere(sp, sun_dir, tm) || coef > 0) {
            float sll = tm / SKYLIGHT_NB_SUNDIR_SAMPLES;
            float odlr = 0;
            float odlm = 0;
            for (uint j = 0; j < SKYLIGHT_NB_SUNDIR_SAMPLES; ++j) {
                float3 spl = sp + sun_dir * ((j + 0.5) * sll);
                float spl_alt = M_PI / 2 - asin(EARTH_RS / max(length(spl), 0.0001));
                spl_alt += acos(normalize(spl).y);
                float coefl = compute_sun_visibility(spl_alt);
                float hl = length(spl) - EARTH_RS;
                odlr += exp(-hl / EARTH_SH_R) * sll * (1 - log(coefl + 0.000001));
                odlm += exp(-hl / EARTH_SH_M) * sll * (1 - log(coefl + 0.000001));
            }
            float3 tau_m = EARTH_BETA_M * 1.05 * (odm + odlm);
            float3 tau_r = EARTH_BETA_R * (odr + odlr);
            float3 tau = tau_m + tau_r;
            float3 attenuation = exp(-tau);
            sumr += hr * attenuation * coef;
            summ += hm * attenuation * coef;
        }
        t += s1;
    }
    float direct_coef = 1;
    if (acos(mu) < SUN_AD * 0.6) {
        direct_coef = 50 + sin(mu / (SUN_AD * 0.5)) * 3;
    }
    return 0.8 * worldLightColor.rgb * direct_coef * (sumr * phase_r * EARTH_BETA_R + summ * phase_m * EARTH_BETA_M);
}
