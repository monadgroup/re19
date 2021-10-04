RWTexture3D<int> boundary_map : register(u0);

[numthreads(32, 32, 1)]
void main(int3 pos : SV_DispatchThreadID) {
    //int3 box_size = int3(50, 20, 50);
    //int3 box_top_left = int3(64, 118, 64) - box_size / 2;
    //int3 box_bottom_right = box_top_left + box_size;


    int booster_height = 32;
    int main_height = 16;
    int main_width = 8;
    int support_width = 36;
    int support_depth = 26;
    
    bool in_support = pos.x >= 64-support_width && pos.x < 64+support_width && pos.z >= 128-support_depth;// && pos.z < 64+support_depth;
    bool in_main = pos.x >= 64-main_width && pos.x < 64+main_width;

    int support_height = 0;
    if (in_support) support_height = booster_height;
    if (in_support && in_main) support_height = main_height;

    boundary_map[pos] = pos.y < support_height;//pos.y > 100;//all(pos > box_top_left) && all(pos < box_bottom_right);
}
