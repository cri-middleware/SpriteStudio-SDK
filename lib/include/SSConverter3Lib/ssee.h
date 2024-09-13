#pragma once

#include <cstdint>
#include <map>

#include "version.h"
#include "ssfb2/common_ssfb2_generated.h"

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
    public:
        std::string ssee;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;

        SSEEVersion version;

        int VersionRenderer;
        ss::ssfb2::Vector2T ScaleLayout;
        int SeedRandom;
        bool FlagLockSeed;
        int FramePerSecond;

        std::vector<std::shared_ptr<ss::ssfb2::DataEffectPartsT>> TableParts;

        // TODO: impl
    };

    class SSEE {
    public:
        SSEE();
        ~SSEE();

        std::shared_ptr<SSEEInformation> parse(const std::string& ssee, const std::shared_ptr<SSPJInformation>& sharedsspjInfo);
    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;
    };
}
