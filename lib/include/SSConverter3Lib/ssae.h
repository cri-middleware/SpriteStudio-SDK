#pragma once

#include "version.h"
#include "ssfb2/type_base_generated.h"
#include "ssfb2/Data/common_ssfb2_generated.h"

namespace SpriteStudio {
    class SSPJInformation;

    enum class SSAEVersion : uint32_t {
        ERROR = static_cast<uint32_t>(Version::ERROR),
        CODE_020000 = static_cast<uint32_t>(Version::CODE_020000),
        CODE_020001 = static_cast<uint32_t>(Version::CODE_020001),
        CODE_020003 = static_cast<uint32_t>(Version::CODE_020003),
        CODE_020004 = static_cast<uint32_t>(Version::CODE_020004),

        CODE_020100 = static_cast<uint32_t>(Version::CODE_020100),
    };
    const std::map<uint32_t, std::string> SSAEVersionMap = {
        {static_cast<uint32_t>(SSAEVersion::CODE_020000), VersionToStringMap.at(static_cast<uint32_t>(SSAEVersion::CODE_020000))},
        {static_cast<uint32_t>(SSAEVersion::CODE_020001), VersionToStringMap.at(static_cast<uint32_t>(SSAEVersion::CODE_020001))},
        {static_cast<uint32_t>(SSAEVersion::CODE_020003), VersionToStringMap.at(static_cast<uint32_t>(SSAEVersion::CODE_020003))},
        {static_cast<uint32_t>(SSAEVersion::CODE_020004), VersionToStringMap.at(static_cast<uint32_t>(SSAEVersion::CODE_020004))},

        {static_cast<uint32_t>(SSAEVersion::CODE_020100), VersionToStringMap.at(static_cast<uint32_t>(SSAEVersion::CODE_020100))},
    };


    class SSAEPartsInformation {
    public:
        std::shared_ptr<ss::ssfb2::DataModelPartsT> data;

        int IndexCellMapMeshBind;
        int IndexCellMeshBind;

        float LengthBone;
        float PositionXBone;
        float PositionYBone;
        float RotateZBone;

        enum KindInheritance {
            KindInheritance_PARENT = 0,
            KindInheritance_SELF
        };
        KindInheritance Inheritance;
        enum FlagBitInheritance {
            FlagBitInheritance_OPACITY_RATE = 0x000000001,
            FlagBitInheritance_SHOW_HIDE = 0x000000002,
            FlagBitInheritance_FLIP_X = 0x000000010,
            FlagBitInheritance_FLIP_Y = 0x000000020,

            FlagBitInheritance_CLEAR = 0x00000000,
            FlagBitInheritance_ALL = FlagBitInheritance_OPACITY_RATE
                | FlagBitInheritance_SHOW_HIDE
                | FlagBitInheritance_FLIP_X
                | FlagBitInheritance_FLIP_Y,
            FlagBitInheritance_PRESET = FlagBitInheritance_OPACITY_RATE
        };
        FlagBitInheritance FlagInheritance;

        std::vector<int> ListIndexPartsChild;

        bool FlagHide;
        bool FlagMasking;
        bool FlagVerbose;

        bool FlagDeform;
    };

    class SSAEInformation {
    public:
        std::string ssae;
        std::string NameDirectory() const;
        std::string NameFileBody() const;
        std::string NameFileExtension() const;
        std::string NameFile() const;

        SSAEVersion version;

        ss::ssfb2::LibraryDataAnimationT data;
        std::vector<std::shared_ptr<SSAEPartsInformation>> TableParts;
    };

    class SSAE {
    public:
        SSAE();
        ~SSAE();

        std::shared_ptr<SSAEInformation> parse(const std::string& ssae, const std::shared_ptr<SSPJInformation> &sspjInfo);
    private:
        struct Impl;
        std::unique_ptr<Impl> pImpl;
    };
}
