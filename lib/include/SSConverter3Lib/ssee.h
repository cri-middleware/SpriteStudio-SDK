#pragma once

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

    };

    class SSEE {
    public:

    };
}
