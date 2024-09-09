#include <filesystem>

#include "SSConverter3Lib/ssqe.h"

#include "error.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"

using namespace SpriteStudio;
const PhaseType PHASE = PhaseType::PARSE;
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

std::shared_ptr<SSQEInformation> SSQE::parse(const std::string &ssqe, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    auto ssqe_path = std::filesystem::path(ssqe);
    if (!std::filesystem::exists(ssqe_path)) {
        throw error::getException(PHASE, DOC_PART, "not found a file: " + ssqe, -1);
    }

    pugi::xml_document doc;
    auto result = doc.load_file(ssqe_path.c_str());
    if (!result) {
        throw error::getException(PHASE, DOC_PART, "failed loading a file: " + ssqe, -1);
    }

    auto nodeRoot = doc.first_child();
    if (std::string(nodeRoot.name()) != "SpriteStudioSequencePack") {
        throw error::getException(PHASE, DOC_PART, "invalid format: " + ssqe, -1);
    }

    auto ssqeInfo = std::make_shared<SSQEInformation>();
    if (ssqeInfo == nullptr) {
        throw SSException("not enough memory", -1);
    }
    ssqeInfo->ssqe = ssqe;

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw error::getException(PHASE, DOC_PART, "invalid format: " + ssqe, -1);
    }
    auto versionText = std::string(attr.as_string());
    ssqeInfo->version = static_cast<SSQEVersion>(util::getVersion(versionText, SSQEVersionMap));
    if (ssqeInfo->version == SSQEVersion::ERROR) {
        throw error::getException(PHASE, DOC_PART, "Unknown Version: " + versionText + " file: " + ssqe, -1);
    }

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

    std::string valueText;
    pugi::xml_node node;

    std::tie(valueText, node) = getValueText("name");
    ssqeInfo->Name = valueText;
    std::tie(valueText, node) = getValueText("exportPath");
    ssqeInfo->ExportPath = valueText;

    auto xpath_node_set = nodeRoot.select_nodes("sequenceList/sequence");
    ssqeInfo->SequenceList = {};
    for (const auto& xpath_node : xpath_node_set) {
        auto dataSequence =std::make_shared<ss::ssfb2::DataSequenceT>();
        node = xpath_node.node();
        dataSequence->name = std::string(node.child("name").text().as_string());
        dataSequence->index = node.child("index").text().as_int();
        valueText = std::string(node.child("type").text().as_string());
        int type = -1;
        if (valueText == "LAST") {
            type = 0;
        } else if (valueText == "KEEP") {
            type = 1;
        } else if (valueText == "TOP") {
            type = 2;
        }
        dataSequence->type = type;

        auto list_node_set = node.select_nodes("list/value");
        dataSequence->valueList = {};
        for (const auto& list_xpath_node : list_node_set) {
            auto details = std::shared_ptr<ss::ssfb2::DataSequenceDetailsT>();
            auto list_node = list_xpath_node.node();
            details->refAnimePack = std::string(list_node.child("refAnimePack").text().as_string());
            details->refAnime = std::string(list_node.child("refAnime").text().as_string());
            details->repeatCount = list_node.child("repeatCount").text().as_int();

            // informationSSQE.InUseAnimationAdd(detail.RefAnimePack, detail.RefAnime); // TODO: impl
            dataSequence->valueList.push_back(details);
        }

        ssqeInfo->SequenceList.push_back(dataSequence);
    }

    return ssqeInfo;
}
