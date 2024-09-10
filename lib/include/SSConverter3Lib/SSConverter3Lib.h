#pragma once

#include <string>
#include <memory>

namespace SpriteStudio {
    class SSConverter3Lib {
    public:
        static void convert(const std::string &input, const std::string &outputDir = "");

    private:
        struct Impl;
        static std::unique_ptr<Impl> pImpl;
    };
}
