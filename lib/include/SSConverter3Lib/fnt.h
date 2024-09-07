#pragma once
#include <memory>

namespace SpriteStudio {
    class SSPJInformation;

    class FntInformation {

    };

    class Fnt {
    public:
        std::shared_ptr<FntInformation> parse(const std::string &fnt, const std::shared_ptr<SSPJInformation>& sspjInfo);
    };
}
