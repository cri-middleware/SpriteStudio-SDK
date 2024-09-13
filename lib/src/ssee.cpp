#include <filesystem>

#include "SSConverter3Lib/ssee.h"

#include "error.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"

using namespace SpriteStudio;
const PhaseType PHASE = PhaseType::PARSE;
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

struct SSEE::Impl {

    std::tuple<int, std::shared_ptr<ss::ssfb2::DataEffectPartsT>> parseParts(const pugi::xml_node& node);
};

SSEE::SSEE(): pImpl(std::make_unique<Impl>()) {
}

SSEE::~SSEE() = default;

std::shared_ptr<SSEEInformation> SSEE::parse(const std::string &ssee, const std::shared_ptr<SSPJInformation> &sharedsspjInfo) {
    auto file = ssee;
    auto ssee_path = std::filesystem::path(ssee);
    if (!std::filesystem::exists(ssee_path)) {
        throw error::getException(PHASE, DOC_PART, "not found a file: " + file, -1);
    }

    pugi::xml_document doc;
    auto result = doc.load_file(ssee_path.c_str());
    if (!result) {
        throw error::getException(PHASE, DOC_PART, "failed loading a file: " + file, -1);
    }

    auto nodeRoot = doc.first_child();
    if (std::string(nodeRoot.name()) != "SpriteStudioEffect") {
        throw error::getException(PHASE, DOC_PART, "invalid format: " + file, -1);
    }

    auto sseeInfo = std::make_shared<SSEEInformation>();
    if (sseeInfo == nullptr) {
        throw SSException("not enough memory: " + file, -1);
    }
    sseeInfo->ssee = ssee;

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw error::getException(PHASE, DOC_PART, "invalid format: " + file, -1);
    }
    auto versionText = std::string(attr.as_string());
    sseeInfo->version = static_cast<SSEEVersion>(util::getVersion(versionText, SSEEVersionMap));
    if (sseeInfo->version == SSEEVersion::ERROR) {
        throw error::getException(PHASE, DOC_PART, "Unknown Version: " + versionText + " file: " + file, -1);
    }


    std::string valueText;
    pugi::xml_node node;

    auto effectDataNode = nodeRoot.child("effectData");
    sseeInfo->SeedRandom = effectDataNode.child("lockRandSeed").text().as_int();
    sseeInfo->FlagLockSeed = effectDataNode.child("effectDataNode.child").text().as_bool();
    sseeInfo->FramePerSecond = effectDataNode.child("fps").text().as_int();
    sseeInfo->VersionRenderer = effectDataNode.child("renderVersion").text().as_int();
    sseeInfo->ScaleLayout.x = effectDataNode.child("layoutScaleX").text().as_float() / 100.0f;
    sseeInfo->ScaleLayout.y = effectDataNode.child("layoutScaleY").text().as_float() / 100.0f;

    auto xpath_node_set = effectDataNode.select_nodes("nodeList/node");
    if (xpath_node_set.empty()) {
        throw error::getException(PHASE, DOC_PART,"not found node list: " + file, -1);
    }
    std::vector<std::shared_ptr<ss::ssfb2::DataEffectPartsT>> tableParts(xpath_node_set.size());
    for(const auto& xpath_node : xpath_node_set) {
        node = xpath_node.node();
        int indexParts;
        std::shared_ptr<ss::ssfb2::DataEffectPartsT> parts;
        std::tie(indexParts, parts) = pImpl->parseParts(node);
        tableParts[indexParts] = parts;
    }
    sseeInfo->TableParts = tableParts;

    // TODO:impl

    return sseeInfo;
}

std::tuple<int, std::shared_ptr<ss::ssfb2::DataEffectPartsT>> SSEE::Impl::parseParts(const pugi::xml_node &node) {
    std::string valueText;

    auto dataEffectParts = std::make_shared<ss::ssfb2::DataEffectPartsT>();
    // get Part-ID
    auto indexParts = node.child("arrayIndex").text().as_int();

    dataEffectParts->name = std::string(node.child("name").text().as_string());
    dataEffectParts->id_parent = node.child("parentIndex").text().as_int();
    // TODO: impl

    return std::make_tuple(indexParts, dataEffectParts);
}
