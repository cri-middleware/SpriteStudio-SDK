#include <filesystem>

#include "SSException.h"
#include "SSConverter3Lib/ssse.h"
#include "pugixml/pugixml.hpp"

using namespace SpriteStudio;

std::shared_ptr<SSSEInformation> SSSE::parse(const std::string& ssse, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    auto ssse_path = std::filesystem::path(ssse);
    if (!std::filesystem::exists(ssse_path)) {
        throw SSException("not found a ssse file: " + ssse, -1);
    }

    pugi::xml_document doc;
    auto result = doc.load_file(ssse_path.c_str());
    if (!result) {
        throw SSException("failed loading a ssse file", -1);
    }

    auto nodeRoot = doc.first_child();
    if (std::string(nodeRoot.name()) != "SpriteStudioProject") {
        throw SSException("invalid sspj format", -1);
    }

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw SSException("invalid sspj format", -1);
    }
    auto versionText = std::string(attr.value());

    auto ssseInfo = std::make_shared<SSSEInformation>();
    if (ssseInfo == nullptr) {
        throw SSException("not enough memory", -1);
    }


    // TODO: impl parse ssse

    return ssseInfo;
}
