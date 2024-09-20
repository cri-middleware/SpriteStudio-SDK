#pragma once

#include <string>
#include <memory>

namespace SpriteStudio {
    class ConvertOptions {
    public:
        bool debugMode{false};
        std::string outputDir;
    };

    class SSConverter3Lib {
    public:
        static void convert(const std::string &input, const ConvertOptions& options = ConvertOptions());

    private:
        struct Impl;
        static std::unique_ptr<Impl> pImpl;
    };
}
