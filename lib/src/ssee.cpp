#include <filesystem>

#include "SSConverter3Lib/ssee.h"

#include "error.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"

using namespace SpriteStudio;
const DocPartType DOC_PART = DocPartType::SSEE;

std::string SSEEInformation::NameDirectory() const {
    return util::getNameDirectory(ssee);
}

std::string SSEEInformation::NameFileBody() const {
    return util::getNameFileBody(ssee);
}

std::string SSEEInformation::NameFileExtension() const {
    return util::getNameFileExtension(ssee);
}
