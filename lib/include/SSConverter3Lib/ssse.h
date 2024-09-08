#pragma once

#include <string>
#include <memory>

#include "version.h"
#include "ssfb2/ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJInformation;

    class SSSEInformation {
    public:
        std::string ssse;
        std::string NameDirectory();
        std::string NameFileBody();
        std::string NameFileExtension();

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
