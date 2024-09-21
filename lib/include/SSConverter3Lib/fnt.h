#pragma once
#include <memory>

namespace SpriteStudio {
    class SSPJInformation;

    class FntInformation {
    public:
        std::string fnt;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;
        std::string NameFile() const;
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
