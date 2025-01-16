#pragma once
#include <memory>
#include <cstddef>
#include <cstdint>

class Generator {
public:
    std::shared_ptr<void> Pointer;

private:
    inline void CleanUp() {
        Pointer = nullptr;
    }

public:
    Generator() {
        CleanUp();
    }

    virtual ~Generator() {
        CleanUp();
    }

    virtual void InitSeed(uint32_t seed) = 0;

    virtual const uint32_t *ListGetSeed(int *lengthList) = 0;

    virtual uint32_t RandomUint32() = 0;

    virtual double RandomDouble(double valueMax = 1.0f) = 0;

    virtual float RandomFloat(float valueMax = 1.0f) = 0;

    virtual int32_t RandomN(int32_t valueMax) = 0;
};

namespace Text {
    inline bool IsNullOrEmpty(const char *text) {
        return (!((nullptr != text) && (0x00 != *text)));
    }

    bool Compare(const char *lhs, const char *rhs, size_t length = 0);
}
