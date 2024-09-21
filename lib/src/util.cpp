#include <filesystem>

#include "util.h"

using namespace SpriteStudio;

uint32_t util::getVersion(const std::string &valueText, const std::map<uint32_t, std::string> &map) {
    for (const auto& pair : map) {
        if (pair.second == valueText) {
            return pair.first;
        }
    }

    return static_cast<uint32_t>(Version::ERROR);
}

std::string util::getNameDirectory(const std::string &file_path) {
    return std::filesystem::path(file_path).parent_path().string();
}

std::string util::getNameFileBody(const std::string &file_path) {
    return std::filesystem::path(file_path).filename().stem().string();
}

std::string util::getNameFileExtension(const std::string &file_path) {
    return std::filesystem::path(file_path).filename().extension().string().substr(1);
}

std::string util::getFileName(const std::string &file_path) {
    return std::filesystem::path(file_path).filename();
}
