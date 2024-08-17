#pragma once

#include <vector>
#include <filesystem>
#include <memory>

#include "ssce.h"
#include "ssae.h"
#include "ssee.h"
#include "ssqe.h"
#include "ssse.h"
#include "fnt.h"
#include "ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJ {
    public:
        class Information {
        public:
            std::filesystem::path sspj;
            std::string NameDirectory() const {
                return sspj.parent_path().string();
            };
            std::string NameFileBody() const {
                return sspj.filename().stem().string();
            };
            std::string NameFileExtension() const {
                return sspj.filename().extension().string();
            };

            // Library_SpriteStudio6.Data.Texture.KindWrap WrapTexture;
            // Library_SpriteStudio6.Data.Texture.KindFilter FilterTexture;
            bool FlagConvertImagePremultipliedAlpha;
            bool FlagBlendImagePremultipliedAlpha;
            bool FlagCorrectShaderPremultipliedAlpha;
            bool FlagVertexAnimeFloat;


            std::string nameDirectoryBaseSSCE;
            std::vector<SSCE::Information> tableInformationSSCE;
            std::vector<std::string> tableNameCellMapDataProject;

            std::string NameDirectoryBaseSSAE;
            std::vector<SSAE::Information> TableInformationSSAE;
            std::vector<int> QueueConvertSSAE;
            std::vector<std::string> TableNameAnimationDataProject;

            std::string NameDirectoryBaseSSEE;
            std::vector<SSEE::Information> TableInformationSSEE;
            std::vector<std::string> TableNameEffectDataProject;

            std::string NameDirectoryBaseSSQE;
            std::vector<SSQE::Information> TableInformationSSQE;
            std::vector<std::string> TableNameSequencePackDataProject;

            std::string NameDirectoryBaseSSSE;
            std::vector<SSSE::Information> TableInformationSSSE;
            std::vector<std::string> TableNameSoundListDataProject;

            std::string NameDirectoryBaseFnt;
            std::vector<Fnt::Information> TableInformationFnt;
            std::vector<std::string> TableNameFontBitmapDataProject;

            std::vector<std::string> TableNameTextureProject;
            std::vector<std::string> TableNameTextureExternalProject;

            ss::ssfb2::DataCellMapT dataCellMapT;
            ss::ssfb2::ProjectDataT projectDataT;
        };

    public:
        std::shared_ptr<SSPJ::Information> parser(const std::filesystem::path &sspj);
    };
}
