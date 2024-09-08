#pragma once

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
    };

    class SSQE {
    public:

    };
}
