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
    void parseSettings(const std::shared_ptr<ss::ssfb2::LibraryDataAnimationT> &data, const pugi::xml_node &node);
    std::tuple<int, std::shared_ptr<SSAEPartsInformation>> parsePartData(const std::shared_ptr<SSAEInformation>& ssaeInfo, const std::vector<std::shared_ptr<SSAEPartsInformation>>& tableParts, const pugi::xml_node& node, const std::shared_ptr<SSPJInformation> &sspjInfo);

    void parseAnimationAttribute(const pugi::xml_node& xml_node, const std::shared_ptr<SSAEInformation>& ssaeInfo, const std::shared_ptr<ss::ssfb2::LibraryDataAnimationT>& dat, int index_parts, bool flagIsSetup, const std::string& ssae);

    std::tuple<bool, std::shared_ptr<ss::ssfb2::LibraryDataAnimationT>> parseAnimation(const std::shared_ptr<SSPJInformation>& sspjInfo, const pugi::xml_node& nodeRoot, const std::shared_ptr<SSAEInformation>& ssaeInfo, int index_animation, const std::string& ssae);

    int NameCheckReserved(const std::string& string);

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

int SSAEInformation::indexGetParts(const std::string &value_text) {
    int index = -1;
    auto result = std::find_if(TableParts.begin(), TableParts.end(),  [value_text](const std::shared_ptr<SSAEPartsInformation>& item) {
        return value_text == item->data->name;
    });
    if (result != TableParts.end()) {
        index = std::distance(TableParts.begin(), result);
    }
    return index;
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
    // Decode Parts-Data
    auto xpath_node_set = getList("Model/partList/value");
    if (xpath_node_set.empty()) {
        throw error::getException(PHASE, DOC_PART, "PartList-Node Not Found", -1);
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
    for(const auto& partData : ssaeInfo->TableParts) {
        partData->data->table_id_children = partData->ListIndexPartsChild;
        partData->ListIndexPartsChild.clear();

        // fix inheritance kind
        // MEMO: Parent-part is always fixed earlier.
        if (partData->Inheritance == SSAEPartsInformation::KindInheritance_SELF) {
            // do nothing
        } else {
            auto indexPartsParent = partData->data->id_parent;
            if (0 <= indexPartsParent) {
                partData->Inheritance = SSAEPartsInformation::KindInheritance_SELF;
                partData->FlagInheritance = ssaeInfo->TableParts[indexPartsParent]->FlagInheritance;
            }
        }
    }

    // decode Bone-List
    {
        xpath_node_set = getList("Model/boneList/item");
        auto indexBoneMin = std::numeric_limits<int32_t>::max();
        int countBoneList = xpath_node_set.size();
        int indexBone;
        for(const auto& xpath_node : xpath_node_set) {
            node = xpath_node.node();

            /* Scan Bone-List */
            indexBone = node.text().as_int();
            if(0 <= indexBone) {
                /* Valid index */
                if(countBoneList <= indexBone)
                {
                    countBoneList = indexBone + 1;
                }
                if(indexBoneMin > indexBone)
                {
                    indexBoneMin = indexBone;
                }
            }
        }
        ssaeInfo->ListBone.resize(countBoneList);

        // MEMO: Some Beta versions of SS6.3 be outputting an "1 origined" index.
        if (std::numeric_limits<int32_t>::max() <= indexBoneMin) {
            // All Invalid or no bone
            indexBoneMin = 0;
        }
        ssaeInfo->OffsetIndexBone = indexBoneMin;

        for(const auto& xpath_node : xpath_node_set) {
            node = xpath_node.node();

            // get Bone-List
            auto nameBoneParts = std::string(node.attribute("key").as_string());
            indexBone = node.text().as_int();

            if(0 <= indexBone)
            {
                ssaeInfo->ListBone[indexBone - indexBoneMin] = nameBoneParts;
            }
        }
    }

    // decode MeshBind List
    {
        xpath_node_set = getList("Model/meshList/value");

        for(const auto& xpath_node : xpath_node_set) {
            node = xpath_node.node();
            auto bindMesh = std::make_shared<ss::ssfb2::BindMeshT>();
            valueText = node.text().as_string();
            if (!valueText.empty()) {
                auto bindVertex = std::make_shared<ss::ssfb2::BindMeshVertexT>();

                // split string
                auto valueTextSplitVertex = std::vector<std::string>();
                auto temp = std::string();
                auto iss = std::istringstream(valueText);
                while (std::getline(iss, temp, ',')) {
                    valueTextSplitVertex.push_back(temp);
                }

                for (const auto& item : valueTextSplitVertex) {
                    if (item.empty()) continue;

                    auto valueTextSplitParameter = std::vector<std::string>();
                    temp = std::string();
                    iss = std::istringstream(item);
                    while (std::getline(iss, temp, ' ')) {
                        valueTextSplitParameter.push_back(temp);
                    }

                    // get Bone-count
                    // MEMO: No-effective bones(weight=0) are counted to VertexBlock's "n(number of Bone)" sometimes, so check real data size.
                    auto countBone = std::stoi(valueTextSplitParameter[0]);
                    auto countParameterBone = static_cast<int>(valueTextSplitParameter.size() - 1) / 4; // "- 1": Bone-Count / "/ 4": number of Parameter
                    if (countBone != countParameterBone) {
                        countBone = countParameterBone;
                    }
                    bindVertex->table_bone.resize(countBone);
                    for (int i = 0; i< countBone; i++) {
                        auto indexParameter = (i * 4) + 1;

                        auto bindmeshBoneT = std::make_shared<ss::ssfb2::BindMeshBoneT>();
                        bindmeshBoneT->index = std::stoi(valueTextSplitParameter[indexParameter + 0]) - ssaeInfo->OffsetIndexBone;
                        bindmeshBoneT->weight = std::stof(valueTextSplitParameter[indexParameter + 1]) * 0.01f;

                        auto coordinate_offset = std::make_shared<ss::ssfb2::Vector3T>();
                        coordinate_offset->x = std::stof(valueTextSplitParameter[indexParameter + 2]);
                        coordinate_offset->y = std::stof(valueTextSplitParameter[indexParameter + 3]);
                        coordinate_offset->z = 0.0f;
                        bindmeshBoneT->coordinate_offset = coordinate_offset;

                        bindVertex->table_bone[i] = bindmeshBoneT;
                    }
                }
            }
            ssaeInfo->ListBindMesh.push_back(bindMesh);
        }
    }

    // solve Referenced-CellMaps' index
    {
        xpath_node_set = getList("cellmapNames/value");

        for(const auto& xpath_node : xpath_node_set) {
            node = xpath_node.node();
            auto nameCellMap = std::string(node.text().as_string());
            nameCellMap = (std::filesystem::path(sspjInfo->NameDirectoryBaseSSCE) /  std::filesystem::path(nameCellMap)).string();

            auto value = sspjInfo->indexGetFileName(sspjInfo->TableNameSSCE, nameCellMap);
            ssaeInfo->TableIndexCellMap.push_back(value);
            if(-1 == value) {
                throw error::getException(PHASE, DOC_PART, "CellMap Not Found: " + file, -1);
            }
        }
    }

    int countAnimation;

    // animations (& Parts' Key-Frames) Get
    {
        xpath_node_set = getList("animeList/anime");
        if (xpath_node_set.empty()) {
            throw error::getException(PHASE, DOC_PART, "AnimationList-Node Not Found" + file, -1);
        }
        auto flagHasSetup = false;
        for (const auto& xpath_node : xpath_node_set) {

            // check "SetUp" animation
            /* MEMO: When judging "Setup"-animation, do not judge by name. (Be sure, use "isSetup" tag's value.)                      */
            /*       Especially, if SS5's SSAE including same name animation is migrated to SS6, "Setup"-animation's name is changed. */
            if (node.child("isSetup").text().as_int(-1) > 0) {
                flagHasSetup = true;
            }
        }
        countAnimation = xpath_node_set.size();
        if(true == flagHasSetup) {
            /* MEMO: If has "Setup", should not be countAnimation negative. */
            countAnimation--;
        }

        ssaeInfo->TableAnimation.resize(countAnimation);

        int indexAnimation = 0;
        auto informationAnimation = std::make_shared<ss::ssfb2::LibraryDataAnimationT>();
        for (const auto& xpath_node : xpath_node_set) {
            // Animation (& Parts' Key-Frames) Get
            flagHasSetup = false;	// recycling Is animation "Setup"-animation ?
            std::tie(flagHasSetup, informationAnimation) = pImpl->parseAnimation(sspjInfo, xpath_node.node(), ssaeInfo, indexAnimation, ssae);

            if(true == flagHasSetup) {
                // "Setup" animation
                ssaeInfo->AnimationSetup = informationAnimation;
            }else
            {	// Normal animation
                ssaeInfo->TableAnimation[indexAnimation] = informationAnimation;
                indexAnimation++;
            }
        }
    }

    // TODO: impl
    /*
    // set secondary parameters
    // MEMO: This process is unnecessary, since "Setup" animation has only attributes' initial values.
    for(int i=0; i<countAnimation; i++) {

        // solving Attributes
        informationSSAE.TableAnimation[i].AttributeSolve(std::ref setting, informationSSPJ, informationSSAE);


        // set Part-Status
        informationSSAE.TableAnimation[i].StatusSetParts(informationSSPJ, informationSSAE);

        // set Draw-Order
        informationSSAE.TableAnimation[i].DrawOrderCreate(informationSSPJ, informationSSAE);
    }
    */

    // TODO: impl

    // get Labels
    auto table_label = std::vector<std::shared_ptr<ss::ssfb2::LabelT>>();
    xpath_node_set = getList("labels/value");
    for(const auto& xpath_node : xpath_node_set) {
        node = xpath_node.node();
        valueText = std::string(node.child("name").text().as_string());
        if (valueText.find("_start") == std::string::npos &&
            valueText.find("_end") == std::string::npos) {
            auto label = std::make_shared<ss::ssfb2::LabelT>();
            label->name = valueText;
            label->frame = node.child("time").text().as_int();

            table_label.push_back(label);
            }
        else
        {
            // TODO: impl
            // LogWarning(messageLogPrefix, "Used reserved Label-Name \"" + valueText + "\" Animation[" + indexAnimation.ToString() + "]", nameFileSSAE, informationSSPJ);
        }
    }
    ssaeInfo->data.table_label = table_label;

    // get Key-Frames
    int countParts = ssaeInfo->TableParts.size();
    // TODO: impl

    return ssaeInfo;
}

void SSAE::DataConverter(const ConvertOptions &options, const std::shared_ptr<SSPJInformation> &sspjInfo, const std::shared_ptr<SSAEInformation> &item) {
    int countParts = item->TableParts.size();
    // TODO: impl
}

void SSAE::DataOutput(const ConvertOptions &options, const std::shared_ptr<SSPJInformation> &sspjInfo, const std::shared_ptr<SSAEInformation> &item) {
    // TODO: impl
}

void SSAE::Impl::parseSettings(const std::shared_ptr<ss::ssfb2::LibraryDataAnimationT> &data, const pugi::xml_node &node) {
    int valueInt;
    std::string valueText;

    valueInt = node.child("fps").text().as_int();
    data->frame_per_second = valueInt;

    valueInt = node.child("frameCount").text().as_int();
    data->count_frame = valueInt;

    ss::ssfb2::KindModeSort sort;
    valueText = std::string(node.child("sortMode").text().as_string());
    if (valueText == "prio") {
        sort = ss::ssfb2::KindModeSort::PRIORITY;
    } else if (valueText == "z") {
        sort = ss::ssfb2::KindModeSort::POSITIONZ;
    } else {
        sort = ss::ssfb2::KindModeSort::PRIORITY;
    }
    data->mode_sort = sort;

    valueText = std::string(node.child("canvasSize").text().as_string());
    if (!valueText.empty()) {
        std::istringstream iss(valueText);
        std::string x,y;

        if (std::getline(iss, x, ' ') && std::getline(iss, y)) {
            data->size_canvas_x = std::stoi(x);
            data->size_canvas_y = std::stoi(y);
        }
    }

    valueInt = node.child("startFrame").text().as_int(0);
    data->frame_valid_start = valueInt;

    valueInt = node.child("endFrame").text().as_int(-1);
    data->frame_valid_end = valueInt;

    data->count_frame_valid = (data->frame_valid_end - data->frame_valid_start) + 1;

    valueInt = node.child("ik_depth").text().as_int(3);
    data->depth_ik = valueInt;
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
            partData->NameUnderControl = valueText;

            valueText = std::string(node.child("refAnime").text().as_string());
            partData->NameAnimationUnderControl = valueText.empty() ? "" : valueText;
        }
    } else if (partData->data->feature == ss::ssfb2::KindFeatureParts::EFFECT) {
        valueText = std::string(node.child("refEffectName").text().as_string());
        if (!valueText.empty()) {
            partData->NameUnderControl = valueText;
            partData->NameAnimationUnderControl = "";
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

void SSAE::Impl::parseAnimationAttribute(const pugi::xml_node &nodeRoot, const std::shared_ptr<SSAEInformation> &ssaeInfo, const std::shared_ptr<ss::ssfb2::LibraryDataAnimationT> &dat, int index_parts, bool flagIsSetup, const std::string &ssae) {
    auto informationAnimationParts = dat->table_parts[index_parts];
    // Set Part's Status
    /* MEMO: Set here at least "Not Used" flag. When this function is called, key-data exists in this part. */
    /*       Other flags are set in "Information.Animation.StatusSetParts".                                 */
    informationAnimationParts->status_parts = ss::ssfb2::FlagBitStatusDataAnimationParts::NOT_USED;
    informationAnimationParts->status_parts = static_cast<ss::ssfb2::FlagBitStatusDataAnimationParts>( static_cast<uint32_t>(informationAnimationParts->status_parts) & ~static_cast<uint32_t>(ss::ssfb2::FlagBitStatusDataAnimationParts::NOT_USED));

    std::string valueText;
    pugi::xml_node node;
    auto getList = [&nodeRoot](const std::string &query)  {
        return nodeRoot.select_nodes(query.c_str());
    };

    // get KeyFrame List
    auto xpath_node_set = getList("attribute");
    for (const auto& node_set : xpath_node_set) {
        auto node = node_set.node();

        // get Attribute-Tag
        auto tagText = node.attribute("tag").as_string();

        // get Key-Data List
        auto listNodeKey = node.select_nodes("key");

        // get Key-Data
        // nodeInterpolation

        // TODO: impl
    }


    // TODO: impl
}

std::tuple<bool, std::shared_ptr<ss::ssfb2::LibraryDataAnimationT>> SSAE::Impl::parseAnimation(const std::shared_ptr<SSPJInformation> &sspjInfo, const pugi::xml_node &nodeRoot, const std::shared_ptr<SSAEInformation> &ssaeInfo, int index_animation, const std::string &ssae) {
    auto flagIsSetup = false;
    auto informationAnimation = std::make_shared<ss::ssfb2::LibraryDataAnimationT>();

    std::string valueText;
    pugi::xml_node node;
    auto getList = [&nodeRoot](const std::string &query)  {
        return nodeRoot.select_nodes(query.c_str());
    };

    // get Base Datas
    valueText = std::string(nodeRoot.child("name").text().as_string());
    informationAnimation->name = valueText;

    if (nodeRoot.child("isSetup").text().as_int(-1) > 0) {
        flagIsSetup = true;
    }

    // settings
    this->parseSettings(informationAnimation, nodeRoot.child("settings"));

    // get Labels
    auto table_label = std::vector<std::shared_ptr<ss::ssfb2::LabelT>>();
    auto xpath_node_set = getList("labels/value");
    for(const auto& xpath_node : xpath_node_set) {
        node = xpath_node.node();
        valueText = std::string(node.child("name").text().as_string());
        if (valueText.find("_start") == std::string::npos &&
            valueText.find("_end") == std::string::npos) {
            auto label = std::make_shared<ss::ssfb2::LabelT>();
            label->name = valueText;
            label->frame = node.child("time").text().as_int();

            table_label.push_back(label);
            }
        else
        {
            // TODO: impl
            // LogWarning(messageLogPrefix, "Used reserved Label-Name \"" + valueText + "\" Animation[" + indexAnimation.ToString() + "]", nameFileSSAE, informationSSPJ);
        }
    }
    informationAnimation->table_label = table_label;

    // get Key-Frames
    int countParts = ssaeInfo->TableParts.size();
    informationAnimation->table_parts.resize(countParts);
    // MEMO: All animation part information is created here. Because parts-animation that has no key-data are not recorded in SSAE.
    for (int i=0; i<countParts; i++) {
        informationAnimation->table_parts[i] = std::make_shared<ss::ssfb2::DataAnimationPartsT>();
    }
    xpath_node_set = getList("partAnimes/partAnime");
    for(const auto& xpath_node : xpath_node_set) {
        node = xpath_node.node();
        valueText = std::string(node.child("partName").text().as_string());
        auto indexParts = ssaeInfo->indexGetParts(valueText);
        if (indexParts == -1) {
            throw error::getException(PHASE, DOC_PART, "Part's Name Not Found " + valueText + " Animation[" + std::to_string(index_animation) + "]", -1);
        }

        auto attributesNode = node.child("attributes");
        this->parseAnimationAttribute(attributesNode, ssaeInfo, informationAnimation, indexParts, flagIsSetup, ssae);
    }

    return std::make_tuple(flagIsSetup, informationAnimation);
}

int SSAE::Impl::NameCheckReserved(const std::string &name) {
    // TODO: impl
    return -1;
    /*
    if(!name.empty()) {
        for (int i=0; i<(int)KindLabelReserved.TERMINATOR; i++) {
            if(name == TableNameLabelReserved[i]) {
                return(i);
            }
        }
    }
    return -1;
    */
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
            auto color = std::make_shared<ss::ssfb2::Color32T>();
            color->a = (argb >> 24) & 0xff;
            color->r = (argb >> 16) & 0xff;
            color->g = (argb >> 8) & 0xff;
            color->b = (argb & 0xff);
            colorLabel->color = color;
        }
    }

    return colorLabel;
}
