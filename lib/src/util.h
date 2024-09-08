#pragma once

#include <string>
#include "../include/SSConverter3Lib/version.h"

namespace SpriteStudio {
    class util {
    public:
        static Version getVersion(const std::string &valueText);
    };
}
