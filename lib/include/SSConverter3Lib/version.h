#pragma once
#include <cstdint>
#include <string>
#include <map>

namespace SpriteStudio {
    enum class Version : uint32_t {
        ERROR = 0x00000000,
        CODE_010000 = 0x00010000,
        CODE_020000 = 0x00020000,
        CODE_020001 = 0x00020001,
        CODE_020003 = 0x00020003,
        CODE_020004 = 0x00020004,
        CODE_020100 = 0x00020100,
        CODE_020101 = 0x00020101,
    };

    // enum値と文字列の対応を定義するmap
    const std::map<uint32_t, std::string> VersionToStringMap = {
        {static_cast<uint32_t>(Version::CODE_010000), "1.00.00"},
        {static_cast<uint32_t>(Version::CODE_020000), "2.00.00"},
        {static_cast<uint32_t>(Version::CODE_020001), "2.00.01"},
        {static_cast<uint32_t>(Version::CODE_020003), "2.00.03"},
        {static_cast<uint32_t>(Version::CODE_020004), "2.00.04"},
        {static_cast<uint32_t>(Version::CODE_020100), "2.01.00"},
        {static_cast<uint32_t>(Version::CODE_020100), "2.01.01"},
    };
}
