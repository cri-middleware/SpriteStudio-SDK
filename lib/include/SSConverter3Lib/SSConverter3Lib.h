#pragma once

#include <string>
#include <memory>

namespace SpriteStudio {
    class ConvertOptions {
    public:
        bool debugMode{false};
        bool split{false};
        std::string outputDir;
        std::function<void(const std::string& taskName, int progress, int maxProgress)> progressCallback;
    };

    class SSConverter3Lib {
    public:
        static void convert(const std::string &input, const ConvertOptions& options = ConvertOptions());

    private:
        struct Impl;
        static std::unique_ptr<Impl> pImpl;
    };
}
