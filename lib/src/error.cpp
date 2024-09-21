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

const std::map<PhaseType, std::string> PhaseTypeStringMap = {
    { PhaseType::PARSE, "parse" },
    { PhaseType::CONVERT, "convert" },
    { PhaseType::COMPRESS, "compress" },
    { PhaseType::PACKING, "packing" },
};

const std::map<PhaseType, uint32_t> PhaseTypeBitFieldMap = {
    { PhaseType::PARSE, 0x01000000 },
    { PhaseType::CONVERT, 0x02000000 },
    { PhaseType::COMPRESS, 0x04000000 },
    { PhaseType::PACKING, 0x08000000 },
};

std::string error::getErrorMessage(PhaseType phaseType, DocPartType docPartType, const std::string& message) {
    return "Error: Phase: " + PhaseTypeStringMap.at(phaseType) + " Type: " + DocPartTypeStringMap.at(docPartType) + ": " + message;
}

SSException error::getException(PhaseType phaseType, DocPartType docPartType, const std::string &message, int code) {
    std::string msg = getErrorMessage(phaseType, docPartType, message);
    int combineCode = static_cast<int>(PhaseTypeBitFieldMap.at(phaseType)) | static_cast<int>(DocPartTypeErrorBitFieldMap.at(docPartType)) | code;
    return SSException(msg, combineCode);
}

std::string error::getPhaseName(PhaseType phaseType) {
    return PhaseTypeStringMap.at(phaseType);
}
