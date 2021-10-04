struct VSOut {
    float4 sv_pos: SV_POSITION;
    float2 tex_pos : TEXCOORD;
};

VSOut main(float2 clip_pos : POSITION, float2 tex_pos : TEXCOORD) {
    VSOut output;
    output.sv_pos = float4(clip_pos, 0, 1);
    output.tex_pos = tex_pos;
    return output;
}
