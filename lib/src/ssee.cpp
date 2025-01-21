#include <filesystem>
#include <sstream>

#include "SSConverter3Lib/ssee.h"

#include "error.h"
#include "pugixml/pugixml.hpp"
#include "SSException.h"
#include "util.h"
#include "SSConverter3Lib/sspj.h"
#include "util/XorShift32.h"

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

void SSEEInformation::InUseCellMapAdd(uint16_t indexCellMap, const std::shared_ptr<SSEEEmitterInformation> &informationEmitter) {
    auto value = std::make_shared<SSEEInUseCellMap>();
    value->IndexCellMap = indexCellMap;
    value->Blend = static_cast<ss::ssfb2::KindOperationBlendEffect>(informationEmitter->Data->operation_blend_target);

    auto result = std::find(this->ListInUseCellMap.begin(), this->ListInUseCellMap.end(), value);
    if (result == this->ListInUseCellMap.end()) {
        ListInUseCellMap.push_back(value);
    }
}

struct SSEE::Impl {
    std::tuple<int, std::shared_ptr<SSEEPartsInformation>> parseParts(const std::shared_ptr<SSEEInformation>& sseeInfo, const std::vector<std::shared_ptr<SSEEPartsInformation>>& tableParts, const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node& node);
    std::shared_ptr<SSEEEmitterInformation> parsePartsEmitter(const pugi::xml_node& node, const std::shared_ptr<SSPJInformation> &sspjInfo, const std::shared_ptr<SSEEInformation> &sseeInfo,
                                                              const std::shared_ptr<SSEEPartsInformation> &sseePartsInformation, ss::ssfb2::KindOperationBlendEffect operationBlendTarget, const std::string &cellMap, const std::string &cell, int index_parts);
    std::tuple<float, float> parsePartsEmitterRangeGetFloat( const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node& node, const std::string& name, int indexParts, const std::string& nameFileSSEE);
    std::tuple<std::shared_ptr<ss::ssfb2::ColorT>, std::shared_ptr<ss::ssfb2::ColorT>> parsePartsEmitterRangeGetColor(const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node &node, const std::string &name,int indexParts, const std::string &nameFileSSEE);

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
    sseeInfo->ScaleLayout.x = effectDataNode.child("layoutScaleX").text().as_float() / 100.0f;
    sseeInfo->ScaleLayout.y = effectDataNode.child("layoutScaleY").text().as_float() / 100.0f;

    auto xpath_node_set = effectDataNode.select_nodes("nodeList/node");
    if (xpath_node_set.empty()) {
        throw error::getException(PHASE, DOC_PART,"not found node list: " + file, -1);
    }
    std::vector<std::shared_ptr<SSEEPartsInformation>> tableParts(xpath_node_set.size());
    for(const auto& xpath_node : xpath_node_set) {
        node = xpath_node.node();
        int indexParts;
        std::shared_ptr<SSEEPartsInformation> parts;
        std::tie(indexParts, parts) = pImpl->parseParts(sseeInfo, tableParts, sspjInfo, node);
        tableParts[indexParts] = parts;
    }
    sseeInfo->TableParts = tableParts;

    // TODO:impl

    return sseeInfo;
}

