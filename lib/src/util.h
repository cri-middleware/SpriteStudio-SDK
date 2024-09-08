#pragma once

#include <string>
#include "../include/SSConverter3Lib/version.h"

namespace SpriteStudio {
    class util {
    public:
        static uint32_t getVersion(const std::string &valueText, const std::map<uint32_t, std::string>& map);
    };
}
