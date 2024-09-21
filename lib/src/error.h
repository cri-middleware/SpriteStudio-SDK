#pragma once
#include <map>
#include <string>

#include "phase.h"
#include "SSException.h"

namespace SpriteStudio {
    class error {
    public:
        static std::string getErrorMessage(PhaseType phaseType, DocPartType docPartType, const std::string& message);
        static SSException getException(PhaseType phaseType, DocPartType docPartType, const std::string& message, int code);
    };
}
