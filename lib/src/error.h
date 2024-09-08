#pragma once
#include <string>

#include "SSException.h"

namespace SpriteStudio {
    enum class XmlType : uint32_t {
        SSPJ,

    };
    class error {
    public:
        static std::string getErrorMessage(const std::string& tag, const std::string& message);
        static SSException getException(const std::string& tag, const std::string& message, int code);
    };
}
