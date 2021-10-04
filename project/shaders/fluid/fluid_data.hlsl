cbuffer FluidData : register(b0) {
    int3 map_size;
    float delta_time;
    float3 input_pos;
    float vorticity_strength;
    float3 input_radius;
    float density_amount;
    float3 velocity_amount;
    float density_dissipation;
    float density_buoyancy;
    float density_weight;
    float temperature_amount;
    float temperature_dissipation;
    float velocity_dissipation;
};
