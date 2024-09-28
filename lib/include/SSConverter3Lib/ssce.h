#pragma once

#include <string>
#include <vector>
#include "version.h"
#include "ssfb2/type_base_generated.h"
#include "ssfb2/Data/common_ssfb2_generated.h"
#include "ssfb2/Data/ssc2_generated.h"
#include "ssfb2/Data/ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJInformation;

    enum class SSCEVersion : uint32_t {
        ERROR = static_cast<uint32_t>(Version::ERROR),
        CODE_010001 = static_cast<uint32_t>(Version::CODE_010001),
        CODE_020000 = static_cast<uint32_t>(Version::CODE_020000),
        CODE_020101 = static_cast<uint32_t>(Version::CODE_020101),
    };
    const std::map<uint32_t, std::string> SSCEVersionMap = {
        {static_cast<uint32_t>(SSCEVersion::CODE_010001), VersionToStringMap.at(static_cast<uint32_t>(SSCEVersion::CODE_010001))},
        {static_cast<uint32_t>(SSCEVersion::CODE_020000), VersionToStringMap.at(static_cast<uint32_t>(SSCEVersion::CODE_020000))},
        {static_cast<uint32_t>(SSCEVersion::CODE_020101), VersionToStringMap.at(static_cast<uint32_t>(SSCEVersion::CODE_020101))},
    };

    class SSCEInformation {
    public:
        ss::ssfb2::LibraryDataCellMapT Data;
        ss::ssfb2::DataCellmapExtendSplitT DataCellmapExtendSplitT;

        std::string file;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;
        std::string NameFile() const;

        SSCEVersion version;

        int IndexTexture;
        int SizePixelX;	/* Temporary */
        int SizePixelY;	/* Temporary */
        // Library_SpriteStudio6.Data.Texture.KindWrap WrapTexture;
        // Library_SpriteStudio6.Data.Texture.KindFilter FilterTexture;
        bool FlagConvertImagePremultipliedAlpha;
        bool FlagBlendImagePremultipliedAlpha;
        bool FlagCorrectShaderPremultipliedAlpha;

        std::vector<std::shared_ptr<ss::ssfb2::CellT>> TableCell;

        std::vector<std::string> TableNameSpriteUnityNative;	/* Temporary */

        ss::ssfb2::DataCellMapT DataCellMapSS6PU;

    public:

    };

    class SSCE {
    public:
        SSCE();
        ~SSCE();

        std::shared_ptr<SSCEInformation> parse(const std::string& ssce, const std::shared_ptr<SSPJInformation> &sspjInfo);
    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;
    };
}
