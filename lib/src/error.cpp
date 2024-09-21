#include "error.h"

using namespace SpriteStudio;

std::string error::getErrorMessage(PhaseType phaseType, DocPartType docPartType, const std::string& message) {
    return "Error: Phase: " + phase::getPhaseName(phaseType) + " Type: " + phase::getDocPartName(docPartType) + ": " + message;
}

SSException error::getException(PhaseType phaseType, DocPartType docPartType, const std::string &message, int code) {
    std::string msg = getErrorMessage(phaseType, docPartType, message);
    int combineCode = static_cast<int>(phase::getPhaseBitField(phaseType)) | static_cast<int>(phase::getDocPartBitField(docPartType)) | code;
    return SSException(msg, combineCode);
}
