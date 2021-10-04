add_library(WaveSabreC
    include/ws.h
    src/ws.cpp)

target_link_libraries(WaveSabreC WaveSabrePlayerLib WaveSabreCore)

target_include_directories(WaveSabreC PUBLIC include)

if(MSVC)
    target_compile_definitions(WaveSabreC PRIVATE _CRT_SECURE_NO_WARNINGS)
    target_compile_options(WaveSabreC PUBLIC
            $<$<CONFIG:MinSizeRel>:/EHs-c->)
    set_property(TARGET WaveSabreC APPEND_STRING PROPERTY STATIC_LIBRARY_FLAGS_MINSIZEREL
            " /LTCG")
endif()

