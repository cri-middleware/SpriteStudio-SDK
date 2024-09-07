#pragma once

#include <string>
#include <vector>
#include "ssfb2/ssc2_generated.h"
#include "ssfb2/ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJInformation;

    class SSCEInformation {
    public:
        ss::ssfb2::LibraryDataCellMapT Data;
        ss::ssfb2::DataCellmapExtendSplitT DataCellmapExtendSplitT;

        std::string ssce;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;

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
