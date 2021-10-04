cbuffer FluidRenderData : register(b2) {
    // Directional light
    matrix scaled_directional_shadow_to_world;
    matrix world_to_directional_shadow;

    // Point light
    float3 point_light_world_pos;
    float point_light_max_radius;
    uint3 point_shadow_map_size;
    float point_light_radius;

    // Density
    matrix world_to_density;
    float3 fluid_box_pos;
    uint shadow_map_depth;
    float3 fluid_box_size;

    uint use_point_light;
    float3 light_color;
    float march_step_length;
    float density_multiplier;
    float rocket_height;
};
