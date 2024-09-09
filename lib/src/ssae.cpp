#include <filesystem>
#include <sstream>
#include <memory>
#include <tuple>

#include "SSException.h"
#include "SSConverter3Lib/ssae.h"

#include "util.h"
#include "pugixml/pugixml.hpp"
#include "SSConverter3Lib/sspj.h"
#include "error.h"

using namespace SpriteStudio;
const PhaseType PHASE = PhaseType::PARSE;
const DocPartType DOC_PART = DocPartType::SSAE;

struct SSAE::Impl {
    void parseSettings(const std::shared_ptr<SSAEInformation>& ssaeInfo, const pugi::xml_node& node);
    std::tuple<int, std::shared_ptr<ss::ssfb2::DataModelPartsT>> parsePartData(const std::vector<std::shared_ptr<ss::ssfb2::DataModelPartsT>>& tableParts, const pugi::xml_node& node);

    static ss::ssfb2::DataModelPartsKindFeature getPartType(const std::string& valueText);

    static std::tuple<ss::ssfb2::KindCollision, int> getCollisionType(const std::string& valueText, float Collider_SizeZ);
};

std::string SSAEInformation::NameDirectory() const {
    return util::getNameDirectory(ssae);
}

std::string SSAEInformation::NameFileBody() const {
    return util::getNameFileBody(ssae);
}

std::string SSAEInformation::NameFileExtension() const {
    return util::getNameFileExtension(ssae);
}

SSAE::SSAE() : pImpl(std::make_unique<Impl>()) {
}

SSAE::~SSAE() = default;

std::shared_ptr<SSAEInformation> SSAE::parse(const std::string &ssae, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    auto ssae_path = std::filesystem::path(ssae);
    if (!std::filesystem::exists(ssae_path)) {
        throw error::getException(PHASE, DOC_PART, "not found a file: " + ssae, -1);
    }

    pugi::xml_document doc;
    auto result = doc.load_file(ssae_path.c_str());
    if (!result) {
        throw SSException("failed loading a ssae file", -1);
    }

    auto nodeRoot = doc.first_child();
    if (std::string(nodeRoot.name()) != "SpriteStudioAnimePack") {
        throw SSException("invalid sspj format", -1);
    }

    auto ssaeInfo = std::make_shared<SSAEInformation>();
    if (ssaeInfo == nullptr) {
        throw SSException("not enough memory", -1);
    }
    ssaeInfo->ssae = ssae;

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw SSException("invalid ssce format", -1);
    }
    auto versionText = std::string(attr.as_string());
    ssaeInfo->version = static_cast<SSAEVersion>(util::getVersion(versionText, SSAEVersionMap));
    if (ssaeInfo->version == SSAEVersion::ERROR) {
        throw SSException("Unknown Version: " + versionText, -1);
    }


    std::string valueText;
    pugi::xml_node node;

    auto getList = [&nodeRoot](const std::string &query)  {
        return nodeRoot.select_nodes(query.c_str());
    };
    auto xpath_node_set = getList("Model/partList/value");
    if (xpath_node_set.empty()) {
        throw SSException("PartList-Node Not Found", -1);
    }

    auto tableParts = std::vector<std::shared_ptr<ss::ssfb2::DataModelPartsT>>(xpath_node_set.size());
    for(const auto& xpath_node : xpath_node_set) {
        int index;
        std::shared_ptr<ss::ssfb2::DataModelPartsT> partData;

        node = xpath_node.node();
        std::tie(index, partData) = pImpl->parsePartData(tableParts, node);
        tableParts[index] = partData;
    }
    ssaeInfo->TableParts = tableParts;

    pImpl->parseSettings(ssaeInfo, nodeRoot.child("settings"));
    return ssaeInfo;
}

void SSAE::Impl::parseSettings(const std::shared_ptr<SSAEInformation> &ssaeInfo, const pugi::xml_node &node) {
    int valueInt;
    std::string valueText;

    valueInt = node.child("fps").text().as_int();
    ssaeInfo->data.frame_per_second = valueInt;

    valueInt = node.child("frameCount").text().as_int();
    ssaeInfo->data.count_frame = valueInt;

    ss::ssfb2::KindModeSort sort;
    valueText = std::string(node.child("sortMode").text().as_string());
    if (valueText == "prio") {
        sort = ss::ssfb2::KindModeSort::Priority;
    } else if (valueText == "z") {
        sort = ss::ssfb2::KindModeSort::PositionZ;
    } else {
        sort = ss::ssfb2::KindModeSort::Priority;
    }
    ssaeInfo->data.mode_sort = sort;

    valueText = std::string(node.child("canvasSize").text().as_string());
    if (!valueText.empty()) {
        std::istringstream iss(valueText);
        std::string x,y;

        if (std::getline(iss, x, ' ') && std::getline(iss, y)) {
            ssaeInfo->data.size_canvas_x = std::stoi(x);
            ssaeInfo->data.size_canvas_y = std::stoi(y);
        }
    }

    valueInt = node.child("startFrame").text().as_int(0);
    ssaeInfo->data.frame_valid_start = valueInt;

    valueInt = node.child("endFrame").text().as_int(-1);
    ssaeInfo->data.frame_valid_end = valueInt;

    ssaeInfo->data.count_frame_valid = (ssaeInfo->data.frame_valid_end - ssaeInfo->data.frame_valid_start) + 1;

    valueInt = node.child("ik_depth").text().as_int(3);
    ssaeInfo->data.depth_ik = valueInt;
}