bool SSEE::DataConvertCalculateInAdvance(const std::shared_ptr<SpriteStudio::SSPJInformation> &sspjInfo, const std::shared_ptr<SpriteStudio::SSEEInformation> &sseeInfo) {
    // MEMO: Use "Calculate In Advance" to distinguish from "Precalculate".
    auto countEmitter = sseeInfo->TableEmitterSS6PU.size();
    auto flagLockSeed = false;
    uint seedRandom = 0;

    // MEMO: Provisionally, create directly. (Because I don't want to implement "Script_SpriteStudio6_RootEffect")
    auto random = std::make_shared<XorShift32>();
    std::vector<std::shared_ptr<ss::ssfb2::PatternEmitT>> tablePatternEmit;
    std::vector<int64_t> tableSeedParticle;
    for (int i=0; i<countEmitter; i++) {
        // Fixed Random-Seed
        flagLockSeed = false;
        if ( 0 != (static_cast<uint32_t>(sseeInfo->TableEmitterSS6PU[i]->flag_data) & static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::SEED_RANDOM)) ) {
            // Seed Overwrite
            seedRandom = (uint)sseeInfo->TableEmitterSS6PU[i]->seed_random + 7573; // TODO: change magic number to a constant variable
            flagLockSeed = true;
        } else {
            if(true == sseeInfo->FlagLockSeed) {
                // Seed Locked
                // MEMO: Overwritten to the Effect's Seed.
                seedRandom = ((uint)sseeInfo->SeedRandom + 1) * (uint)7573;  // TODO: change magic number to a constant variable
                flagLockSeed = true;
            }
        }

        // Calcurate Table-Offset-Pattern
        {
            int countEmitMax = sseeInfo->TableEmitterSS6PU[i]->count_particle_max;
            int countEmit = sseeInfo->TableEmitterSS6PU[i]->count_particle_emit;
            countEmit = (1 > countEmit) ? 1 : countEmit;

            // Create Offset-Pattern Table
            // MEMO: This Table will be solved at Importing.
            int shot = 0;
            int offset = sseeInfo->TableEmitterSS6PU[i]->delay;
            int count = countEmitMax;
            sseeInfo->TableEmitterSS6PU[i]->table_pattern_offset = std::vector<int>(count);
            for(int j=0; j<count; j++) {
                if(shot >= countEmit) {
                    shot = 0;
                    offset += sseeInfo->TableEmitterSS6PU[i]->interval;
                }
                sseeInfo->TableEmitterSS6PU[i]->table_pattern_offset[j] = offset;
                shot++;
            }
        }

        // Set to Data
        if (true == flagLockSeed) {
            // Fixed Data
            // Calcurate Table-Datas

            int count;
            int countEmitMax = sseeInfo->TableEmitterSS6PU[i]->count_particle_max;

            auto listPatternEmit = std::vector<ss::ssfb2::PatternEmitT>();
            int countEmit = sseeInfo->TableEmitterSS6PU[i]->count_particle_emit;
            countEmit = (1 > countEmit) ? 1 : countEmit;

            // Create Emit-Pattern Table
            // MEMO: This Table will be solved at Importing (at seedRandom is fixed).
            random->InitSeed(seedRandom);
            int cycle = (int)(((float)(countEmitMax * sseeInfo->TableEmitterSS6PU[i]->interval) / (float)countEmit) + 0.5f);
            count = countEmitMax * 8; // TODO: use const variable
            if(64 > count) { // TODO: use const variable
                count = 64;
            }

            tablePatternEmit.resize(count);
            int duration;
            for(int j=0; j<count; j++) {
                tablePatternEmit[j] = std::make_shared<ss::ssfb2::PatternEmitT>();
                tablePatternEmit[j]->index_generate = j;

                duration = (int)(sseeInfo->TableEmitterSS6PU[i]->duration_particle->main + random->RandomFloat((sseeInfo->TableEmitterSS6PU[i]->duration_particle->sub)));
                tablePatternEmit[j]->duration = duration;
                tablePatternEmit[j]->cycle = (duration > cycle) ? duration : cycle;
            }

            // Create Random-Seed Table
            // MEMO: This Table will be solved at Importing (at seedRandom is fixed).
            count = countEmitMax * 3;
            tableSeedParticle.clear();
            tableSeedParticle.resize(count);
            random->InitSeed(seedRandom);
            for(int j=0; j<count; j++) {
                tableSeedParticle[j] = (long)(random->RandomUint32());
            }
            sseeInfo->TableEmitterSS6PU[i]->table_pattern_emit = tablePatternEmit;
            sseeInfo->TableEmitterSS6PU[i]->table_seed_particle = tableSeedParticle;
        } else {
            // Calculate on runtime
            sseeInfo->TableEmitterSS6PU[i]->table_pattern_emit.clear();
            sseeInfo->TableEmitterSS6PU[i]->table_seed_particle.clear();
        }
        tablePatternEmit.clear();
        tableSeedParticle.clear();
    }
    return true;
}


