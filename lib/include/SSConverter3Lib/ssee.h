#pragma once

#include <cstdint>
#include <map>

#include "version.h"

namespace SpriteStudio {
    class SSPJInformation;

    enum class SSEEVersion : uint32_t {
        ERROR = static_cast<uint32_t>(Version::ERROR),
        CODE_020000 = static_cast<uint32_t>(Version::CODE_020000),
    };
    const std::map<uint32_t, std::string> SSEEVersionMap = {
        {static_cast<uint32_t>(SSEEVersion::CODE_020000), VersionToStringMap.at(static_cast<uint32_t>(SSEEVersion::CODE_020000))},
    };

    class SSEEInformation {
        std::string ssee;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;

        SSEEVersion version;

    };

    class SSEE {
    public:

    };
}
