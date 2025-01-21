#pragma once
#include "Generator.h"

class XorShift32 : public Generator {

protected:
    uint32_t SeedNow;

protected:
    inline void init_genrand(uint32_t seed) {
        SeedNow = seed;
    }

    inline uint32_t genrand_uint32() {
        SeedNow = SeedNow ^ (SeedNow << 13);
        SeedNow = SeedNow ^ (SeedNow >> 17);
        SeedNow = SeedNow ^ (SeedNow << 15);

        return (SeedNow);
    }

    inline float genrand_float32() {
        return ((float) ((genrand_uint32() >> 9) & 0x007fffff) * (1.0f / 8388607.0f));
    }

    inline double genrand_real1() {
        return (genrand_uint32() * (1.0 / 4294967295.0));    /* divided by 2^32-1 */
    }

    inline double genrand_real2() {
        return (genrand_uint32() * (1.0 / 4294967296.0));    /* divided by 2^32 */
    }

    inline int32_t genrand_N(int32_t iN) {
        return ((int32_t) (genrand_uint32() * (iN / 4294967296.0)));
    }

public:
    XorShift32();

    ~XorShift32() override = default;

    void InitSeed(uint32_t seed) override;

    const uint32_t *ListGetSeed(int *lengthList) override;

    uint32_t RandomUint32() override;

    double RandomDouble(double valueMax = 1.0) override;

    float RandomFloat(float valueMax = 1.0f) override;

    int32_t RandomN(int32_t valueMax) override;
};