void SSEE::DataConverter(const ConvertOptions &options, const std::shared_ptr<SSPJInformation> &sspjInfo, const std::shared_ptr<SSEEInformation> &sseeInfo) {

    // Rebuild Parts
    // MEMO: Effect data has only parts "Root" and "Emitter".
    int countPartsRebuild = 0;
    int countEmitter = 0;
    auto tableIndexPartsRebuild = std::vector<int>();
    auto workAreaRebuildParts = std::vector<std::shared_ptr<WorkAreaRebuildPartsDataConvert>>();
    tableIndexPartsRebuild.resize(sseeInfo->TableParts.size());
    workAreaRebuildParts.resize(sseeInfo->TableParts.size());
    for (int i=0; i< sseeInfo->TableParts.size(); i++) {
        auto parts = sseeInfo->TableParts[i];
        switch (parts->Data->feature) {
            case static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::ROOT): {
                auto item = std::make_shared<WorkAreaRebuildPartsDataConvert>();
                item->IndexParts = i;
                item->IndexPartsChildParicle = -1;
                item->IndexEmitter = -1;	// Has no "Emitter"
                workAreaRebuildParts[countPartsRebuild] = item;

                tableIndexPartsRebuild[i] = countPartsRebuild;
                countPartsRebuild++;
            }
                break;

            case static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::EMITTER): {
                // MEMO: "Emitter" always has only 1 child(definitely "Particle").
                auto item = std::make_shared<WorkAreaRebuildPartsDataConvert>();
                item->IndexParts = i;
                item->IndexPartsChildParicle = sseeInfo->TableParts[i]->Data->table_id_child[0];
                item->IndexEmitter = countEmitter;
                workAreaRebuildParts[countPartsRebuild] = item;
                countEmitter++;

                tableIndexPartsRebuild[i] = countPartsRebuild;
                countPartsRebuild++;
            }
                break;

            case static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::PARTICLE): {
                // MEMO: Parent(Emitter) has already been set to "tableIndexPartsRebuild".
                // MEMO: "Particle" always has parent.
                tableIndexPartsRebuild[i] = tableIndexPartsRebuild[sseeInfo->TableParts[i]->Data->id_parent];
            }
            break;
        }

        std::vector<std::shared_ptr<SSEEPartsInformation>> tablePartsNew;
        tablePartsNew.resize(countPartsRebuild);
        int indexPartsTemp;
        int countPartsChild;

        for(int i=0; i<countPartsRebuild; i++)
        {
            auto partsNew = std::make_shared<SSEEPartsInformation>();
            auto partsOriginal = sseeInfo->TableParts[workAreaRebuildParts[i]->IndexParts];

            partsNew->Data = std::make_shared<ss::ssfb2::DataEffectPartsT>();
            partsNew->Data->name = partsOriginal->Data->name;
            partsNew->Data->id = i;

            indexPartsTemp = partsOriginal->Data->id_parent;
            partsNew->Data->id_parent = (-1 == indexPartsTemp) ? -1 : tableIndexPartsRebuild[indexPartsTemp];
            partsNew->Data->feature = partsOriginal->Data->feature;
            partsNew->Data->index_emitter = workAreaRebuildParts[i]->IndexEmitter;

            partsNew->DataEmitter = partsOriginal->DataEmitter;
            indexPartsTemp = workAreaRebuildParts[i]->IndexPartsChildParicle; //Sub-emitters are "Particle"'s children.
            indexPartsTemp = (-1 == indexPartsTemp) ? workAreaRebuildParts[i]->IndexParts : indexPartsTemp;
            countPartsChild = sseeInfo->TableParts[indexPartsTemp]->Data->table_id_child.size();
            partsNew->Data->table_id_child.resize(countPartsChild);
            for(int j=0; j<countPartsChild; j++) {
                partsNew->Data->table_id_child[j] = tableIndexPartsRebuild[sseeInfo->TableParts[indexPartsTemp]->Data->table_id_child[j]];
            }

            partsNew->DataEmitter = partsOriginal->DataEmitter;

            tablePartsNew[i] = partsNew;
        }
        sseeInfo->TableParts = tablePartsNew;

        // Tidy up & Create Parts for Runtime
        auto tableDataParts = std::vector<std::shared_ptr<ss::ssfb2::DataEffectPartsT>>();
        tableDataParts.resize(countPartsRebuild);
        for(int i=0; i<countPartsRebuild; i++)
        {
            tableDataParts[i] = std::make_shared<ss::ssfb2::DataEffectPartsT>();
            tableDataParts[i] = tablePartsNew[i]->Data;
        }
        sseeInfo->TablePartsSS6PU = tableDataParts;

        // Create Pattern-Offset & Emt-Pattern
        if(!DataConvertCalculateInAdvance(sspjInfo, sseeInfo)) {
            // TODO: warning
            // LogError(messageLogPrefix, "Failure to Generate PatternEmit Datas", informationSSEE.FileNameGetFullPath(), informationSSPJ);
            // goto DataConvert_ErroeEnd;
        }

        // TODO: impl

        // create Draw-Order

    }

    // TODO: impl
}

