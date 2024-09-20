#pragma once
#include <map>
#include <string>

#include "SSException.h"

namespace SpriteStudio {
    enum class DocPartType : uint32_t {
        SSPJ,
        SSCE,
        SSAE,
        SSEE,
        SSQE,
        SSSE,
        FNT,
    };

    enum class PhaseType : uint32_t {
        PARSE,
        CONVERT,
        COMPRESS,
        PACKING,
    };

    class error {
    public:
        static std::string getErrorMessage(PhaseType phaseType, DocPartType docPartType, const std::string& message);
        static SSException getException(PhaseType phaseType, DocPartType docPartType, const std::string& message, int code);
    };
}
