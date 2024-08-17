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
    information->sspj_path = sspj;

    auto getValueText = [&nodeRoot](const std::string &query) -> std::tuple<std::string, pugi::xml_node> {
        std::string text;
        pugi::xml_node* node;
        if (auto xpath_node = nodeRoot.select_node(query.c_str())) {
            if (auto value = xpath_node.node().text().as_string()) {
                text = std::string(value);
            }
        }
        return std::make_tuple(text, nodeRoot);
    };
    std::string valueText;
    pugi::xml_node node;
    std::tie(valueText, node) = getValueText("settings/imageBaseDirectory");
    information->NameDirectoryBaseTexture = valueText.empty() ? information->NameDirectory() : (sspj / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("settings/cellMapBaseDirectory");
    information->NameDirectoryBaseSSCE = valueText.empty() ? information->NameDirectory() : (sspj / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("settings/animeBaseDirectory");
    information->NameDirectoryBaseSSAE = valueText.empty() ? information->NameDirectory() : (sspj / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("settings/effectBaseDirectory");
    information->NameDirectoryBaseSSEE = valueText.empty() ? information->NameDirectory() : (sspj / std::filesystem::path(valueText)).string();

    information->NameDirectoryBaseSSQE = information->NameDirectory();

    information->NameDirectoryBaseSSSE = information->NameDirectory();


    std::tie(valueText, node) = getValueText("settings/wrapMode");
    if (valueText == "repeat") {
        // TODO:
    } else if (valueText == "mirror") {
        // TODO:
    } else {
        // TODO:
    }

    std::tie(valueText, node) = getValueText("settings/filterMode");
    if (valueText == "nearlest") {
        // TODO:
    } else if (valueText == "linear") {
        // TODO:
    } else {
        // TODO:
    }

    std::tie(valueText, node) = getValueText("settings/convertImageToPMA");
    if (!valueText.empty()) {
        information->FlagConvertImagePremultipliedAlpha = node.text().as_bool();
    }

    std::tie(valueText, node) = getValueText("settings/blendImageAsPMA");
    if (!valueText.empty()) {
        auto value = node.text().as_bool();
        information->FlagBlendImagePremultipliedAlpha = value;
        information->FlagCorrectShaderPremultipliedAlpha = value;
    }

    std::tie(valueText, node) = getValueText("settings/vertexAnimeFloat");
    if (!valueText.empty()) {
        information->FlagVertexAnimeFloat = node.text().as_bool();
    }

    /*
    //add for 6.4 ------------------------------------------------
    std::tie(valueText, node) = getValueText("settings/signal");
    if (!valueText.empty()) {
        information->Signal
    }
    */

    std::tie(valueText, node) = getValueText("settings/coordUnit");
    if (!valueText.empty()) {
        if (valueText == "rate") {
            // TODO:
        } else if (valueText == "pix") {
            // TODO:
        }
    }

    return information;
}

