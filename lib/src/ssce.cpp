#include <filesystem>
#include <sstream>
#include <memory>

#include "SSException.h"
#include "SSConverter3Lib/ssce.h"

#include "error.h"
#include "util.h"
#include "pugixml/pugixml.hpp"
#include "SSConverter3Lib/sspj.h"

using namespace SpriteStudio;
const PhaseType PHASE = PhaseType::PARSE;
const DocPartType DOC_PART = DocPartType::SSCE;

struct SSCE::Impl {
    std::shared_ptr<ss::ssfb2::CellT> parseCell(const pugi::xml_node& node);
    std::vector<std::shared_ptr<ss::ssfb2::Vector2T>> parseMeshCoordinate(const pugi::xpath_node_set& xpath_nodes);
    std::vector<uint16_t> parseMeshIndices(const pugi::xpath_node_set& xpath_nodes);
};

std::string SSCEInformation::NameDirectory() const {
    return util::getNameDirectory(file);
}

std::string SSCEInformation::NameFileBody() const {
    return util::getNameFileBody(file);
}

std::string SSCEInformation::NameFileExtension() const {
    return util::getNameFileExtension(file);
}

std::string SSCEInformation::NameFile() const {
    return util::getFileName(file);
}

SSCE::SSCE() : pImpl(std::make_unique<SSCE::Impl>()) {
}
SSCE::~SSCE() = default;

std::shared_ptr<SSCEInformation> SSCE::parse(const std::string &ssce, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    auto ssce_path = std::filesystem::path(ssce);
    if (!std::filesystem::exists(ssce_path)) {
        throw error::getException(PHASE, DOC_PART, "not found a file: " + ssce, -1);
    }

    pugi::xml_document doc;
    auto result = doc.load_file(ssce_path.c_str());
    if (!result) {
        throw error::getException(PHASE, DOC_PART, "failed loading a file: " + ssce, -1);
    }

    auto nodeRoot = doc.first_child();
    if (std::string(nodeRoot.name()) != "SpriteStudioCellMap") {
        throw error::getException(PHASE, DOC_PART, "invalid format: " + ssce, -1);
    }

    auto ssceInfo = std::make_shared<SSCEInformation>();
    if (ssceInfo == nullptr) {
        throw SSException("not enough memory", -1);
    }
    ssceInfo->file = ssce;

    auto attr = nodeRoot.attribute("version");
    if (attr.empty()) {
        throw error::getException(PHASE, DOC_PART, "invalid format: " + ssce, -1);
    }
    auto versionText = std::string(attr.as_string());
    ssceInfo->version = static_cast<SSCEVersion>(util::getVersion(versionText, SSCEVersionMap));
    if (ssceInfo->version == SSCEVersion::ERROR) {
        throw error::getException(PHASE, DOC_PART, "Unknown Version: " + versionText + " file: " + ssce, -1);
    }


    std::string valueText;
    pugi::xml_node node;

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

    // get Texture Path-Names
    std::string namePathTexture;
    std::tie(valueText, node) = getValueText("imagePath");
    if (!valueText.empty()) {
        if (std::filesystem::path(valueText).is_absolute()) {
            namePathTexture = valueText;
        } else {
            namePathTexture = (std::filesystem::path(sspjInfo->NameDirectoryBaseTexture) / std::filesystem::path(valueText)).string();
        }
    }
    ssceInfo->IndexTexture = sspjInfo->addTexture(namePathTexture);

    // get Texture Pixel-Size
    std::tie(valueText, node) = getValueText("pixelSize");
    if (valueText.empty()) {
        // get directly from texture
        ssceInfo->SizePixelX = -1;
        ssceInfo->SizePixelY = -1;
    } else {
        // Synchronized with Cell size
        std::istringstream iss(valueText);

        std::string x, y;
        if (std::getline(iss, x, ' ') && std::getline(iss, y)) {
            // Data Valid
            ssceInfo->SizePixelX = std::stoi(x);
            ssceInfo->SizePixelY = std::stoi(y);
        } else {
            // Data Broken ?
            ssceInfo->SizePixelX = -1;
            ssceInfo->SizePixelY = -1;
        }
    }

    // get Texture Premultified-Alpha
    ssceInfo->FlagConvertImagePremultipliedAlpha = sspjInfo->FlagConvertImagePremultipliedAlpha;
    ssceInfo->FlagBlendImagePremultipliedAlpha = sspjInfo->FlagBlendImagePremultipliedAlpha;
    ssceInfo->FlagCorrectShaderPremultipliedAlpha = sspjInfo->FlagCorrectShaderPremultipliedAlpha;

    // get Texture Addressing
    ssceInfo->WrapTexture = sspjInfo->WrapTexture;
    ssceInfo->FilterTexture = sspjInfo->FilterTexture;

    std::tie(valueText, node) = getValueText("overrideTexSettings");
    if (!valueText.empty()) {
        auto valueBool = node.text().as_bool();
        if (valueBool) {
            std::tie(valueText, node) = getValueText("wrapMode");
            if (valueText == "repeat") {
                ssceInfo->WrapTexture = KindWrap::Repeart;
            } else if (valueText == "mirror") {
                ssceInfo->WrapTexture = KindWrap::Mirror;
            } else {
                ssceInfo->WrapTexture = KindWrap::Clamp;
            }

            std::tie(valueText, node) = getValueText("filterMode");
            if (valueText == "nearlest") {
                ssceInfo->FilterTexture = KindFilter::Nearest;
            } else if (valueText == "bilinear") {
                ssceInfo->FilterTexture = KindFilter::Bilinear;
            } else {
                ssceInfo->FilterTexture = KindFilter::Linear;
            }

        }
    }

    auto getList = [&nodeRoot](const std::string &query)  {
        return nodeRoot.select_nodes(query.c_str());
    };

    // get Cells
    auto xpath_node_set = getList("cells/cell");
    if (xpath_node_set.empty()) {
        throw error::getException(PHASE, DOC_PART,"not found cell nodes: " + ssce, -1);
    }
    std::vector<std::shared_ptr<ss::ssfb2::CellT>> listCell;
    for(const auto& xpath_node : xpath_node_set) {
        node = xpath_node.node();
        auto cell = pImpl->parseCell(node);
        listCell.push_back(cell);
    }
    ssceInfo->TableCell = listCell;

    return ssceInfo;
}

