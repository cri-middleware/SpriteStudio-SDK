#pragma once

#include <cstdint>
#include <map>

#include "version.h"
#include "ssfb2/common_ssfb2_generated.h"

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
    public:
        std::string ssqe;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;
        std::string NameFile() const;

        SSQEVersion version;

        std::string Name;
        std::string ExportPath;
        std::vector<std::shared_ptr<ss::ssfb2::DataSequenceT>> SequenceList;

        class InUseAnimation {
        public:
            std::string NameAnimationPack;
            int IndexAnimationPack;
            std::string NameAnimation;
        };
        std::vector<std::shared_ptr<InUseAnimation>> ListInUseAnimation;

        // public LibraryEditor_SpriteStudio6.Import.Assets<Script_SpriteStudio6_DataSequence> DataSequenceSS6PU; // TODO: impl

        void InUseAnimationAdd(const std::string& nameAnimationPack, const std::string& nameAnimation);
        void InUseAnimationIndexing(const std::shared_ptr<SSPJInformation> &sspjInfo);

    private:
        std::mutex m;
    };

    class SSQE {
    public:
        std::shared_ptr<SSQEInformation> parse(const std::string& ssqe, const std::shared_ptr<SSPJInformation> &sspjInfo);
    };
}
