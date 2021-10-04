#define NUM_OCTAVES 5

float mod289(float x){return x - floor(x * (1.0 / 289.0)) * 289.0;}
float4 mod289(float4 x){return x - floor(x * (1.0 / 289.0)) * 289.0;}
float4 perm(float4 x){return mod289(((x * 34.0) + 1.0) * x);}

float noise_procedural(float3 p) {
    float3 a = floor(p);
    float3 d = p - a;
    d = d * d * (3.0 - 2.0 * d);

    float4 b = a.xxyy + float4(0.0, 1.0, 0.0, 1.0);
    float4 k1 = perm(b.xyxy);
    float4 k2 = perm(k1.xyxy + b.zzww);

    float4 c = k2 + a.zzzz;
    float4 k3 = perm(c);
    float4 k4 = perm(c + 1.0);

    float4 o1 = frac(k3 * (1.0 / 41.0));
    float4 o2 = frac(k4 * (1.0 / 41.0));

    float4 o3 = o2 * d.z + o1 * (1.0 - d.z);
    float2 o4 = o3.yw * d.x + o3.xz * (1.0 - d.x);

    return o4.y * d.y + o4.x * (1.0 - d.y);
}

float fbm_procedural(float3 x) {
	float v = 0.0;
	float a = 0.5;
	float3 shift = 100;
	for (int i = 0; i < NUM_OCTAVES; ++i) {
		v += a * noise_procedural(x);
		x = x * 2.0 + shift;
		a *= 0.5;
	}
	return v;
}

float rand(float n){return frac(sin(n) * 43758.5453123);}

float noise(float3 x, SamplerState smp, Texture3D noise_tex) {
	return noise_tex.SampleLevel(smp, x/256, 0).x;
}

float fbm(float3 x, SamplerState smp, Texture3D noise_tex) {
	float v = 0.0;
	float a = 0.5;
	float3 shift = 100;
	for (int i = 0; i < NUM_OCTAVES; ++i) {
		v += a * noise(x, smp, noise_tex);
		x = x * 2.0 + shift;
		a *= 0.5;
	}
	return v;
}