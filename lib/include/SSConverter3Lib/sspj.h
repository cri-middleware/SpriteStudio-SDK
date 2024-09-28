#pragma once

#include <string>
#include <vector>
#include <mutex>

#include "version.h"
#include "ssfb2/type_base_generated.h"
#include "ssfb2/Data/common_ssfb2_generated.h"
#include "ssfb2/Data/ssfb2_generated.h"

namespace SpriteStudio {
    class SSSE;
    class SSSEInformation;
    class Fnt;
    class FntInformation;
    class SSCE;
    class SSCEInformation;
    class SSAE;
    class SSAEInformation;
    class SSEE;
    class SSEEInformation;
    class SSQE;
    class SSQEInformation;

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


    enum class SSPJVersion : uint32_t {
        ERROR = static_cast<uint32_t>(Version::ERROR),
        CODE_020000 = static_cast<uint32_t>(Version::CODE_020000),
        CODE_020001 = static_cast<uint32_t>(Version::CODE_020001),
        CODE_020003 = static_cast<uint32_t>(Version::CODE_020003),
        CODE_020004 = static_cast<uint32_t>(Version::CODE_020004),

        CODE_020100 = static_cast<uint32_t>(Version::CODE_020100),
    };
    const std::map<uint32_t, std::string> SSPJVersionMap = {
        {static_cast<uint32_t>(SSPJVersion::CODE_020000), VersionToStringMap.at(static_cast<uint32_t>(SSPJVersion::CODE_020000))},
        {static_cast<uint32_t>(SSPJVersion::CODE_020001), VersionToStringMap.at(static_cast<uint32_t>(SSPJVersion::CODE_020001))},
        {static_cast<uint32_t>(SSPJVersion::CODE_020003), VersionToStringMap.at(static_cast<uint32_t>(SSPJVersion::CODE_020003))},
        {static_cast<uint32_t>(SSPJVersion::CODE_020004), VersionToStringMap.at(static_cast<uint32_t>(SSPJVersion::CODE_020004))},

        {static_cast<uint32_t>(SSPJVersion::CODE_020100), VersionToStringMap.at(static_cast<uint32_t>(SSPJVersion::CODE_020100))},
    };

    class SSPJInformation {
    public:
        std::string sspj;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;

        SSPJVersion version;

        KindWrap WrapTexture;
        KindFilter FilterTexture;
        bool FlagConvertImagePremultipliedAlpha;
        bool FlagBlendImagePremultipliedAlpha;
        bool FlagCorrectShaderPremultipliedAlpha;
        bool FlagVertexAnimeFloat;

        std::string NameDirectoryBaseTexture;
        //SSCE::Information.Texture[] TableInformationTexture;
        std::vector<std::string> ListNameTexture;	/* Temporary ... Accumulating list for determine TableNameTexture. Valid during analyzing SSxx-s. */

        // SSCE::Information::Texture[] TableInformationTextureExternal;
        std::vector<std::tuple<std::string, int, int>> ListTextureExternal;	/* Temporary ... Accumulating list for determine TableInformationTextureExternal. Valid during analyzing SSxx-s. */


        std::string NameDirectoryBaseSSCE;
        std::vector<std::shared_ptr<SSCEInformation>> TableInformationSSCE;
        std::vector<std::string> TableNameSSCE;	/* Temporary */
        std::vector<std::string> tableNameCellMapDataProject;

        std::string NameDirectoryBaseSSAE;
        std::vector<std::shared_ptr<SSAEInformation>> TableInformationSSAE;
        std::vector<std::string> TableNameSSAE;
        std::vector<int> QueueConvertSSAE;
        std::vector<std::string> TableNameAnimationDataProject;

        std::string NameDirectoryBaseSSEE;
        std::vector<std::shared_ptr<SSEEInformation>> TableInformationSSEE;
        std::vector<std::string> TableNameSSEE; /* Temporary */
        std::vector<std::string> TableNameEffectDataProject;

        std::string NameDirectoryBaseSSQE;
        std::vector<std::shared_ptr<SSQEInformation>> TableInformationSSQE;
        std::vector<std::string> TableNameSSQE;
        std::vector<std::string> TableNameSequencePackDataProject;

        std::string NameDirectoryBaseSSSE;
        std::vector<std::shared_ptr<SSSEInformation>> TableInformationSSSE;
        std::vector<std::string> TableNameSSSE;
        std::vector<std::string> TableNameSoundListDataProject;

        std::string NameDirectoryBaseFnt;
        std::vector<std::shared_ptr<FntInformation>> TableInformationFnt;
        std::vector<std::string> TableNameFnt;
        std::vector<std::string> TableNameFontBitmapDataProject;

        std::vector<std::string> TableNameTextureProject;
        std::vector<std::string> TableNameTextureExternalProject;

        ss::ssfb2::DataCellMapT dataCellMapT;
        ss::ssfb2::ProjectDataT projectDataT;

        std::string Signal;
        KindCoordUnit CoordUnit;

        std::vector<std::string> ListNameFontBitmap;
        std::vector<std::string> TableNameFontBitmap;

    public:
        int addTexture(const std::string& nameTexture);
        int addTextureExternal(const std::string& nameTexture, int indexTexture, int flagTexture);
        int indexGetFileName(const std::vector<std::string>& tableNameFile, const std::string& nameFile);
        int indexGetAnimation(const std::string& name);
        int indexGetFontBitmap(const std::string& name);

    private:
        std::mutex m;
    };

    class SSPJ {
    public:
        SSPJ();
        ~SSPJ();

        std::shared_ptr<SSPJInformation> parse(const std::string &sspj);

    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;
    };
}