std::tuple<int, std::shared_ptr<ss::ssfb2::DataModelPartsT>> SSAE::Impl::parsePartData(const std::vector<std::shared_ptr<ss::ssfb2::DataModelPartsT>>& tableParts, const pugi::xml_node &node) {
    int valueInt;
    std::string valueText;

    int indexPart;
    auto partData = std::make_shared<ss::ssfb2::DataModelPartsT>();

    valueInt = node.child("arrayIndex").text().as_int(-1);
    if (valueInt < 0) {
        throw SSException("Invalid arrayIndex" + std::to_string(valueInt), -1);
    }
    indexPart = valueInt;

    valueText = node.child("name").text().as_string();
    if(valueText.empty()) {
        throw SSException("Invalid", -1);
    }
    partData->name = valueText;

    valueInt = node.child("parentIndex").text().as_int(-1);
    partData->id_parent = valueInt;
    if (partData->id_parent >= 0) {
        auto parentParts = tableParts[partData->id_parent];
        parentParts->table_id_children.push_back(indexPart);
    }

    // get Parts-Type
    valueText = std::string(node.child("type").text().as_string());
    partData->feature = this->getPartType(valueText);

    // get "Collision" Data
    valueText = std::string(node.child("boundsType").text().as_string());
    float Collider_SizeZ = 0.0f; // TODO: impl
    ss::ssfb2::KindCollision collision;
    auto sizeCollision = 0.0f;

    std::tie(collision, sizeCollision) = getCollisionType(valueText, Collider_SizeZ);
    partData->shape_collision = collision;
    partData->size_collision = sizeCollision;

    /* Get "Inheritance" Datas */
    // TODO: impl
    valueText = std::string(node.child("inheritType").text().as_string());
    ss::ssfb2::AnimationAttributeFlagBit status = ss::ssfb2::AnimationAttributeFlagBit::CLEAR;
    if (valueText == "self") {

    }
    partData->status = status;

    return std::make_tuple(indexPart, partData);
}

ss::ssfb2::DataModelPartsKindFeature SSAE::Impl::getPartType(const std::string &valueText) {
    ss::ssfb2::DataModelPartsKindFeature feature;

    if (valueText == "normal") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Normal;
    } else if (valueText == "instance") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Instance;
    } else if (valueText == "effect") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Effect;
    } else if (valueText == "mask") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Mask;
    } else if (valueText == "mesh") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Mesh;
    } else if (valueText == "joint") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Joint;
    } else if (valueText == "armature") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Bone;
    } else if (valueText == "movenode") {
        feature = ss::ssfb2::DataModelPartsKindFeature::MoveNode;
    } else if (valueText == "constraint") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Constraint;
    } else if (valueText == "bonepoint") {
        feature = ss::ssfb2::DataModelPartsKindFeature::BonePoint;
    } else if (valueText == "transformconstraint") /* 仮: Yuzu. */ {
        feature = ss::ssfb2::DataModelPartsKindFeature::Transform_Constraint;
    } else if (valueText == "camera") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Camera;
    } else if (valueText == "ssaudio") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Audio;
    } else if (valueText == "shape") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Shape;
    } else if (valueText == "text") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Text;
    } else if (valueText == "nines") {
        feature = ss::ssfb2::DataModelPartsKindFeature::Nine_Slice;
    } else {
        // Null or others
        feature = ss::ssfb2::DataModelPartsKindFeature::Null;
    }

    return feature;
}

std::tuple<ss::ssfb2::KindCollision, int> SSAE::Impl::getCollisionType(const std::string &valueText, float Collider_SizeZ) {
    ss::ssfb2::KindCollision collision;
    auto sizeCollision = 0.0f;

    if (valueText == "quad") {
        collision = ss::ssfb2::KindCollision::Square;
        sizeCollision = Collider_SizeZ;
    } else if (valueText == "aabb") {
        collision = ss::ssfb2::KindCollision::Aabb;
        sizeCollision = Collider_SizeZ;
    } else if (valueText == "circle") {
        collision = ss::ssfb2::KindCollision::Circle;
        sizeCollision = Collider_SizeZ;
    } else if (valueText == "circle_smin") {
        collision = ss::ssfb2::KindCollision::CircleScaleMinimum;
        sizeCollision = Collider_SizeZ;
    } else if (valueText == "circle_smax") {
        collision = ss::ssfb2::KindCollision::CircleScaleMaximum;
        sizeCollision = Collider_SizeZ;
    } else {
        // Non or others
        collision = ss::ssfb2::KindCollision::Non;
        sizeCollision = 0.0f;
    }

    return std::make_tuple(collision, sizeCollision);
}
