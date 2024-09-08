#pragma once

#include <cstdint>
#include <map>

#include "version.h"

namespace SpriteStudio {
    class SSPJInformation;

    enum class SSQEVersion : uint32_t {
        ERROR = static_cast<uint32_t>(Version::ERROR),
        CODE_010000 = static_cast<uint32_t>(Version::CODE_010000),
        CODE_020000 = static_cast<uint32_t>(Version::CODE_020000),
    };
    const std::map<uint32_t, std::string> SSQEVersionMap = {
        {static_cast<uint32_t>(SSQEVersion::CODE_010000), VersionToStringMap.at(static_cast<uint32_t>(SSQEVersion::CODE_010000))},
        {static_cast<uint32_t>(SSQEVersion::CODE_020000), VersionToStringMap.at(static_cast<uint32_t>(SSQEVersion::CODE_020000))},
    };

    class SSQEInformation {
        std::string ssqe;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;

        SSQEVersion version;
    };

    class SSQE {
    public:

    };
}