void SSCE::DataConverter(const ConvertOptions &options, const std::shared_ptr<SSPJInformation> &sspjInfo, const std::shared_ptr<SSCEInformation> &item) {
    auto flagDataIsSplit = options.split;

    if(0 <= item->IndexTexture) {
        auto informationTexture = sspjInfo->TableInformationTexture[item->IndexTexture];
        int countCell = item->TableCell.size();

        auto data = std::make_shared<ss::ssfb2::LibraryDataCellMapT>();
        data->name = item->NameFileBody();
        auto size_original = std::make_shared<ss::ssfb2::Vector2T>();
        size_original->x = (float)informationTexture->SizeX;
        size_original->y = (float)informationTexture->SizeY;
        data->size_original = size_original;
        data->table_cell = std::vector<std::shared_ptr<ss::ssfb2::CellT>>(countCell);
        data->filter = static_cast<int16_t>(item->FilterTexture);
        data->wrap = static_cast<int16_t>(item->WrapTexture);
        data->status = ss::ssfb2::LibraryDataCellMapFlagBitStatus::CLEAR;
        if(true == item->FlagConvertImagePremultipliedAlpha) {
            data->status = static_cast<ss::ssfb2::LibraryDataCellMapFlagBitStatus>(static_cast<uint16_t>(data->status) | static_cast<uint16_t>(ss::ssfb2::LibraryDataCellMapFlagBitStatus::TEXTURE_CONVERT_PMA));
        }
        if(true == item->FlagBlendImagePremultipliedAlpha) {
            data->status = static_cast<ss::ssfb2::LibraryDataCellMapFlagBitStatus>(static_cast<uint16_t>(data->status) | static_cast<uint16_t>(ss::ssfb2::LibraryDataCellMapFlagBitStatus::BLEND_IMAGE_PMA));
        }
        if(true == item->FlagCorrectShaderPremultipliedAlpha) {
            data->status = static_cast<ss::ssfb2::LibraryDataCellMapFlagBitStatus>(static_cast<uint16_t>(data->status) | static_cast<uint16_t>(ss::ssfb2::LibraryDataCellMapFlagBitStatus::CORRECT_SHADER_PMA));
        }
        item->Data = data;

        for(int i=0; i<countCell; i++) {
            data->table_cell[i] = item->TableCell[i];
        }
        item->TableCell.clear();
    }
}

