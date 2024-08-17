#pragma once

#include <string>
#include <vector>
#include "ssc2_generated.h"

namespace SpriteStudio {
    class SSCE {
    public:
        class Information {
        public:
            ss::ssfb2::DataCellmapExtendSplitT dataCellmapExtendSplitT;

            std::string NameDirectory;
            std::string NameFileBody;
            std::string NameFileExtension;

            int IndexTexture;
            int SizePixelX;	/* Temporary */
            int SizePixelY;	/* Temporary */
            // Library_SpriteStudio6.Data.Texture.KindWrap WrapTexture;
            // Library_SpriteStudio6.Data.Texture.KindFilter FilterTexture;
            bool FlagConvertImagePremultipliedAlpha;
            bool FlagBlendImagePremultipliedAlpha;
            bool FlagCorrectShaderPremultipliedAlpha;

            std::vector<ss::ssfb2::CellT> TableCell;

            std::vector<std::string> TableNameSpriteUnityNative;	/* Temporary */

            // LibraryEditor_SpriteStudio6.Import.Assets<Script_SpriteStudio6_DataCellMap> DataCellMapSS6PU;

        };

    public:

    };
}
