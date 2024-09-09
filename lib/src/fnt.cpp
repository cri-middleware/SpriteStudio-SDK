#include <filesystem>

#include "SSException.h"
#include "SSConverter3Lib/fnt.h"

#include "error.h"
#include "util.h"
#include "SSConverter3Lib/SSConverter3Lib.h"

using namespace SpriteStudio;
const PhaseType PHASE = PhaseType::PARSE;
const DocPartType DOC_PART = DocPartType::FNT;

struct Fnt::Impl {

};

std::string FntInformation::NameDirectory() const {
    return util::getNameDirectory(fnt);
}

std::string FntInformation::NameFileBody() const {
    return util::getNameFileBody(fnt);
}

std::string FntInformation::NameFileExtension() const {
    return util::getNameFileExtension(fnt);
}

Fnt::Fnt() : pImpl(std::make_unique<Impl>()){
}

Fnt::~Fnt() = default;

std::shared_ptr<SpriteStudio::FntInformation> SpriteStudio::Fnt::parse(const std::string &fnt, const std::shared_ptr<SSPJInformation> &sspjInfo) {
    auto fnt_path = std::filesystem::path(fnt);
    if (!std::filesystem::exists(fnt_path)) {
        throw error::getException(PHASE, DOC_PART, "not found a fnt file: " + fnt, -1);
    }

    auto fntInfo = std::make_shared<FntInformation>();
    if (fntInfo == nullptr) {
        throw SSException("not enough memory", -1);
    }
    fntInfo->fnt = fnt;

    // TODO: impl parse fnt

    return fntInfo;
}