void SSCE::DataOutputCellMap(const ConvertOptions &options, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    /* MEMO: Even when outputting Split-Data,  "Cell-Map" data in Script_SpriteStudio6_DataCellMap is referenced    */
    /*         starting from "Script_SpriteStudio6_DataCellMap.CellMap".                                            */
    /*       For this reason, "Cell-Map" data is generated for each part after the state of Merged-Data is created. */

    // generate Merged-Data
    auto dataCellMap = std::make_shared<ss::ssfb2::DataCellMapT>();

    // sspjInfo->dataCellMapT->table_cell_map[0] =  DataCellMapSS6PU.TableData[0] = dataCellMap;

    // TODO: impl version
    int countSSCE = sspjInfo->TableInformationSSCE.size();
    dataCellMap->version = std::make_shared<ss::ssfb2::DataVersion>();
    // dataCellMap.Version.Code = LibraryEditor_SpriteStudio6.KindVersionData.TARGET_LATEST;
    // dataCellMap.Version.Property = ss.ssfb2.DataVersionPropertyFlagBitStatus.Clear;

    dataCellMap->index_texture = {};
    for(int i=0; i<countSSCE; i++) {
        dataCellMap->index_texture.push_back(sspjInfo->TableInformationSSCE[i]->IndexTexture);
    }
    dataCellMap->table_cell_map = {};
    for(int i=0; i<countSSCE; i++) {
        dataCellMap->table_cell_map.push_back(sspjInfo->TableInformationSSCE[i]->Data);
    }

    // TODO: impl split
    // dataCellMap.ExtendSplit = null;
    // generated Split-Data
    /*
    if(true == options.split) {
        dataCellMap.Version.Property |= (ss.ssfb2.DataVersionPropertyFlagBitStatus)(Script_SpriteStudio6_DataCellMap.PropertyVersion.SPLIT_PROJECT);
        for(int i=0; i<countSSCE; i++)
        {
            if(false == LibraryEditor_SpriteStudio6.Import.SSCE.ModeSS6PU.DataOutputSplit(ref setting, informationSSPJ, informationSSPJ.TableInformationSSCE[i]))
            {
                return(false);
            }
        }
        dataCellMap.ExtendSplit = null!;
    }
    */
}

