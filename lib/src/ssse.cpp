#include <filesystem>

#include "SSException.h"
#include "SSConverter3Lib/ssse.h"
#include "pugixml/pugixml.hpp"
#include "util.h"

using namespace SpriteStudio;

std::string SSSEInformation::NameDirectory() {
    return std::filesystem::path(ssse).parent_path().string();
}

std::string SSSEInformation::NameFileBody() {
    return std::filesystem::path(ssse).filename().stem().string();
}

std::string SSSEInformation::NameFileExtension() {
    return std::filesystem::path(ssse).filename().extension().string().substr(1);
}

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
    if (std::string(nodeRoot.name()) != "SpriteStudioSoundList") {
        throw SSException("invalid ssse format", -1);
    }

    auto ssseInfo = std::make_shared<SSSEInformation>();
    if (ssseInfo == nullptr) {
        throw SSException("not enough memory", -1);
    }

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw SSException("invalid sspj format", -1);
    }
    auto versionText = std::string(attr.as_string());

    std::string valueText;
    pugi::xml_node node;

    auto getList = [&nodeRoot](const std::string &query)  {
        return nodeRoot.select_nodes(query.c_str());
    };

    auto getValueText = [&nodeRoot](const std::string &query) -> std::tuple<std::string, pugi::xml_node> {
        std::string text;
        pugi::xml_node node;
        if (auto xpath_node = nodeRoot.select_node(query.c_str())) {
            if (auto value = xpath_node.node().text().as_string()) {
                text = std::string(value);
                node = xpath_node.node();
            }
        }
        return std::make_tuple(text, node);
    };

    std::tie(valueText, node) = getValueText("table/id");
    ssseInfo->ID = (valueText.empty() ? -1 : std::stoi(valueText));

    std::tie(valueText, node) = getValueText("table/Alias");
    ssseInfo->Name = valueText;

    auto xpath_node_set = getList("table/sounds/value");
    std::vector<std::shared_ptr<ss::ssfb2::LibraryDataSoundT>> tableSound = {};
    if (xpath_node_set.empty()) {
        throw SSException("not found cell nodes", -1);
    } else {
        for(const auto& xpath_node : xpath_node_set) {
            node = xpath_node.node();
            auto data = std::make_shared<ss::ssfb2::LibraryDataSoundT>();

            valueText = std::string(node.child("Alias").text().as_string());
            data->name = valueText;

            valueText = std::string(node.child("filename").text().as_string());
            data->path = valueText;

            auto valueLong = node.child("msec").text().as_llong();
            data->time_total = valueLong * (1.0f / 1000.0f); // msec -> sec
            tableSound.push_back(data);
        }
    }
    ssseInfo->TableSound = tableSound;

    return ssseInfo;
}
