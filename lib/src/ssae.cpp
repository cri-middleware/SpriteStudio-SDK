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
    std::tuple<int, std::shared_ptr<SSAEPartsInformation>> parsePartData(const std::shared_ptr<SSAEInformation>& ssaeInfo, const std::vector<std::shared_ptr<SSAEPartsInformation>>& tableParts, const pugi::xml_node& node, const std::shared_ptr<SSPJInformation> &sspjInfo);

    static ss::ssfb2::KindFeatureParts getPartType(const std::string& valueText);

    static std::tuple<ss::ssfb2::KindCollision, int> getCollisionType(const std::string& valueText, float Collider_SizeZ);

    static ss::ssfb2::KindOperationBlend getAlphaBlendType(const std::string& valueText);

    static std::shared_ptr<ss::ssfb2::ColorLabelT>  getColorLabel(const std::string& valueText);
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

std::string SSAEInformation::NameFile() const {
    return util::getFileName(ssae);
}

SSAE::SSAE() : pImpl(std::make_unique<Impl>()) {
}

SSAE::~SSAE() = default;

std::shared_ptr<SSAEInformation> SSAE::parse(const std::string &ssae, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    auto file = ssae;
    auto ssae_path = std::filesystem::path(ssae);
    if (!std::filesystem::exists(ssae_path)) {
        throw error::getException(PHASE, DOC_PART, "not found a file: " + file, -1);
    }

    pugi::xml_document doc;
    auto result = doc.load_file(ssae_path.c_str());
    if (!result) {
        throw error::getException(PHASE, DOC_PART, "failed loading a file: " + file, -1);
    }

    auto nodeRoot = doc.first_child();
    if (std::string(nodeRoot.name()) != "SpriteStudioAnimePack") {
        throw error::getException(PHASE, DOC_PART, "invalid format: " + file, -1);
    }

    auto ssaeInfo = std::make_shared<SSAEInformation>();
    if (ssaeInfo == nullptr) {
        throw SSException("not enough memory", -1);
    }
    ssaeInfo->ssae = ssae;

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw error::getException(PHASE, DOC_PART, "invalid format: " + file, -1);
    }
    auto versionText = std::string(attr.as_string());
    ssaeInfo->version = static_cast<SSAEVersion>(util::getVersion(versionText, SSAEVersionMap));
    if (ssaeInfo->version == SSAEVersion::ERROR) {
        throw error::getException(PHASE, DOC_PART, "Unknown Version: " + versionText + " file: " + file, -1);
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

    auto tableParts = std::vector<std::shared_ptr<SSAEPartsInformation>>(xpath_node_set.size());
    for(const auto& xpath_node : xpath_node_set) {
        int index;
        std::shared_ptr<SSAEPartsInformation> partData;

        node = xpath_node.node();
        std::tie(index, partData) = pImpl->parsePartData(ssaeInfo, tableParts, node, sspjInfo);
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
        sort = ss::ssfb2::KindModeSort::PRIORITY;
    } else if (valueText == "z") {
        sort = ss::ssfb2::KindModeSort::POSITIONZ;
    } else {
        sort = ss::ssfb2::KindModeSort::PRIORITY;
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

std::tuple<int, std::shared_ptr<SSAEPartsInformation>> SSAE::Impl::parsePartData(const std::shared_ptr<SSAEInformation>& ssaeInfo, const std::vector<std::shared_ptr<SSAEPartsInformation>>& tableParts, const pugi::xml_node &node, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    int valueInt;
    std::string valueText;
    bool valueBool;

    int indexPart;
    auto partData = std::make_shared<SSAEPartsInformation>();
    partData->data = std::make_shared<ss::ssfb2::DataModelPartsT>();

    valueInt = node.child("arrayIndex").text().as_int(-1);
    if (valueInt < 0) {
        throw SSException("Invalid arrayIndex" + std::to_string(valueInt), -1);
    }
    indexPart = valueInt;

    valueText = node.child("name").text().as_string();
    if(valueText.empty()) {
        throw SSException("Invalid", -1);
    }
    partData->data->name = valueText;

    valueInt = node.child("parentIndex").text().as_int(-1);
    partData->data->id_parent = valueInt;
    if (partData->data->id_parent >= 0) {
        const auto& parentParts = tableParts[partData->data->id_parent];
        parentParts->data->table_id_children.push_back(indexPart);
    }

    // get Parts-Type
    valueText = std::string(node.child("type").text().as_string());
    partData->data->feature = getPartType(valueText);

    // get "Collision" Data
    valueText = std::string(node.child("boundsType").text().as_string());
    float Collider_SizeZ = 0.0f; // TODO: impl to get an external class
    ss::ssfb2::KindCollision collision;
    auto sizeCollision = 0.0f;

    std::tie(collision, sizeCollision) = getCollisionType(valueText, Collider_SizeZ);
    partData->data->shape_collision = collision;
    partData->data->size_collision = sizeCollision;

    // get "Inheritance" Data
    valueText = std::string(node.child("inheritType").text().as_string());
    if (valueText == "self") {
        partData->Inheritance = SSAEPartsInformation::KindInheritance_SELF;
        partData->FlagInheritance =
                static_cast<SSAEPartsInformation::FlagBitInheritance>(static_cast<uint32_t>(SSAEPartsInformation::FlagBitInheritance_PRESET) |
                                                                      static_cast<uint32_t>(SSAEPartsInformation::FlagBitInheritance_FLIP_X) |
                                                                      static_cast<uint32_t>(SSAEPartsInformation::FlagBitInheritance_FLIP_Y) |
                                                                      static_cast<uint32_t>(SSAEPartsInformation::FlagBitInheritance_SHOW_HIDE));
        auto ineheritRatesNode = node.child("ineheritRates");
        valueBool = ineheritRatesNode.child("ALPH").text().as_bool();
        if (!valueBool) {
            partData->FlagInheritance =
                    static_cast<SSAEPartsInformation::FlagBitInheritance>(
                        static_cast<uint32_t>(partData->FlagInheritance) & ~static_cast<uint32_t>(SSAEPartsInformation::FlagBitInheritance_OPACITY_RATE)
                    );
        }

        valueBool = ineheritRatesNode.child("FLPH").text().as_bool();
        if (!valueBool) {
            partData->FlagInheritance =
                    static_cast<SSAEPartsInformation::FlagBitInheritance>(
                        static_cast<uint32_t>(partData->FlagInheritance) & ~static_cast<uint32_t>(SSAEPartsInformation::FlagBitInheritance_FLIP_X)
                    );
        }

        valueBool = ineheritRatesNode.child("FLPV").text().as_bool();
        if (!valueBool) {
            partData->FlagInheritance =
                    static_cast<SSAEPartsInformation::FlagBitInheritance>(
                        static_cast<uint32_t>(partData->FlagInheritance) & ~static_cast<uint32_t>(SSAEPartsInformation::FlagBitInheritance_FLIP_Y)
                    );
        }

        valueBool = ineheritRatesNode.child("HIDE").text().as_bool();
        if (!valueBool) {
            partData->FlagInheritance =
                    static_cast<SSAEPartsInformation::FlagBitInheritance>(
                        static_cast<uint32_t>(partData->FlagInheritance) & ~static_cast<uint32_t>(SSAEPartsInformation::FlagBitInheritance_SHOW_HIDE)
                    );
        }
    } else {
        // (valueText == "parent") or else
        if (partData->data->id == 0) {
            partData->Inheritance = SSAEPartsInformation::KindInheritance_SELF;
            partData->FlagInheritance = SSAEPartsInformation::FlagBitInheritance_PRESET;
        } else {
            partData->Inheritance = SSAEPartsInformation::KindInheritance_PARENT;
            partData->FlagInheritance = SSAEPartsInformation::FlagBitInheritance_CLEAR;
        }
    }

    // get Target-Blending
    valueText = std::string(node.child("alphaBlendType").text().as_string());
    partData->data->operation_blend_target = getAlphaBlendType(valueText);

    // get Parts Show(Hide)
    valueBool = node.child("show").text().as_bool(false);
    partData->FlagHide = valueBool;

    // get Mask-Targeting
    valueBool = node.child("maskInfluence").text().as_bool(false);
    partData->FlagMasking = valueBool;

    // UnderControl Data Get
    if (partData->data->feature == ss::ssfb2::KindFeatureParts::INSTANCE) {
        valueText = std::string(node.child("refAnimePack").text().as_string());
        if (!valueText.empty()) {
            // partData->name_under_control = valueText;

            valueText = std::string(node.child("refAnime").text().as_string());
            partData->data->name_animation_under_control = valueText.empty() ? "" : valueText;
        }
    } else if (partData->data->feature == ss::ssfb2::KindFeatureParts::EFFECT) {
        valueText = std::string(node.child("refEffectName").text().as_string());
        if (!valueText.empty()) {
            // partData->name_under_control = valueText; // TODO: impl
            partData->data->name_animation_under_control = "";
        }
    }

    // get Color-Label
    valueText = std::string(node.child("colorLabel").text().as_string());
    partData->data->label_color = getColorLabel(valueText);

    // get Bone-Data
    partData->LengthBone = node.child("boneLength").text().as_float(0.0f);

    valueText = std::string(node.child("bonePosition").text().as_string());
    partData->PositionXBone = partData->PositionYBone = 0.0f;
    if (!valueText.empty()) {
        std::istringstream iss(valueText);
        std::string x,y;

        if (std::getline(iss, x, ' ') && std::getline(iss, y)) {
            partData->PositionXBone = std::stof(x);
            partData->PositionYBone = std::stof(y);
        }
    }

    partData->RotateZBone = node.child("boneRotation").text().as_float(0.0f);

    partData->FlagVerbose = node.child("verbose").text().as_bool(false);
    if (partData->FlagVerbose == false) {
        /* MEMO: "Verbose" is always "true" when parents are "true". */
        auto informationPartsParent = partData;
        bool verbose = false;
        while (informationPartsParent->data->id_parent >= 0) {
            informationPartsParent = tableParts[informationPartsParent->data->id_parent];
            if(true == informationPartsParent->FlagVerbose) {
                verbose = true;
                break;
            }
        }
        partData->FlagVerbose = verbose;
    }

    // clear Status
    // partData->data->status = 0; // TODO: impl

    // get "Text" data
    valueText = std::string(node.child("text").text().as_string(""));
    partData->data->text = valueText;

    valueBool = node.child("textBitmap").text().as_bool(false);
    if (valueBool) {
        // partData->data->status |= 0x00000010; // BITMAP TODO: impl
    }

    valueText = std::string(node.child("textFamily").text().as_string(""));
    partData->data->name_family_font_text = valueText;


    valueText = std::string(node.child("textCharMap").text().as_string());
    partData->data->index_bitmap_font_text = sspjInfo->indexGetFontBitmap(valueText);

    partData->data->size_font_text = node.child("textSize").text().as_float(0.0f);

    partData->data->space_text = node.child("textSize").text().as_float(0.0f);

    partData->data->size_x_text = node.child("textWidth").text().as_int(-1);

    partData->data->size_y_text = node.child("textHeight").text().as_int(-1);

    if (node.child("textSmooth").text().as_bool(false)) {
        // partData->data->status |= 0x00000020; // SMOOTH_TEXTURE TODO: impl
    }

    if (node.child("textMask").text().as_bool(false)) {
        // partData->data->status |= 0x00000001; // MASK TODO: impl
    }


    auto anchor = node.child("eAnchor").text().as_int(-1);
    if (anchor != -1) {
        switch (anchor) {
            case 0x00:	/*  0 : 0000-0000 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::LU;
            break;
            case 0x01:	/* 16 : 0000-0001 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::CU;
            break;
            case 0x02:	/* 32 : 0000-0010 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::RU;
            break;

            case 0x10:	/*  1 : 0001-0000 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::LC;
            break;
            case 0x11:	/* 17 : 0001-0001 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::CC;
            break;
            case 0x12:	/* 33 : 0001-0010 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::RC;
            break;

            case 0x20:	/*  2 : 0010-0000 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::LD;
            break;
            case 0x21:	/* 18 : 0010-0001 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::CD;
            break;
            case 0x22:	/* 34 : 0010-0010 */
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::RD;
            break;

            default:
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::LU;
            break;
        }
    } else {
        switch (partData->data->feature) {
            case ss::ssfb2::KindFeatureParts::TEXT:
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::LU;
            break;

            case ss::ssfb2::KindFeatureParts::ROOT:
            case ss::ssfb2::KindFeatureParts::NULL_PARTS:
            case ss::ssfb2::KindFeatureParts::NORMAL:
            case ss::ssfb2::KindFeatureParts::INSTANCE:
            case ss::ssfb2::KindFeatureParts::EFFECT:
            case ss::ssfb2::KindFeatureParts::MASK:
            case ss::ssfb2::KindFeatureParts::JOINT:
            case ss::ssfb2::KindFeatureParts::BONE:
            case ss::ssfb2::KindFeatureParts::MOVENODE:
            case ss::ssfb2::KindFeatureParts::CONSTRAINT:
            case ss::ssfb2::KindFeatureParts::BONEPOINT:
            case ss::ssfb2::KindFeatureParts::MESH:
            case ss::ssfb2::KindFeatureParts::TRANSFORM_CONSTRAINT:
            case ss::ssfb2::KindFeatureParts::CAMERA:
            case ss::ssfb2::KindFeatureParts::AUDIO:
            case ss::ssfb2::KindFeatureParts::SHAPE:
            case ss::ssfb2::KindFeatureParts::NINE_SLICE:
            default:
                partData->data->anchor_text = ss::ssfb2::KindAnchorText::NON;
            break;
        }
    }

    // get "Shape" data
    valueText = std::string(node.child("shapeType").text().as_string(""));
    partData->data->shape = ss::ssfb2::KindShape::NON;
    if (!valueText.empty()) {
        if (valueText == "rectangle") {
            partData->data->shape = ss::ssfb2::KindShape::RECTANGLE;
        }
    }

    // MEMO: "Mask" status is shared between "Shape" and "Text".
    if (node.child("shapeMask").text().as_bool(false)) {
        // partData->data->status |= 0x00000001; // MASK TODO: impl
    }

    partData->data->margin_l_nineslice = node.child("ninesMarginL").text().as_int(-1);
    partData->data->margin_r_nineslice = node.child("ninesMarginR").text().as_int(-1);
    partData->data->margin_u_nineslice = node.child("ninesMarginT").text().as_int(-1);
    partData->data->margin_d_nineslice = node.child("ninesMarginB").text().as_int(-1);

    if (node.child("ninesFillMode").text().as_bool(false)) {
        // partData->data->status |= 0x00000100; // MODE_TILE_NINE_SLICE TODO: impl
    }

    if (node.child("ninesMask").text().as_bool(false)) {
        // partData->data->status |= 0x00000001; // MASK TODO: impl
    }

    return std::make_tuple(indexPart, partData);
}

ss::ssfb2::KindFeatureParts SSAE::Impl::getPartType(const std::string &valueText) {
    ss::ssfb2::KindFeatureParts feature;

    if (valueText == "normal") {
        feature = ss::ssfb2::KindFeatureParts::NORMAL;
    } else if (valueText == "instance") {
        feature = ss::ssfb2::KindFeatureParts::INSTANCE;
    } else if (valueText == "effect") {
        feature = ss::ssfb2::KindFeatureParts::EFFECT;
    } else if (valueText == "mask") {
        feature = ss::ssfb2::KindFeatureParts::MASK;
    } else if (valueText == "mesh") {
        feature = ss::ssfb2::KindFeatureParts::MESH;
    } else if (valueText == "joint") {
        feature = ss::ssfb2::KindFeatureParts::JOINT;
    } else if (valueText == "armature") {
        feature = ss::ssfb2::KindFeatureParts::BONE;
    } else if (valueText == "movenode") {
        feature = ss::ssfb2::KindFeatureParts::MOVENODE;
    } else if (valueText == "constraint") {
        feature = ss::ssfb2::KindFeatureParts::CONSTRAINT;
    } else if (valueText == "bonepoint") {
        feature = ss::ssfb2::KindFeatureParts::BONEPOINT;
    } else if (valueText == "transformconstraint") /* 仮: Yuzu. */ {
        feature = ss::ssfb2::KindFeatureParts::TRANSFORM_CONSTRAINT;
    } else if (valueText == "camera") {
        feature = ss::ssfb2::KindFeatureParts::CAMERA;
    } else if (valueText == "ssaudio") {
        feature = ss::ssfb2::KindFeatureParts::AUDIO;
    } else if (valueText == "shape") {
        feature = ss::ssfb2::KindFeatureParts::SHAPE;
    } else if (valueText == "text") {
        feature = ss::ssfb2::KindFeatureParts::TEXT;
    } else if (valueText == "nines") {
        feature = ss::ssfb2::KindFeatureParts::NINE_SLICE;
    } else {
        // Null or others
        feature = ss::ssfb2::KindFeatureParts::NULL_PARTS;
    }

    return feature;
}

std::tuple<ss::ssfb2::KindCollision, int> SSAE::Impl::getCollisionType(const std::string &valueText, float Collider_SizeZ) {
    ss::ssfb2::KindCollision collision;
    auto sizeCollision = 0.0f;

    if (valueText == "quad") {
        collision = ss::ssfb2::KindCollision::SQUARE;
        sizeCollision = Collider_SizeZ;
    } else if (valueText == "aabb") {
        collision = ss::ssfb2::KindCollision::AABB;
        sizeCollision = Collider_SizeZ;
    } else if (valueText == "circle") {
        collision = ss::ssfb2::KindCollision::CIRCLE;
        sizeCollision = Collider_SizeZ;
    } else if (valueText == "circle_smin") {
        collision = ss::ssfb2::KindCollision::CIRCLE_SCALE_MINIMUM;
        sizeCollision = Collider_SizeZ;
    } else if (valueText == "circle_smax") {
        collision = ss::ssfb2::KindCollision::CIRCLE_SCALE_MAXIMUM;
        sizeCollision = Collider_SizeZ;
    } else {
        // Non or others
        collision = ss::ssfb2::KindCollision::NON;
        sizeCollision = 0.0f;
    }

    return std::make_tuple(collision, sizeCollision);
}

ss::ssfb2::KindOperationBlend SSAE::Impl::getAlphaBlendType(const std::string &valueText) {
    ss::ssfb2::KindOperationBlend operationBlend;

    if (valueText == "mul") {
        operationBlend = ss::ssfb2::KindOperationBlend::MULNA;
    } else if (valueText == "add") {
        operationBlend = ss::ssfb2::KindOperationBlend::ADD;
    } else if (valueText == "sub") {
        operationBlend = ss::ssfb2::KindOperationBlend::SUB;
    } else if (valueText == "mulalpha" || valueText == "mul2") {
        operationBlend = ss::ssfb2::KindOperationBlend::MUL;
    } else if (valueText == "screen" || valueText == "screen2") {
        operationBlend = ss::ssfb2::KindOperationBlend::SCR;
    } else if (valueText == "exclusion") {
        operationBlend = ss::ssfb2::KindOperationBlend::EXC;
    } else if (valueText == "invert") {
        operationBlend = ss::ssfb2::KindOperationBlend::INV;
    } else {
        // "mix", "div2", "overlay2", else
        operationBlend = ss::ssfb2::KindOperationBlend::MIX;
    }

    return operationBlend;
}

std::shared_ptr<ss::ssfb2::ColorLabelT> SSAE::Impl::getColorLabel(const std::string &valueText) {
    auto colorLabel = std::make_shared<ss::ssfb2::ColorLabelT>();

    if (valueText.empty()) {
        colorLabel->form = ss::ssfb2::KindFormColorLabel::NON;
    } else {
        if (valueText == "Red") {
            colorLabel->form = ss::ssfb2::KindFormColorLabel::RED;
        } else if (valueText == "Orange") {
            colorLabel->form = ss::ssfb2::KindFormColorLabel::ORANGE;
        } else if (valueText == "Yellow") {
            colorLabel->form = ss::ssfb2::KindFormColorLabel::YELLOW;
        } else if (valueText == "Green") {
            colorLabel->form = ss::ssfb2::KindFormColorLabel::GREEN;
        } else if (valueText == "Blue") {
            colorLabel->form = ss::ssfb2::KindFormColorLabel::BLUE;
        } else if (valueText == "Violet") {
            colorLabel->form = ss::ssfb2::KindFormColorLabel::VIOLET;
        } else if (valueText == "Gray") {
            colorLabel->form = ss::ssfb2::KindFormColorLabel::GRAY;
        } else {
            auto argb = std::stoul(valueText, 0, 16);
            colorLabel->color->a = (argb >> 24) & 0xff;
            colorLabel->color->r = (argb >> 16) & 0xff;
            colorLabel->color->g = (argb >> 8) & 0xff;
            colorLabel->color->b = (argb & 0xff);
        }
    }

    return colorLabel;
}
