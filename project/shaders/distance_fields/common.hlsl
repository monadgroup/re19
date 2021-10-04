float vmax(float2 v) {
    return max(v.x, v.y);
}

float vmax(float3 v) {
    return max(max(v.x, v.y), v.z);
}

void pR(inout float2 p, float a) {
    p = cos(a) * p + sin(a) * float2(p.y, -p.x);
}

float fCylinder(float3 p, float r, float height) {
    float d = length(p.xz) - r;
    d = max(d, abs(p.y) - height);
    return d;
}

float fCone(float3 p, float radius, float height) {
    float2 q = float2(length(p.xz), p.y);
    float2 tip = q - float2(0, height);
    float2 mantleDir = normalize(float2(height, radius));
    float mantle = dot(tip, mantleDir);
    float d = max(mantle, -q.y);
    float projected = dot(tip, float2(mantleDir.y, -mantleDir.x));

    // distance to tip
    if ((q.y > height) && (projected < 0)) {
        d = max(d, length(tip));
    }

    // distance to base ring
    if ((q.x > radius) && (projected > length(float2(height, radius)))) {
        d = max(d, length(q - float2(radius, 0)));
    }

    return d;
}

float fDisc(float3 p, float r) {
    float l = length(p.xz) - r;
    return l < 0 ? abs(p.y) : length(float2(p.y, l));
}

float pMirror(inout float p, float dist) {
    float s = sign(p);
    p = abs(p) - dist;
    return s;
}

float fBoxCheap(float3 p, float3 b) {
    return vmax(abs(p) - b);
}

float fBox(float3 p, float3 b) {
    float3 d = abs(p) - b;
    return length(max(d, 0)) + vmax(min(d, 0));
}
