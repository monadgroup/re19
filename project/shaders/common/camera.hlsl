cbuffer Camera : register(b1) {
    float3 camPosition;
    float camFovRadians;
    float4 camDirection;
    float4 zRange;
    matrix viewMatrix;
    matrix projMatrix;
    matrix viewProjMatrix;
    matrix lastViewProjMatrix;
    matrix invViewMatrix;
    matrix invProjMatrix;
    matrix normViewMatrix;
};
