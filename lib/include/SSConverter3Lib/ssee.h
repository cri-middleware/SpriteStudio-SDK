#pragma once

#include <cstdint>
#include <map>

#include "SSConverter3Lib.h"
#include "version.h"
#include "ssfb2/type_base_generated.h"
#include "ssfb2/Data/common_ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJInformation;

    enum class SSEEVersion : uint32_t {
        ERROR = static_cast<uint32_t>(Version::ERROR),
        CODE_010002 = static_cast<uint32_t>(Version::CODE_010002),
        CODE_010100 = static_cast<uint32_t>(Version::CODE_010100),
        CODE_020000 = static_cast<uint32_t>(Version::CODE_020000),
    };
    const std::map<uint32_t, std::string> SSEEVersionMap = {
        {static_cast<uint32_t>(SSEEVersion::CODE_010002), VersionToStringMap.at(static_cast<uint32_t>(SSEEVersion::CODE_010002))},
        {static_cast<uint32_t>(SSEEVersion::CODE_010100), VersionToStringMap.at(static_cast<uint32_t>(SSEEVersion::CODE_010100))},
        {static_cast<uint32_t>(SSEEVersion::CODE_020000), VersionToStringMap.at(static_cast<uint32_t>(SSEEVersion::CODE_020000))},
    };

    class SSEEInformation {
    public:
        std::string ssee;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;
        std::string NameFile() const;

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

        std::shared_ptr<SSEEInformation> parse(const std::string& ssee, const std::shared_ptr<SSPJInformation>& sspjInfo);

        void DataConverter(const ConvertOptions& options, const std::shared_ptr<SSPJInformation>& sspjInfo, const std::shared_ptr<SSEEInformation>& item);

        void DataOutput(const ConvertOptions& options, const std::shared_ptr<SSPJInformation>& sspjInfo, const std::shared_ptr<SSEEInformation>& item);

    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;

        struct WorkAreaRebuildPartsDataConvert {
            int IndexParts{-1};
            int IndexPartsChildParicle{-1};
            int IndexEmitter{-1};
        };

        struct WorkAreaBuildDrawOrderDataConvert {
            int IndexEmitter{-1};
            float PriorityParticle{0.0f / 0.0f};
        };

    };
}
