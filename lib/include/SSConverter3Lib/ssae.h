#pragma once

#include "version.h"
#include "ssfb2/ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJInformation;

    class SSAEInformation {
    public:
        std::string ssae;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;

        Version version;

        ss::ssfb2::LibraryDataAnimationT data;
        std::vector<std::shared_ptr<ss::ssfb2::DataModelPartsT>> TableParts;
    };

    class SSAE {
    public:
        SSAE();
        ~SSAE();

        std::shared_ptr<SSAEInformation> parse(const std::string& ssae, const std::shared_ptr<SSPJInformation> &sspjInfo);
    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;
    };
}