std::shared_ptr<ss::ssfb2::CellT> SSCE::Impl::parseCell(const pugi::xml_node &node) {
    std::string valueText;
    std::istringstream iss;

    auto cell = std::make_shared<ss::ssfb2::CellT>();
    auto rectangle = std::make_shared<ss::ssfb2::RectT>();
    auto pivot = std::make_shared<ss::ssfb2::Vector2T>();
    auto sizeUntrimmed = std::make_shared<ss::ssfb2::Vector2T>();
    auto mesh = std::make_shared<ss::ssfb2::DataMeshCellT>();

    valueText = std::string(node.child("name").text().as_string());
    if (valueText.empty()) {
        throw error::getException(PHASE, DOC_PART, "unnamed cell was found: ", -1);
    }
    cell->name = valueText;

    valueText = std::string(node.child("pos").text().as_string());
    if (valueText.empty()) {
        throw SSException("No-coordinates cell is found. Name: " + cell->name, -1);
    }
    iss = std::istringstream(valueText);
    std::string x, y;
    if (!(std::getline(iss, x, ' ') && std::getline(iss, y))) {
        // Data Broken ?
        throw SSException("No-coordinates cell is found. Name: " + cell->name, -1);
    }
    rectangle->x_min = std::stof(x);
    rectangle->y_min = std::stof(y);

    valueText = std::string(node.child("size").text().as_string());
    if (valueText.empty()) {
        throw SSException("No-size cell is found. Name: " + cell->name, -1);
    }
    iss = std::istringstream(valueText);
    std::string width, height;
    if (!(std::getline(iss, width, ' ') && std::getline(iss, height))) {
        // Data Broken ?
        throw SSException("No-size cell is found. Name: " + cell->name, -1);
    }
    rectangle->x_max = std::stof(width);
    rectangle->y_max = std::stof(height);

    bool flagRotated = node.child("rotated").text().as_bool();
    if (flagRotated) {
        /* MEMO: Currently only "TexturePackerToSS" is able to rotate. (And angle is also fixed.) */
        cell->rotate = 90.0f;
    }

    auto pivotNormalizeX = 0.0f;
    auto pivotNormalizeY = 0.0f;
    valueText = std::string(node.child("pivot").text().as_string());
    if (!valueText.empty()) {
        iss = std::istringstream(valueText);
        if (std::getline(iss, x, ' ') && std::getline(iss, y)) {
            pivotNormalizeX = std::stof(x);
            pivotNormalizeY = std::stof(y);
        }
    }
    if (flagRotated) {
        auto temp = pivotNormalizeY;
        pivotNormalizeY = pivotNormalizeX;
        pivotNormalizeX = temp;
    }
    pivot->x = rectangle->x_max * (pivotNormalizeX + 0.5f);
    pivot->y = rectangle->y_max * (-pivotNormalizeY + 0.5f);

    auto sizeUntrimmedX = 0.0f;
    auto sizeUntrimmedY = 0.0f;
    auto valueBool = node.child("trimmed").text().as_bool();
    if (valueBool) {
        valueText = std::string(node.child("untrimmedSize").text().as_string());
        iss = std::istringstream(valueText);
        if (std::getline(iss, x, ' ') && std::getline(iss, y)) {
            sizeUntrimmedX = std::stof(x);
            sizeUntrimmedY = std::stof(y);
            if (flagRotated) {
                auto temp = sizeUntrimmedY;
                sizeUntrimmedY = sizeUntrimmedX;
                sizeUntrimmedX = temp;
            }
        }
    }
    sizeUntrimmed->x = sizeUntrimmedX;
    sizeUntrimmed->y = sizeUntrimmedY;

    // Mesh
    valueBool = node.child("ismesh").text().as_bool();
    if (valueBool) {
        mesh->table_coordinate = this->parseMeshCoordinate(node.select_nodes("meshPointList/value"));
        mesh->table_index_vertex = this->parseMeshIndices(node.select_nodes("meshTriList/value"));
    }

    cell->rectangle = rectangle;
    cell->pivot = pivot;
    cell->size_untrimmed = sizeUntrimmed;
    cell->mesh = mesh;
    return cell;
}

std::vector<std::shared_ptr<ss::ssfb2::Vector2T>> SSCE::Impl::parseMeshCoordinate(const pugi::xpath_node_set &xpath_nodes) {
    auto lst = std::vector<std::shared_ptr<ss::ssfb2::Vector2T>>();

    for (const auto& xpath_node : xpath_nodes) {
        auto item = std::make_shared<ss::ssfb2::Vector2T>();
        auto valueText = std::string(xpath_node.node().text().as_string());
        std::istringstream iss(valueText);

        std::string x, y;
        if (std::getline(iss, x, ' ') && std::getline(iss, y)) {
            item->x = std::stof(x);
            item->y = std::stof(y);
        }
        lst.push_back(item);
    }

    return std::move(lst);
}

std::vector<uint16_t> SSCE::Impl::parseMeshIndices(const pugi::xpath_node_set &xpath_nodes) {
    auto lst = std::vector<uint16_t>();

    for (const auto& xpath_node : xpath_nodes) {
        auto valueText = std::string(xpath_node.node().text().as_string());
        std::istringstream iss(valueText);

        std::string x, y, z;
        if (std::getline(iss, x, ' ') && std::getline(iss, y, ' ') && std::getline(iss, z)) {
            lst.push_back(std::stoi(x));
            lst.push_back(std::stoi(y));
            lst.push_back(std::stoi(z));
        }
    }

    return std::move(lst);
}