void SSEE::DataOutput(const ConvertOptions &options, const std::shared_ptr<SSPJInformation> &sspjInfo, const std::shared_ptr<SSEEInformation> &item) {
    // TODO: impl
}

std::tuple<int, std::shared_ptr<SSEEPartsInformation>> SSEE::Impl::parseParts(const std::shared_ptr<SSEEInformation>& sseeInfo, const std::vector<std::shared_ptr<SSEEPartsInformation>>& tableParts, const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node& node) {
    std::string valueText;

    auto sseePartsInformation = std::make_shared<SSEEPartsInformation>();
    sseePartsInformation->Data = std::make_shared<ss::ssfb2::DataEffectPartsT>();
    // get Part-ID
    auto indexParts = node.child("arrayIndex").text().as_int();

    sseePartsInformation->Data->id = indexParts;
    sseePartsInformation->Data->name = std::string(node.child("name").text().as_string());
    auto id_parent = node.child("parentIndex").text().as_int();
    if (id_parent >= 0) {
        std::lock_guard<std::mutex> lock(m);
        const auto& partsParent = tableParts[sseePartsInformation->Data->id_parent];
        partsParent->ListIndexPartsChild.push_back(sseePartsInformation->Data->id);
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
        if (sseePartsInformation->Data->name.empty()) {
            sseePartsInformation->Data->name = "Root";
        }
        sseePartsInformation->Data->feature = static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::ROOT);
        sseePartsInformation->Data->index_emitter = -1; // null TODO: impl
    } else if (valueText == "Emmiter") {
        if (sseePartsInformation->Data->name.empty()) {
            sseePartsInformation->Data->name = "Emitter";
        }
        sseePartsInformation->Data->feature = static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::EMITTER);
        sseePartsInformation->DataEmitter = parsePartsEmitter(node, sspjInfo, sseeInfo,
                                                              sseePartsInformation,
                                                              operationBlendTarget,
                                                              nameCellMap,
                                                              nameCell, indexParts);
        if (sseePartsInformation->DataEmitter == nullptr) {
            throw SSException("not enough memory: ", -1);
        }
    } else if (valueText == "Particle") {
        if (sseePartsInformation->Data->name.empty()) {
            sseePartsInformation->Data->name = "Particle";
        }
        sseePartsInformation->Data->feature = static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::PARTICLE);
        sseePartsInformation->Data->index_emitter = -1; // null TODO: impl
    } else {
        if (sseePartsInformation->Data->name.empty()) {
            sseePartsInformation->Data->name = "Error";
        }
        sseePartsInformation->Data->feature = static_cast<uint8_t>(ss::ssfb2::KindFeaturePartsDataEffect::TERMINATOR); // Error
        sseePartsInformation->Data->index_emitter = -1; // null TODO: impl
    }

    return std::make_tuple(indexParts, sseePartsInformation);
}


