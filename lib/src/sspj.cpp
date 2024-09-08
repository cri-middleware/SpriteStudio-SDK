#include <filesystem>

#include "SSConverter3Lib/sspj.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"

using namespace SpriteStudio;

struct SSPJ::Impl {
    void parseSettings(std::shared_ptr<SSPJInformation> &information, const std::filesystem::path &sspj_path, const pugi::xml_node& nodeRoot);
    void parseValues(std::shared_ptr<SSPJInformation> &information, const pugi::xml_node& nodeRoot);
};

std::string SSPJInformation::NameDirectory() const {
    return util::getNameDirectory(sspj);
}

std::string SSPJInformation::NameFileBody() const {
    return util::getNameFileBody(sspj);
}

std::string SSPJInformation::NameFileExtension() const {
    return util::getNameFileExtension(sspj);
}

int SSPJInformation::addTexture(const std::string &nameTexture) {
    int index;
    auto result = std::find(ListNameTexture.begin(), ListNameTexture.end(), nameTexture);
    if (result != ListNameTexture.end()) {
        ListNameTexture.push_back(nameTexture);
        index = static_cast<int>(ListNameTexture.size() - 1);
    } else {
        index = static_cast<int>(std::distance(ListNameTexture.begin(), result));
    }

    return index;
}

SSPJ::SSPJ() : pImpl(std::make_unique<Impl>()) {

}
SSPJ::~SSPJ() = default;

std::shared_ptr<SSPJInformation> SSPJ::parse(const std::string &sspj) {
    auto sspj_path = std::filesystem::path(sspj);
    if (!std::filesystem::exists(sspj_path)) {
        throw SSException("not found a ssae file: " + sspj, -1);
    }

    pugi::xml_document doc;
    auto result = doc.load_file(sspj.c_str());
    if (!result) {
        throw SSException("failed loading a sspj file", -1);
    }

    auto nodeRoot = doc.first_child();
    if (std::string(nodeRoot.name()) != "SpriteStudioProject") {
        throw SSException("invalid sspj format", -1);
    }

    auto sspjInfo = std::make_shared<SSPJInformation>();
    if (sspjInfo == nullptr) {
        throw SSException("not enough memory", -1);
    }
    sspjInfo->sspj = sspj;

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw SSException("invalid sspj format", -1);
    }
    auto versionText = std::string(attr.as_string());
    sspjInfo->version = static_cast<SSPJVersion>(util::getVersion(versionText, SSPJVersionMap));
    if (sspjInfo->version == SSPJVersion::ERROR) {
        throw SSException("Unknown Version: " + versionText, -1);
    }

    // settings
    pImpl->parseSettings(sspjInfo, sspj_path, nodeRoot.child("settings"));

    // cellmap
    pImpl->parseValues(sspjInfo, nodeRoot);

    return sspjInfo;
}

void SSPJ::Impl::parseSettings(std::shared_ptr<SSPJInformation> &information, const std::filesystem::path &sspj_path, const pugi::xml_node& nodeRoot) {
    auto getValueText = [&nodeRoot](const std::string &name) -> std::tuple<std::string, pugi::xml_node> {
        pugi::xml_node node = nodeRoot.child(name.c_str());
        auto text = std::string(node.text().as_string());
        return std::make_tuple(text, node);
    };
    std::string valueText;
    pugi::xml_node node;
    std::tie(valueText, node) = getValueText("imageBaseDirectory");
    information->NameDirectoryBaseTexture = valueText.empty() ? information->NameDirectory() : (sspj_path / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("cellMapBaseDirectory");
    information->NameDirectoryBaseSSCE = valueText.empty() ? information->NameDirectory() : (sspj_path / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("animeBaseDirectory");
    information->NameDirectoryBaseSSAE = valueText.empty() ? information->NameDirectory() : (sspj_path / std::filesystem::path(valueText)).string();

    std::tie(valueText, node) = getValueText("effectBaseDirectory");
    information->NameDirectoryBaseSSEE = valueText.empty() ? information->NameDirectory() : (sspj_path / std::filesystem::path(valueText)).string();

    // std::tie(valueText, node) = getValueText("sequenceBaseDirectory");
    valueText = "";
    information->NameDirectoryBaseSSQE = valueText.empty() ? information->NameDirectory() : (sspj_path / std::filesystem::path(valueText)).string();

    // std::tie(valueText, node) = getValueText("audiopackBaseDirectory");
    valueText = "";
    information->NameDirectoryBaseSSSE = valueText.empty() ? information->NameDirectory() : (sspj_path / std::filesystem::path(valueText)).string();

    // std::tie(valueText, node) = getValueText("charmapBaseDirectory");
    valueText = "";
    information->NameDirectoryBaseFnt = valueText.empty() ? information->NameDirectory() : (sspj_path / std::filesystem::path(valueText)).string();

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

void SSPJ::Impl::parseValues(std::shared_ptr<SSPJInformation> &information, const pugi::xml_node& nodeRoot) {
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

    // get Sound-List FileNames
    xpath_node_set = getList("audiopackNames/value");
    if(xpath_node_set.empty()) {
        information->TableInformationSSSE = {};
    } else {
        for (const auto& xpath_node : xpath_node_set) {
            auto value = std::string(xpath_node.node().text().as_string());
            auto path = std::filesystem::path(information->NameDirectoryBaseSSSE) / std::filesystem::path(value);
            information->TableNameSSSE.push_back(path.string());
        }
    }


    // get External-Texture List FileNames
    // TODO: Impl
}
