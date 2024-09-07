#include <filesystem>

#include "SSException.h"
#include "SSConverter3Lib/fnt.h"
#include "pugixml/pugixml.hpp"

std::shared_ptr<SpriteStudio::FntInformation> SpriteStudio::Fnt::parse(const std::string &fnt, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    auto fntInfo = std::make_shared<SpriteStudio::FntInformation>();

    // TODO: impl parse fnt

    return fntInfo;
}
