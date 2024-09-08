#include "util.h"

using namespace SpriteStudio;

uint32_t util::getVersion(const std::string &valueText, const std::map<uint32_t, std::string> &map) {
    for (const auto& pair : map) {
        if (pair.second == valueText) {
            return pair.first;
        }
    }

    return static_cast<uint32_t>(Version::ERROR);
}
