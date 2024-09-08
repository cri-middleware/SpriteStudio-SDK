#pragma once

#include <string>
#include "SSConverter3Lib/version.h"

namespace SpriteStudio {
    class util {
    public:
        static uint32_t getVersion(const std::string &valueText, const std::map<uint32_t, std::string>& map);
        static std::string getNameDirectory(const std::string& file_path);
        static std::string getNameFileBody(const std::string& file_path);
        static std::string getNameFileExtension(const std::string& file_path);
    };
}
