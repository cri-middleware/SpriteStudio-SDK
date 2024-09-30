#pragma once
#include <memory>

#include "ssfb2/type_base_generated.h"

namespace SpriteStudio {
    class SSPJInformation;

    class FntInformation {
    public:
        std::string fnt;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;
        std::string NameFile() const;

        std::string NameFace;
        int Size;
        // FlagBitStatus Status;
        ss::ssfb2::Vector2T Scale;							/* Strech */
        // Vector4 Padding;							/* .x=L / .y=U / .z=R / .W=D */
        ss::ssfb2::Vector2T Spacing;

        int HeightLine;
        int Base;
        ss::ssfb2::Vector2T PixelSizeTexture;
        int CountPage;

        int CountGryph;

        std::vector<std::string> TableNameTexture;
        // Gryph[]? TableGryph;

        // public LibraryEditor_SpriteStudio6.Import.Assets<Script_SpriteStudio6_DataFontBitmap> DataFontBitmapSS6PU;
    };

    class Fnt {
    public:
        Fnt();
        ~Fnt();

        std::shared_ptr<FntInformation> parse(const std::string &fnt, const std::shared_ptr<SSPJInformation>& sspjInfo);
    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;
    };
}
