#pragma once

#include <string>
#include <memory>

namespace SpriteStudio {
    class SSPJInformation;

    class SSSEInformation {
    public:
        //LibraryEditor_SpriteStudio6.Import.SSSE.KindVersion Version;
        std::string NameDirectory;
        std::string NameFileBody;
        std::string NameFileExtension;

        int ID;
        std::string Name;
        // std::vector<Sound> TableSound;

        // public LibraryEditor_SpriteStudio6.Import.Assets<Script_SpriteStudio6_DataSoundList> DataSoundListSS6PU;
    };

    class SSSE {
    public:
        std::shared_ptr<SSSEInformation> parse(const std::string& ssse, const std::shared_ptr<SSPJInformation> &sspjInfo);
    };

}
