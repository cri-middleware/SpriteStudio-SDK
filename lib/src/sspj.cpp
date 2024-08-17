#include "sspj.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"

using namespace SpriteStudio;

std::shared_ptr<SSPJ::Information> SSPJ::parser(const std::filesystem::path &sspj) {
    pugi::xml_document doc;
    auto result = doc.load_file(sspj.c_str());
    if (!result) {
        throw SSException("failed loading a sspj file", -1);
    }

    auto nodeRoot = doc.first_child();
    if (nodeRoot.next_sibling().empty()) {
        throw SSException("invalid sspj format", -1);
    }

    nodeRoot = nodeRoot.next_sibling();
    if (std::string(nodeRoot.name()) != "SpriteStudioProject") {
        throw SSException("invalid sspj format", -1);
    }

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw SSException("invalid sspj format", -1);
    }
    auto versionText = std::string(attr.value());

    auto information = std::make_shared<SSPJ::Information>();
    if (information == nullptr) {
        throw SSException("not enough memory", -1);
    }
    information->sspj = sspj;

    pugi::xpath_node xpath_node;
    std::string valueText;

    xpath_node = nodeRoot.select_node("settings/imageBaseDirectory");
    xpath_node.node().value();

    return information;
}

