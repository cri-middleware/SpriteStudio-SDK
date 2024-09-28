#include <filesystem>

#include "SSConverter3Lib/ssee.h"

#include "error.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"
#include "SSConverter3Lib/sspj.h"

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

std::string SSEEInformation::NameFile() const {
    return util::getFileName(ssee);
}

struct SSEE::Impl {
    std::tuple<int, std::shared_ptr<ss::ssfb2::DataEffectPartsT>> parseParts(const std::shared_ptr<SSEEInformation>& sseeInfo, const std::vector<std::shared_ptr<ss::ssfb2::DataEffectPartsT>>& tableParts, const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node& node);
    void parsePartsEmitter(const std::shared_ptr<SSEEInformation>& sseeInfo, const pugi::xml_node& node);
    void parsePartsEmitter(pugi::xml_node node, const std::shared_ptr<SSEEInformation>& sseeInfo, const std::shared_ptr<ss::ssfb2::DataEffectPartsT>& dataEffectParts,
                           ss::ssfb2::KindOperationBlendEffect operationBlendTarget, const std::string& cellmap, const std::string& cell, int index_parts);

    std::mutex m;
};

SSEE::SSEE(): pImpl(std::make_unique<Impl>()) {
}

SSEE::~SSEE() = default;

std::shared_ptr<SSEEInformation> SSEE::parse(const std::string &ssee, const std::shared_ptr<SSPJInformation> &sspjInfo) {
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
    sseeInfo->ScaleLayout.mutate_x(effectDataNode.child("layoutScaleX").text().as_float() / 100.0f);
    sseeInfo->ScaleLayout.mutate_y(effectDataNode.child("layoutScaleY").text().as_float() / 100.0f);

    auto xpath_node_set = effectDataNode.select_nodes("nodeList/node");
    if (xpath_node_set.empty()) {
        throw error::getException(PHASE, DOC_PART,"not found node list: " + file, -1);
    }
    std::vector<std::shared_ptr<ss::ssfb2::DataEffectPartsT>> tableParts(xpath_node_set.size());
    for(const auto& xpath_node : xpath_node_set) {
        node = xpath_node.node();
        int indexParts;
        std::shared_ptr<ss::ssfb2::DataEffectPartsT> parts;
        std::tie(indexParts, parts) = pImpl->parseParts(sseeInfo, tableParts, sspjInfo, node);
        tableParts[indexParts] = parts;
    }
    sseeInfo->TableParts = tableParts;

    // TODO:impl

    return sseeInfo;
}

std::tuple<int, std::shared_ptr<ss::ssfb2::DataEffectPartsT>> SSEE::Impl::parseParts(const std::shared_ptr<SSEEInformation>& sseeInfo, const std::vector<std::shared_ptr<ss::ssfb2::DataEffectPartsT>>& tableParts, const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node& node) {
    std::string valueText;

    auto dataEffectParts = std::make_shared<ss::ssfb2::DataEffectPartsT>();
    // get Part-ID
    auto indexParts = node.child("arrayIndex").text().as_int();

    dataEffectParts->id = indexParts;
    dataEffectParts->name = std::string(node.child("name").text().as_string());
    auto id_parent = node.child("parentIndex").text().as_int();
    if (id_parent >= 0) {
        std::lock_guard<std::mutex> lock(m);
        const auto& partsParent = tableParts[dataEffectParts->id_parent];
        partsParent->table_id_child.push_back(indexParts);

        dataEffectParts->id_parent = id_parent;
    }

    auto operationBlendTarget = ss::ssfb2::KindOperationBlendEffect::NON;
    std::string nameCellMap;
    std::string nameCell;
    auto behaviorNode = node.child("behavior");
    if (behaviorNode) {
        valueText = behaviorNode.child("BlendType").text().as_string();
        if (valueText == "Mix") {
            operationBlendTarget = ss::ssfb2::KindOperationBlendEffect::MIX;
        } else if (valueText == "Add") {
            operationBlendTarget = ss::ssfb2::KindOperationBlendEffect::ADD;
        } else {
            operationBlendTarget = ss::ssfb2::KindOperationBlendEffect::MIX;
        }

        nameCellMap = std::string(behaviorNode.child("CellMapName").text().as_string());
        nameCell = std::string(behaviorNode.child("CellName").text().as_string());
    }

    valueText = std::string(node.child("type").text().as_string());
    if (valueText == "Root") {
        if (dataEffectParts->name.empty()) {
            dataEffectParts->name = "Root";
        }
        dataEffectParts->feature = static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::ROOT);
        dataEffectParts->index_emitter = -1; // null TODO: impl
    } else if (valueText == "Emmiter") {
        if (dataEffectParts->name.empty()) {
            dataEffectParts->name = "Emitter";
        }
        dataEffectParts->feature = static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::EMITTER);
        auto indexCellMap = sspjInfo->indexGetFileName(sspjInfo->TableNameSSCE, nameCellMap);
        // TODO: impl
        /*
        parsePartsEmitter(node, sseeInfo, dataEffectParts,
                          operationBlendTarget,
                          indexCellMap,
                          nameCell,
                          indexParts);
            */
    } else if (valueText == "Particle") {
        if (dataEffectParts->name.empty()) {
            dataEffectParts->name = "Particle";
        }
        dataEffectParts->feature = static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::PARTICLE);
        dataEffectParts->index_emitter = -1; // null TODO: impl
    } else {
        if (dataEffectParts->name.empty()) {
            dataEffectParts->name = "Error";
        }
        dataEffectParts->feature = static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::TERMINATOR); // Error
        dataEffectParts->index_emitter = -1; // null TODO: impl
    }

    return std::make_tuple(indexParts, dataEffectParts);
}

/*
void SSEE::Impl::parsePartsEmitter(pugi::xml_node node, const std::shared_ptr<SSEEInformation> &sseeInfo, const std::shared_ptr<ss::ssfb2::DataEffectPartsT> &dataEffectParts,
    ss::ssfb2::KindOperationBlendEffect operationBlendTarget, int indexCellMap,
    const std::string &cell, int index_parts) {

    auto dataEffectEmitter = std::make_shared<ss::ssfb2::DataEffectEmitterT>();
    if (dataEffectEmitter == nullptr) throw error::getException(PHASE, DOC_PART, "not enough memory", -1);

    dataEffectEmitter->flag_data = static_cast<uint32_t>(ss::ssfb2::DataEffectEmitterFlagBit::Clear);
    dataEffectEmitter->operation_blend_target = static_cast<uint32_t>(operationBlendTarget);
    dataEffectEmitter->index_cell_map = indexCellMap;
    dataEffectEmitter->index_cell = -1;
}
*/