std::shared_ptr<SSEEEmitterInformation> SSEE::Impl::parsePartsEmitter(const pugi::xml_node& node, const std::shared_ptr<SSPJInformation> &sspjInfo, const std::shared_ptr<SSEEInformation> &sseeInfo,
                                                                      const std::shared_ptr<SSEEPartsInformation> &sseePartsInformation, ss::ssfb2::KindOperationBlendEffect operationBlendTarget, const std::string &cellMap, const std::string &cell, int index_parts) {

    auto emitter = std::make_shared<SSEEEmitterInformation>();
    if (emitter == nullptr) throw error::getException(PHASE, DOC_PART, "not enough memory", -1);

    emitter->Data = std::make_shared<ss::ssfb2::DataEffectEmitterT>();
    emitter->Data->flag_data = static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::CLEAR);
    emitter->Data->operation_blend_target = static_cast<uint32_t>(operationBlendTarget);
    emitter->NameCellMap = cellMap;
    emitter->NameCell = cell;

    std::string valueText;
    if (cellMap.empty()) {
        valueText = std::filesystem::path(sspjInfo->NameDirectoryBaseSSCE / std::filesystem::path(cellMap));
        emitter->Data->index_cell_map = sspjInfo->indexGetFileName(sspjInfo->TableNameSSCE, valueText);

        // add to In-use-Cell-Map information
        if(0 <= emitter->Data->index_cell_map)
        {
            sseeInfo->InUseCellMapAdd(emitter->Data->index_cell_map, emitter);
        }

    }
    emitter->Data->index_cell = -1;

    auto nodeEmitter = node.child("behavior");
    auto xpath_node_set = nodeEmitter.select_nodes("list/node");
    std::vector<std::shared_ptr<SSEEPartsInformation>> tableParts(xpath_node_set.size());
    for(const auto& xpath_node : xpath_node_set) {
        // get Attribute-Classification
        auto nodeAttribute = xpath_node.node();
        valueText = nodeAttribute.attribute("name").as_string();

        if (valueText == "Basic") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::BASIC);

            emitter->PriorityParticle = nodeAttribute.child("priority").text().as_float();
            emitter->Data->count_particle_max = nodeAttribute.child("maximumParticle").text().as_int();
            emitter->Data->count_particle_emit = nodeAttribute.child("attimeCreate").text().as_int();
            emitter->Data->interval = nodeAttribute.child("interval").text().as_float();

            emitter->Data->speed = std::make_shared<ss::ssfb2::RangeFloatT>();
            std::tie(emitter->Data->speed->main, emitter->Data->speed->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "speed", index_parts, ""); // TODO: nameFileSSEE
            emitter->Data->duration_particle = std::make_shared<ss::ssfb2::RangeFloatT>();
            std::tie(emitter->Data->duration_particle->main, emitter->Data->duration_particle->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "lifespan", index_parts, ""); // TODO: nameFileSSEE

            emitter->Data->angle = std::make_shared<ss::ssfb2::RangeFloatT>();
            emitter->Data->angle->main = nodeAttribute.child("angle").text().as_float();
            emitter->Data->angle->sub = nodeAttribute.child("angleVariance").text().as_float();

        } else if (valueText == "OverWriteSeed") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::SEED_RANDOM);
            emitter->Data->seed_random = nodeAttribute.child("Seed").text().as_int();
        } else if (valueText == "Delay") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::DELAY);
            emitter->Data->delay = nodeAttribute.child("DelayTime").text().as_float();
        } else if (valueText == "Gravity") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::GRAVITY_DIRECTION);
            emitter->Data->gravity_directional = std::make_shared<ss::ssfb2::Vector2T>();

            valueText = std::string(nodeAttribute.child("Gravity").text().as_string());
            auto iss = std::istringstream(valueText);
            std::string x, y;
            if (!(std::getline(iss, x, ' ') && std::getline(iss, y))) {
                // TODO: warning
            } else {
                emitter->Data->gravity_directional->x = std::stof(x);
                emitter->Data->gravity_directional->y = std::stof(y);
            }

        } else if (valueText == "init_position") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::POSITION);

            emitter->Data->position = std::make_shared<ss::ssfb2::RangeVector2T>();
            emitter->Data->position->main = std::make_shared<ss::ssfb2::Vector2T>();
            emitter->Data->position->sub = std::make_shared<ss::ssfb2::Vector2T>();

            std::tie(emitter->Data->position->main->x, emitter->Data->position->sub->x) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "OffsetX", index_parts, ""); // TODO: nameFileSSEE
            std::tie(emitter->Data->position->main->y, emitter->Data->position->sub->y) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "OffsetY", index_parts, ""); // TODO: nameFileSSEE
        } else if (valueText == "trans_speed") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::SPEED_FLUCTUATION);

            emitter->Data->speed_fluctuation = std::make_shared<ss::ssfb2::RangeFloatT>();
            std::tie(emitter->Data->speed_fluctuation->main, emitter->Data->speed_fluctuation->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "Speed", index_parts, ""); // TODO: nameFileSSEE
        } else if (valueText == "init_rotation") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::ROTATION);

            emitter->Data->rotation = std::make_shared<ss::ssfb2::RangeFloatT>();
            emitter->Data->rotation_fluctuation = std::make_shared<ss::ssfb2::RangeFloatT>();

            std::tie(emitter->Data->rotation->main, emitter->Data->rotation->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "Rotation", index_parts, ""); // TODO: nameFileSSEE
            std::tie(emitter->Data->rotation_fluctuation->main, emitter->Data->rotation_fluctuation->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "RotationAdd", index_parts, ""); // TODO: nameFileSSEE
        } else if (valueText == "trans_rotation") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::ROTATION_FLUCTUATION);

            emitter->Data->rotation_fluctuation_rate = nodeAttribute.child("RotationFactor").text().as_float();
            emitter->Data->rotation_fluctuation_rate_time = nodeAttribute.child("EndLifeTimePer").text().as_float() * 0.01f;
        } else if (valueText == "add_tangentiala") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::TANGENTIAL_ACCELATION);

            emitter->Data->rate_tangential_acceleration = std::make_shared<ss::ssfb2::RangeFloatT>();
            std::tie(emitter->Data->rate_tangential_acceleration->main, emitter->Data->rate_tangential_acceleration->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "Acceleration", index_parts, ""); // TODO: nameFileSSEE
        } else if (valueText == "add_pointgravity") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::GRAVITY_POINT);
            emitter->Data->gravity_point_position = std::make_shared<ss::ssfb2::Vector2T>();

            valueText = std::string(nodeAttribute.child("Position").text().as_string());
            auto iss = std::istringstream(valueText);

            std::string x, y;
            if (!(std::getline(iss, x, ' ') && std::getline(iss, y))) {
                // TODO: warning
            } else {
                emitter->Data->gravity_point_position->x = std::stof(x);
                emitter->Data->gravity_point_position->y = std::stof(y);
            }

            emitter->Data->gravity_point_power = nodeAttribute.child("Power").text().as_float();
        } else if (valueText == "init_vertexcolor") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::COLOR_VERTEX);

            emitter->Data->color_vertex = std::make_shared<ss::ssfb2::RangeColorT>();
            std::tie(emitter->Data->color_vertex->main, emitter->Data->color_vertex->sub) = parsePartsEmitterRangeGetColor(sspjInfo, nodeAttribute, "Color", index_parts, ""); // TODO: nameFileSSEE
        } else if (valueText == "trans_vertexcolor") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::COLOR_VERTEX_FLUCTUATION);

            emitter->Data->color_vertex_fluctuation = std::make_shared<ss::ssfb2::RangeColorT>();
        } else if (valueText == "init_size") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::SCALE_START);

            emitter->Data->scale_start = std::make_shared<ss::ssfb2::RangeVector2T>();
            emitter->Data->scale_rate_start = std::make_shared<ss::ssfb2::RangeFloatT>();

            std::tie(emitter->Data->scale_start->main->x, emitter->Data->scale_start->sub->x) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "SizeX", index_parts, "");
            std::tie(emitter->Data->scale_start->main->y, emitter->Data->scale_start->sub->y) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "SizeY", index_parts, "");
            std::tie(emitter->Data->scale_rate_start->main, emitter->Data->scale_rate_start->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "ScaleFactor", index_parts, "");
        } else if (valueText == "trans_size") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::SCALE_END);

            emitter->Data->scale_end = std::make_shared<ss::ssfb2::RangeVector2T>();
            emitter->Data->scale_rate_end = std::make_shared<ss::ssfb2::RangeFloatT>();

            std::tie(emitter->Data->scale_end->main->x, emitter->Data->scale_end->sub->x) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "SizeX", index_parts, "");
            std::tie(emitter->Data->scale_end->main->y, emitter->Data->scale_end->sub->y) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "SizeY", index_parts, "");
            std::tie(emitter->Data->scale_rate_end->main, emitter->Data->scale_rate_end->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "ScaleFactor", index_parts, "");
        } else if (valueText == "trans_colorfade") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::FADE_ALPHA);
            auto rangeTemp = std::make_shared<ss::ssfb2::RangeFloatT>();

            std::tie(rangeTemp->main, rangeTemp->sub) = parsePartsEmitterRangeGetFloat(sspjInfo, nodeAttribute, "disprange", index_parts, "");

            emitter->Data->alpha_fade_start = rangeTemp->main * 0.01f;
            emitter->Data->alpha_fade_end = (rangeTemp->main + rangeTemp->sub) * 0.01f;
        } else if (valueText == "TurnToDirection") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::TURN_DIRECTION);

            emitter->Data->turn_direction_fluctuation = nodeAttribute.child("Rotation").text().as_float();
        } else if (valueText == "InfiniteEmit") {
            emitter->Data->flag_data |= static_cast<uint32_t>(ss::ssfb2::FlagBitEmitterDataEffect::EMIT_INFINITE);
        } else {
            // do nothing
        }
    }

    return emitter;
}

