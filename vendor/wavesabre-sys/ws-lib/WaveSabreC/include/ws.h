#pragma once

#include <cstdint>

extern "C" {
void AudioInit(uint8_t isPrerender, void (*prerenderCallback)(double, void *), void *prerenderData);
void AudioPlay();
double AudioGetPos();
}
