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


    class error {
    public:
        static std::string getErrorMessage(DocPartType docPartType, const std::string& message);
        static SSException getException(DocPartType docPartType, const std::string& message, int code);
    };
}
