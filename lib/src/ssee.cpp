#include <filesystem>

#include "SSConverter3Lib/ssee.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"

std::string SpriteStudio::SSEEInformation::NameDirectory() const {
    return util::getNameDirectory(ssee);
}

std::string SpriteStudio::SSEEInformation::NameFileBody() const {
    return util::getNameFileBody(ssee);
}

std::string SpriteStudio::SSEEInformation::NameFileExtension() const {
    return util::getNameFileExtension(ssee);
}
