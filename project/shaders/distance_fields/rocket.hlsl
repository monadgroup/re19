#include "common.hlsl"

static const float BODY_HEIGHT = 4.4;
static const float BODY_GROUND_OFFSET = 0.8;
static const float FAIRING_HEIGHT = 1.6;
static const float BODY_RADIUS = 0.5;
static const float BOOSTER_HEIGHT = 2.5;
static const float BOOSTER_GROUND_OFFSET = 0.3;
static const float BOOSTER_RADIUS = 0.3;
static const float BOOSTER_CAP_HEIGHT = 1.2;
static const float BOOSTER_SEP = 0.06;

static const float BOOSTER_CURVE_SIZE = 0.3;
static const float BOOSTER_CURVE_X_RADIUS = 0.6;
static const float BOOSTER_CURVE_Y_RADIUS = 0.6;
static const float BOOSTER_Z = BOOSTER_CURVE_SIZE - (1 - cos(asin(BOOSTER_CURVE_SIZE/BOOSTER_CURVE_Y_RADIUS)))*BOOSTER_CURVE_X_RADIUS;

void fRocketBody(float3 p, out float dist, out float boosterSide, out float3 boosterP) {
    float3 rocketP = p - float3(0, BODY_GROUND_OFFSET + BODY_HEIGHT, 0);
    float x = clamp((BODY_HEIGHT-rocketP.y)/FAIRING_HEIGHT, 0, 1);
    float y1 = pow(x, 0.6);
    float y2 = 1 - pow(1 - x, 3);
    float y = lerp(y1, y2, x);
    dist = fCylinder(rocketP, y*BODY_RADIUS, BODY_HEIGHT);
    dist = min(dist, fCone(rocketP + float3(0, BODY_HEIGHT+0.8, 0), BODY_RADIUS/2, 1.5));
    dist = min(dist, fDisc(rocketP - float3(0, 1, 0), BODY_RADIUS) - 0.05);
    dist = min(dist, fDisc(rocketP - float3(0, 0.5, 0), BODY_RADIUS) - 0.05);

    boosterP = p - float3(0, BOOSTER_GROUND_OFFSET + BOOSTER_HEIGHT, 0);
    boosterSide = pMirror(boosterP.x, BODY_RADIUS + BOOSTER_SEP);
}

void fRocketBooster(inout float3 boosterP, float bodyDist, out float dist, out float ao) {
    float x = clamp((BOOSTER_HEIGHT-boosterP.y+BOOSTER_CURVE_SIZE/2+0.12)/BOOSTER_CAP_HEIGHT, 0, 1);
    float y1 = x;
    float y2 = BOOSTER_CURVE_Y_RADIUS * sin(acos(1 - (x - BOOSTER_Z) / BOOSTER_CURVE_X_RADIUS));
    float y = lerp(saturate(y2), y1, step(BOOSTER_CURVE_SIZE, x));
    boosterP.x -= x*BOOSTER_RADIUS;
    dist = fCylinder(boosterP, y*BOOSTER_RADIUS, BOOSTER_HEIGHT);
    dist = min(dist, fDisc(boosterP + float3(0, 1, 0), BOOSTER_RADIUS) - 0.03);
    dist = min(dist, fCone(boosterP + float3(0, BOOSTER_HEIGHT, 0), BOOSTER_RADIUS*1.2, 1));
    if (BOOSTER_ATTACHMENTS) {
        dist = min(dist, fBoxCheap(boosterP - float3(-0.5, 0.8, 0), float3(0.3, 0.3, 0.1)));
    }

    ao = saturate(abs(bodyDist - dist));
}

void fRocket(float3 p, out float dist, out float ao, out float boosterSide, out float3 boosterP) {
    float bodyDist, boosterDist;
    fRocketBody(p, bodyDist, boosterSide, boosterP);
    fRocketBooster(boosterP, bodyDist, boosterDist, ao);
    dist = min(bodyDist, boosterDist);
}

float box(float2 p, float2 size) {
    float2 s1 = step(-size, p);
    float2 s2 = step(p, size);

    return s1.x * s1.y * s2.x * s2.y;
}

void rocketMaterial(inout float3 p, float3 n, inout float3 albedo, inout float metallic, inout float roughness, inout float3 emissive) {
    albedo = 0.99;
    metallic = 0.3;
    roughness = 0.6;
    emissive = 0;

    float is_ensign = step(distance(p.xy, float2(0, 7.5)), 0.15);
    albedo = lerp(albedo, float3(0.1, 0.1, 1), is_ensign);

    p.y -= 4;
    pMirror(p.x, 0.8);
    float size = 0.18;
    albedo = lerp(albedo, float3(1, 0.6, 0.6), box(p.xy, size));
    albedo = lerp(albedo, float3(0.6, 0.6, 1), box(p.xy + float2(0, 0.5), size));
    albedo = lerp(albedo, float3(1, 0.6, 0.6), box(p.xy + float2(0, 1), size));
}
