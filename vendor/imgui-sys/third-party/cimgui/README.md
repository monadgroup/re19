# cimgui [![Build Status](https://travis-ci.org/sonoro1234/cimgui.svg?branch=master)](https://travis-ci.org/sonoro1234/cimgui)


This is a thin c-api wrapper programmatically generated for the excellent C++ immediate mode gui [Dear ImGui](https://github.com/ocornut/imgui).
All functions are programmatically wrapped except `ImVector` constructors and destructors. (Unless someone find a use case for them)(Now they exist for `ImVector_ImWchar`)
Generated files are: `cimgui.cpp`, `cimgui.h` for C compilation. Also for helping in bindings creation, `definitions.lua` with function definition information and `structs_and_enums.lua`.
This library is intended as a intermediate layer to be able to use Dear ImGui from other languages that can interface with C (like D - see [D-binding](https://github.com/Extrawurst/DerelictImgui))

History:

Initially cimgui was developed by Stephan Dilly as hand-written code but lately turned into an auto-generated version by sonoro1234 in order to keep up with imgui more easily (letting the user select the desired branch and commit)

Notes:
* currently this wrapper is based on version [1.67 of Dear ImGui]
* only functions, structs and enums from imgui.h are wrapped.
* if you are interested in imgui implementations you should look LuaJIT-ImGui project.
* overloaded function names try to be the most compatible with traditional cimgui names. So all naming is algorithmic except for those names that were in conflict with widely used cimgui names and were thus coded in a table (https://github.com/cimgui/cimgui/blob/master/generator/generator.lua#L58). Current overloaded function names can be found in (https://github.com/cimgui/cimgui/blob/master/generator/output/overloads.txt)

# compilation

* clone 
  * git clone --recursive https://github.com/cimgui/cimgui.git
  * git submodule update
* compile 
  * using makefile on linux/macOS/mingw (Or use CMake to generate project)
  * or as in https://github.com/sonoro1234/LuaJIT-ImGui/tree/master_auto_implementations/build

# using generator

* this is only needed if you want an imgui version different from the one provided, otherwise generation is already done.
* you will need LuaJIT (https://github.com/LuaJIT/LuaJIT.git better 2.1 branch) or precompiled for linux/macOS/windows in https://luapower.com/luajit/download
* you can use also a C++ compiler for doing preprocessing: gcc (In windows MinGW-W64-builds for example), clang or cl (MSVC) or not use a compiler (experimental nocompiler option) at all. (this repo was done with gcc)
* update `imgui` folder to the version you desire.
* edit `generator/generator.bat` (or make a .sh version and please PR) to choose between gcc, clang, cl or nocompiler. Run it with gcc, clang or cl and LuaJIT on your PATH.
* as a result some files are generated: `cimgui.cpp` and `cimgui.h` for compiling and some lua/json files with information about the binding: `definitions.json` with function info, `structs_and_enums.json` with struct and enum info, `impl_definitions.json` with functions from the implementations info. 

# generate binding
* C interface is exposed by cimgui.h when you define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
* with your prefered language you can use the lua or json files generated as in:
  * https://github.com/sonoro1234/LuaJIT-ImGui/blob/master_auto_implementations/lua/build.bat (with lua code generation in https://github.com/sonoro1234/LuaJIT-ImGui/blob/master_auto_implementations/lua/class_gen.lua)
  * https://github.com/mellinoe/ImGui.NET/tree/autogen/src/CodeGenerator
### definitions description
* It is a collection in which key is the cimgui name that would result without overloadings and the value is an array of overloadings (may be only one overloading)
* Each overloading is a collection. Some relevant keys and values are:
  * stname : the name of the struct the function belongs to (may be ImGui if it is top level in ImGui namespace)
  * ov_cimguiname : the overloaded cimgui name (if absent it would be taken from cimguiname)
  * cimguiname : the name without overloading (this should be used if there is not ov_cimguiname)
  * ret : the return type
  * argsT : an array of collections (each one with type: argument type and name: the argument name)
  * args : a string of argsT concatenated and separated by commas
  * call_args : a string with the argument names separated by commas for calling imgui function
  * defaults : a collection in which key is argument name and value is the default value.
  * manual : will be true if this function is hand-written (not generated)
  * isvararg : is setted if some argument is a vararg
  * constructor : is setted if the function is a constructor for a class
  * destructor : is setted if the functions is a destructor for a class
  * nonUDT : if present can be 1 or 2 (explained meaning in usage) if return type was a user defined type
### structs_and_enums description
* Is is a collection with two items:
  * under key enums we get the enums collection in which each key is the enum tagname and the value is an array of the ordered values represented as a collection with keys
    * name : the name of this enum value
    * value : the C string
    * calc_value : the numeric value corresponding to value
  * under key structs we get the structs collection in which the key is the struct name and the value is an array of the struct members. Each one given as a collection with keys
    * type : the type of the struct member
    * template_type : if type has a template argument (as ImVector) here will be
    * name : the name of the struct member
# usage

* use whatever method is in ImGui c++ namespace in the original [imgui.h](https://github.com/ocornut/imgui/blob/master/imgui.h) by prepending `ig`
* methods have the same parameter list and return values (where possible)
* functions that belong to a struct have an extra first argument with a pointer to the struct.
* where a function returns UDT (user defined type) by value some compilers complain so another function with the name `function_name_nonUDT` is generated accepting a pointer to the UDT type as the first argument.
* also is generated `function_name_nonUDT2` which instead of returning the UDT type returns a simple version (without functions) called `UDTType_Simple` (`ImVec2_Simple` for `ImVec2`)

# example bindings based on cimgui

* [DerelictImgui](https://github.com/Extrawurst/DerelictImgui)
* [ImGui.NET](https://github.com/mellinoe/ImGui.NET)
* [ImGuiCS](https://github.com/conatuscreative/ImGuiCS)
* [imgui-rs](https://github.com/Gekkio/imgui-rs)
* [imgui-pas](https://github.com/dpethes/imgui-pas)
* [odin-imgui](https://github.com/ThisDrunkDane/odin-imgui)
* [LuaJIT-imgui](https://github.com/sonoro1234/LuaJIT-ImGui)