std::tuple<float, float> SSEE::Impl::parsePartsEmitterRangeGetFloat(const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node &node, const std::string &name,int indexParts, const std::string &nameFileSSEE) {
    float main = 0.0f;
    float sub = 0.0f;
    std::string valueText;
    if (!node.child(name.c_str()).empty()) {
        auto nodeNow = node.child(name.c_str());

        main = nodeNow.attribute("value").as_float();
        sub = nodeNow.attribute("subvalue").as_float();

        if(main > sub)
        {
            float floatTemp = sub;
            sub = main;
            main = floatTemp;
        }
        sub -= main;
    }
    return std::make_tuple(main, sub);
}

std::tuple<std::shared_ptr<ss::ssfb2::ColorT>, std::shared_ptr<ss::ssfb2::ColorT>> SSEE::Impl::parsePartsEmitterRangeGetColor(const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node &node, const std::string &name,int indexParts, const std::string &nameFileSSEE) {
    auto main = std::make_shared<ss::ssfb2::ColorT>();
    auto sub = std::make_shared<ss::ssfb2::ColorT>();

    std::string valueText;
    if (!node.child(name.c_str()).empty()) {
        auto nodeNow = node.child(name.c_str());

        auto colorTemp = nodeNow.attribute("value").as_uint();
        main->a = ((float)((colorTemp >> 24) & 0xff)) / 255.0f;
        main->r = ((float)((colorTemp >> 16) & 0xff)) / 255.0f;
        main->g = ((float)((colorTemp >> 8) & 0xff)) / 255.0f;
        main->b = ((float)(colorTemp & 0xff)) / 255.0f;

        colorTemp = nodeNow.attribute("subvalue").as_uint();
        sub->a = ((float)((colorTemp >> 24) & 0xff)) / 255.0f;
        sub->r = ((float)((colorTemp >> 16) & 0xff)) / 255.0f;
        sub->g = ((float)((colorTemp >> 8) & 0xff)) / 255.0f;
        sub->b = ((float)(colorTemp & 0xff)) / 255.0f;

        float floatTemp;
        if(main->a > sub->a) {
            floatTemp = sub->a;
            sub->a = main->a;
            main->a = floatTemp;
        }
        if(main->r > sub->r) {
            floatTemp = sub->r;
            sub->r = main->r;
            main->r = floatTemp;
        }
        if(main->g > sub->g) {
            floatTemp = sub->g;
            sub->g = main->g;
            main->g = floatTemp;
        }
        if(main->b > sub->b) {
            floatTemp = sub->b;
            sub->b = main->b;
            main->b = floatTemp;
        }

        sub->a -= main->a;
        sub->r -= main->r;
        sub->g -= main->g;
        sub->b -= main->b;
    }

    return std::make_tuple(main, sub);
}

/*
void SSEE::Impl::DataConvertDrawOrderSortLayerParts(List<WorkAreaBuildDrawOrderDataConvert> listOutput,
                                                    WorkAreaBuildDrawOrderDataConvert[] tableInput,
                                                    Library_SpriteStudio6.Data.Parts.Effect[] tableParts,
                                                    int indexParts) {
    // MEMO: Need to add child emitters after parent emitters.
    int[] indexPartsChild = tableParts[indexParts].TableIDChild;
    int indexEmitter;

    for(int i=0; i<indexPartsChild.Length; i++)
    {
        indexEmitter = tableParts[indexPartsChild[i]].IndexEmitter;
        listOutput.Add(tableInput[indexEmitter]);
    }

    for(int i=0; i<indexPartsChild.Length; i++)
    {
        DataConvertDrawOrderSortLayerParts(listOutput, tableInput, tableParts, indexPartsChild[i]);
    }
}
*/
