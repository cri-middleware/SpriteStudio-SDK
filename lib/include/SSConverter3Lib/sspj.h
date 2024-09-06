#pragma once

#include <string>
#include <vector>

#include "ssce.h"
#include "ssae.h"
#include "ssee.h"
#include "ssqe.h"
#include "fnt.h"
#include "ssfb2/ssfb2_generated.h"

namespace SpriteStudio {
    class SSSE;
    class SSSEInformation;

    enum class KindWrap : uint8_t {
        Clamp = 0,
        Repeart,
        Mirror, // Unsupported
    };

    enum class KindFilter : uint8_t {
        Nearest = 0,
        Linear,
        Bilinear,
    };

    enum class KindCoordUnit : int8_t {
        Invalid = -1,
        Rate,
        Pix,

        Terminator,
    };

    class SSPJInformation {
    public:
        std::string sspj;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;

        KindWrap WrapTexture;
        KindFilter FilterTexture;
        bool FlagConvertImagePremultipliedAlpha;
        bool FlagBlendImagePremultipliedAlpha;
        bool FlagCorrectShaderPremultipliedAlpha;
        bool FlagVertexAnimeFloat;

        std::string NameDirectoryBaseTexture;
        //SSCE::Information.Texture[] TableInformationTexture;
        //std::vector<std::string> ListNameTexture;	/* Temporary ... Accumulating list for determine TableNameTexture. Valid during analyzing SSxx-s. */

        // SSCE::Information::Texture[] TableInformationTextureExternal;
        // std::vector<TextureExternal> ListTextureExternal;	/* Temporary ... Accumulating list for determine TableInformationTextureExternal. Valid during analyzing SSxx-s. */


        std::string NameDirectoryBaseSSCE;
        std::vector<SSCE::Information> tableInformationSSCE;
        std::vector<std::string> TableNameSSCE;	/* Temporary */
        std::vector<std::string> tableNameCellMapDataProject;

        std::string NameDirectoryBaseSSAE;
        std::vector<SSAE::Information> TableInformationSSAE;
        std::vector<std::string> TableNameSSAE;
        std::vector<int> QueueConvertSSAE;
        std::vector<std::string> TableNameAnimationDataProject;

        std::string NameDirectoryBaseSSEE;
        std::vector<SSEE::Information> TableInformationSSEE;
        std::vector<std::string> TableNameSSEE; /* Temporary */
        std::vector<std::string> TableNameEffectDataProject;

        std::string NameDirectoryBaseSSQE;
        std::vector<SSQE::Information> TableInformationSSQE;
        std::vector<std::string> TableNameSSQE;
        std::vector<std::string> TableNameSequencePackDataProject;

        std::string NameDirectoryBaseSSSE;
        std::vector<std::shared_ptr<SSSEInformation>> TableInformationSSSE;
        std::vector<std::string> TableNameSSSE;
        std::vector<std::string> TableNameSoundListDataProject;

        std::string NameDirectoryBaseFnt;
        std::vector<Fnt::Information> TableInformationFnt;
        std::vector<std::string> TableNameFnt;
        std::vector<std::string> TableNameFontBitmapDataProject;

        std::vector<std::string> TableNameTextureProject;
        std::vector<std::string> TableNameTextureExternalProject;

        ss::ssfb2::DataCellMapT dataCellMapT;
        ss::ssfb2::ProjectDataT projectDataT;

        std::string Signal;
        KindCoordUnit CoordUnit;
    };

    class SSPJ {
    public:
        SSPJ();
        ~SSPJ();

        std::shared_ptr<SSPJInformation> parser(const std::string &sspj);

    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;
    };
}
