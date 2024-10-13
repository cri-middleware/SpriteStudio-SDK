#pragma once

#include <string>
#include <vector>

#include "SSConverter3Lib.h"
#include "version.h"
#include "ssfb2/type_base_generated.h"
#include "ssfb2/Data/common_ssfb2_generated.h"
#include "ssfb2/Data/ssc2_generated.h"
#include "ssfb2/Data/ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJInformation;
    enum class KindWrap;
    enum class KindFilter;

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
        std::shared_ptr<ss::ssfb2::LibraryDataCellMapT> Data;
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
        KindWrap WrapTexture;
        KindFilter FilterTexture;
        bool FlagConvertImagePremultipliedAlpha;
        bool FlagBlendImagePremultipliedAlpha;
        bool FlagCorrectShaderPremultipliedAlpha;

        std::vector<std::shared_ptr<ss::ssfb2::CellT>> TableCell;

        std::vector<std::string> TableNameSpriteUnityNative;	/* Temporary */

        ss::ssfb2::DataCellMapT DataCellMapSS6PU;

    public:

    };

    class SSCEInformationTexture {
    public:
        std::string file;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;
        std::string NameFile() const;

        KindWrap Wrap;
        KindFilter Filter;

        int SizeX;
        int SizeY;
    };

    class SSCE {
    public:
        SSCE();
        ~SSCE();

        std::shared_ptr<SSCEInformation> parse(const std::string& ssce, const std::shared_ptr<SSPJInformation> &sspjInfo);

        void DataConverter(const ConvertOptions& options, const std::shared_ptr<SSPJInformation>& sspjInfo, const std::shared_ptr<SSCEInformation>& item);

        void DataOutputCellMap(const ConvertOptions& options, const std::shared_ptr<SSPJInformation>& sspjInfo);

    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;
    };
}
