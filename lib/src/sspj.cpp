#include <filesystem>

#include "SSConverter3Lib/sspj.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"

using namespace SpriteStudio;

struct SSPJ::Impl {
    void parserSettings(std::shared_ptr<SSPJ::Information> &information, const std::filesystem::path &sspj, const pugi::xml_node& nodeRoot);
    void parserValues(std::shared_ptr<SSPJ::Information> &information, const pugi::xml_node& nodeRoot);
};

std::string SSPJ::Information::NameDirectory() {
    return std::filesystem::path(sspj_path).parent_path().string();
}

std::string SSPJ::Information::NameFileBody() {
    return std::filesystem::path(sspj_path).filename().stem().string();
}

std::string SSPJ::Information::NameFileExtension() {
    return std::filesystem::path(sspj_path).filename().extension().string();
}

SSPJ::SSPJ() : pImpl(std::make_unique<Impl>()) {

}
SSPJ::~SSPJ() = default;

std::shared_ptr<SSPJ::Information> SSPJ::parser(const std::string &sspj) {
    pugi::xml_document doc;
    auto result = doc.load_file(sspj.c_str());
    if (!result) {
        throw SSException("failed loading a sspj file", -1);
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

    auto information = std::make_shared<SSPJ::Information>();
    if (information == nullptr) {
        throw SSException("not enough memory", -1);
    }
    information->sspj_path = sspj;

    // settings
    pImpl->parserSettings(information, sspj, nodeRoot);


    // cellmap
    pImpl->parserValues(information, nodeRoot);

    return information;
}

void SSPJ::Impl::parserSettings(std::shared_ptr<SSPJ::Information> &information, const std::filesystem::path &sspj, const pugi::xml_node& nodeRoot) {
    auto n = nodeRoot.child("settings");
    auto getValueText = [&n](const std::string &query) -> std::tuple<std::string, pugi::xml_node> {
        std::string text;
        pugi::xml_node node;
        if (auto xpath_node = n.select_node(query.c_str())) {
            if (auto value = xpath_node.node().text().as_string()) {
                text = std::string(value);
                node = xpath_node.node();
            }
        }
        return std::make_tuple(text, node);
    };
    std::string valueText;
    pugi::xml_node node;
    std::tie(valueText, node) = getValueText("imageBaseDirectory");
    information->NameDirectoryBaseTexture = valueText.empty() ? information->NameDirectory() : (sspj / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("cellMapBaseDirectory");
    information->NameDirectoryBaseSSCE = valueText.empty() ? information->NameDirectory() : (sspj / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("animeBaseDirectory");
    information->NameDirectoryBaseSSAE = valueText.empty() ? information->NameDirectory() : (sspj / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("effectBaseDirectory");
    information->NameDirectoryBaseSSEE = valueText.empty() ? information->NameDirectory() : (sspj / std::filesystem::path(valueText)).string();

    information->NameDirectoryBaseSSQE = information->NameDirectory();

    information->NameDirectoryBaseSSSE = information->NameDirectory();

    std::tie(valueText, node) = getValueText("wrapMode");
    if (valueText == "repeat") {
        information->WrapTexture = KindWrap::Repeart;
    } else if (valueText == "mirror") {
        information->WrapTexture = KindWrap::Mirror;
    } else {
        information->WrapTexture = KindWrap::Clamp;
    }

    std::tie(valueText, node) = getValueText("filterMode");
    if (valueText == "nearlest") {
        information->FilterTexture = KindFilter::Nearest;
    } else if (valueText == "bilinear") {
        information->FilterTexture = KindFilter::Bilinear;
    } else {
        information->FilterTexture = KindFilter::Linear;
    }

    std::tie(valueText, node) = getValueText("convertImageToPMA");
    if (!valueText.empty()) {
        information->FlagConvertImagePremultipliedAlpha = node.text().as_int();
    }

    std::tie(valueText, node) = getValueText("blendImageAsPMA");
    if (!valueText.empty()) {
        auto value = node.text().as_int();
        information->FlagBlendImagePremultipliedAlpha = value;
        information->FlagCorrectShaderPremultipliedAlpha = value;
    }

    std::tie(valueText, node) = getValueText("vertexAnimeFloat");
    if (!valueText.empty()) {
        information->FlagVertexAnimeFloat = node.text().as_int();
    }

    //add for 6.4 ------------------------------------------------
    std::tie(valueText, node) = getValueText("signal");
    if (!valueText.empty()) {
        information->Signal = valueText;
    }

    std::tie(valueText, node) = getValueText("coordUnit");
    if (!valueText.empty()) {
        if (valueText == "rate") {
            information->CoordUnit = KindCoordUnit::Rate;
        } else if (valueText == "pix") {
            information->CoordUnit = KindCoordUnit::Pix;
        }
    }
}

void SSPJ::Impl::parserValues(std::shared_ptr<SSPJ::Information> &information, const pugi::xml_node& nodeRoot) {
    auto getList = [&nodeRoot](const std::string &query)  {
        return nodeRoot.select_nodes(query.c_str());
    };

    // get cell map names
    auto xpath_node_set = getList("cellmapNames/value");
    if (xpath_node_set.empty()) {
        information->TableNameSSCE = {};
    } else {
        for(const auto& xpath_node : xpath_node_set) {
            auto value = std::string(xpath_node.node().text().as_string());
            auto path = std::filesystem::path(information->NameDirectoryBaseSSCE) / std::filesystem::path(value);
            information->TableNameSSCE.push_back(path.string());
        }
    }
    information->tableInformationSSCE.reserve(information->TableNameSSCE.size());

    // get anime pack names
    xpath_node_set = getList("animepackNames/value");
    if (xpath_node_set.empty()) {
        information->TableNameSSAE = {};
    } else {
        for (const auto& xpath_node : xpath_node_set) {
            auto value = std::string(xpath_node.node().text().as_string());
            auto path = std::filesystem::path(information->NameDirectoryBaseSSAE) / std::filesystem::path(value);
            information->TableNameSSAE.push_back(path.string());
        }
    }
    information->TableInformationSSAE.reserve(information->TableNameSSAE.size());

    // get effect file names
    xpath_node_set = getList("effectFileNames/value");
    if(xpath_node_set.empty()) {
        information->TableNameSSEE = {};
    } else {
        for (const auto& xpath_node : xpath_node_set) {
            auto value = std::string(xpath_node.node().text().as_string());
            auto path = std::filesystem::path(information->NameDirectoryBaseSSEE) / std::filesystem::path(value);
            information->TableNameSSEE.push_back(path.string());
        }
    }
    information->TableInformationSSEE.reserve(information->TableNameSSEE.size());

    // get Sequence FileNames
    xpath_node_set = getList("sequencepackNames/value");
    if(xpath_node_set.empty()) {
        information->TableNameSSQE = {};
    } else {
        for (const auto& xpath_node : xpath_node_set) {
            auto value = std::string(xpath_node.node().text().as_string());
            auto path = std::filesystem::path(information->NameDirectoryBaseSSQE) / std::filesystem::path(value);
            information->TableNameSSQE.push_back(path.string());
        }
    }
    information->TableInformationSSQE.reserve(information->TableNameSSQE.size());

    // get Bitmap-Font FileNames
    xpath_node_set = getList("charmapNames/value");
    if(xpath_node_set.empty()) {
        information->TableNameFnt = {};
    } else {
        for (const auto& xpath_node : xpath_node_set) {
            auto value = std::string(xpath_node.node().text().as_string());
            auto path = std::filesystem::path(information->NameDirectoryBaseFnt) / std::filesystem::path(value);
            information->TableNameFnt.push_back(path.string());
        }
    }
    information->TableInformationFnt.reserve(information->TableNameFnt.size());

    // get Sound-List FileNames
    // TODO: impl
}
