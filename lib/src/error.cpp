#include "error.h"

using namespace SpriteStudio;

const std::map<DocPartType, std::string> DocPartTypeStringMap = {
    { DocPartType::SSPJ, "sspj"},
    { DocPartType::SSCE, "ssce"},
    { DocPartType::SSAE, "ssae"},
    { DocPartType::SSEE, "ssee"},
    { DocPartType::SSQE, "ssqe"},
    { DocPartType::SSSE, "ssse"},
    { DocPartType::FNT, "fnt"},
};

const std::map<DocPartType, uint32_t> DocPartTypeErrorBitFieldMap = {
    { DocPartType::SSPJ, 0x00010000 },
    { DocPartType::SSCE, 0x00020000 },
    { DocPartType::SSAE, 0x00040000 },
    { DocPartType::SSEE, 0x00080000 },
    { DocPartType::SSQE, 0x00100000 },
    { DocPartType::SSSE, 0x00200000 },
    { DocPartType::FNT,  0x00400000 },
};

std::string error::getErrorMessage(DocPartType docPartType, const std::string &message) {
    return "Error: " + DocPartTypeStringMap.at(docPartType) + ": " + message;
}

SSException error::getException(DocPartType docPartType, const std::string &message, int code) {
    std::string msg = getErrorMessage(docPartType, message);
    return SSException(msg, DocPartTypeErrorBitFieldMap.at(docPartType) | code);
}
