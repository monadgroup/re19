#include <ws.h>
#include <WaveSabrePlayerLib.h>
#include <Windows.h>

#include <song.h>

using namespace WaveSabrePlayerLib;

IPlayer *player;

void AudioInit(uint8_t isPrerender, void (*prerenderCallback)(double, void *), void *prerenderData) {
    SYSTEM_INFO systemInfo;
    GetSystemInfo(&systemInfo);

    if (isPrerender) {
        player = new PreRenderPlayer(&Song, systemInfo.dwNumberOfProcessors, prerenderCallback, prerenderData);
    } else {
        int numRenderThreads = systemInfo.dwNumberOfProcessors / 2 - 1;
        if (numRenderThreads < 1) numRenderThreads = 1;

        player = new RealtimePlayer(&Song, numRenderThreads);
    }
}

void AudioPlay() {
    player->Play();
}

double AudioGetPos() {
    return player->GetSongPos();
}
