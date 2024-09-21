#pragma once

#include <string>
#include <memory>

#include "version.h"
#include "ssfb2/common_ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJInformation;

    enum class SSSEVersion : uint32_t {
        ERROR = static_cast<uint32_t>(Version::ERROR),
        CODE_010000 = static_cast<uint32_t>(Version::CODE_010000),
    };
    const std::map<uint32_t, std::string> SSSEVersionMap = {
        {static_cast<uint32_t>(SSSEVersion::CODE_010000), VersionToStringMap.at(static_cast<uint32_t>(SSSEVersion::CODE_010000))},
    };

    class SSSEInformation {
    public:
        std::string ssse;
        std::string NameDirectory();
        std::string NameFileBody();
        std::string NameFileExtension();
        std::string NameFile();

        Version version;

        int ID;
        std::string Name;
        std::vector<std::shared_ptr<ss::ssfb2::LibraryDataSoundT>> TableSound;

        // public LibraryEditor_SpriteStudio6.Import.Assets<Script_SpriteStudio6_DataSoundList> DataSoundListSS6PU;
    };

    class SSSE {
    public:
        std::shared_ptr<SSSEInformation> parse(const std::string& ssse, const std::shared_ptr<SSPJInformation> &sspjInfo);
    };

}
