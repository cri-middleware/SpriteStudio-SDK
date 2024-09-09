#include <filesystem>

#include "SSConverter3Lib/ssqe.h"

#include "error.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"

using namespace SpriteStudio;
const DocPartType DOC_PART = DocPartType::SSQE;

std::string SSQEInformation::NameDirectory() const {
    return util::getNameDirectory(ssqe);
}

std::string SSQEInformation::NameFileBody() const {
    return util::getNameFileBody(ssqe);
}

std::string SSQEInformation::NameFileExtension() const {
    return util::getNameFileExtension(ssqe);
}
