#include <filesystem>

#include "SSConverter3Lib/ssqe.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"

std::string SpriteStudio::SSQEInformation::NameDirectory() const {
    return util::getNameDirectory(ssqe);
}

std::string SpriteStudio::SSQEInformation::NameFileBody() const {
    return util::getNameFileBody(ssqe);
}

std::string SpriteStudio::SSQEInformation::NameFileExtension() const {
    return util::getNameFileExtension(ssqe);
}
