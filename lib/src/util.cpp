#include "util.h"

using namespace SpriteStudio;

Version util::getVersion(const std::string &valueText) {
    for (const auto& pair : VersionToStringMap) {
        if (pair.second == valueText) {
            return pair.first;
        }
    }

    return Version::ERROR;
}
