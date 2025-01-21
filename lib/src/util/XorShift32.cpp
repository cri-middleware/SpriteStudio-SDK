#include "XorShift32.h"

XorShift32::XorShift32() : Generator() {
    SeedNow = 0;
}

void XorShift32::InitSeed(uint32_t seed) {
    SeedNow = seed;
}

const uint32_t* XorShift32::ListGetSeed(int* lengthList) {
    if (nullptr != lengthList) {
        *lengthList = 0;
    }
    return (nullptr);
}

uint32_t XorShift32::RandomUint32(void) {
    return (genrand_uint32());
}

double XorShift32::RandomDouble(double valueMax) {
    return ((genrand_uint32() * (valueMax / 4294967296.0)));
}

float XorShift32::RandomFloat(float valueMax) {
    //  return((float)(genrand_uint32() * (valueMax / 4294967296.0f))); /* 8388607.0f */
    //  return((float)(genrand_uint32() * (valueMax * (1.0f/ 4294967296.0f)))); /* 8388607.0f */
    SeedNow = SeedNow ^ (SeedNow << 13);
    SeedNow = SeedNow ^ (SeedNow >> 17);
    SeedNow = SeedNow ^ (SeedNow << 15);
    return ((float) SeedNow * (valueMax * (1.0f / 4294967296.0f)));
}

int32_t XorShift32::RandomN(int32_t valueMax) {
    return (genrand_N(valueMax));
}
