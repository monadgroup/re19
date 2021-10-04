#include <Windows.h>

void *operator new(size_t s) {
    return HeapAlloc(GetProcessHeap(), 0, s);
}

void operator delete(void *p) noexcept {
    HeapFree(GetProcessHeap(), 0, p);
}

void operator delete(void *p, size_t) noexcept {
    HeapFree(GetProcessHeap(), 0, p);
}

void *operator new[](size_t s) {
    return HeapAlloc(GetProcessHeap(), 0, s);
}

void operator delete[](void *p) noexcept {
    HeapFree(GetProcessHeap(), 0, p);
}

void operator delete[](void *p, size_t) noexcept {
    HeapFree(GetProcessHeap(), 0, p);
}
