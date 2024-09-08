#include "error.h"

using namespace SpriteStudio;

std::string error::getErrorMessage(const std::string &tag, const std::string &message) {
    return "Error: " + tag + ": " + message;
}

SSException error::getException(const std::string &tag, const std::string &message, int code) {
    std::string msg = getErrorMessage(tag, message);
    return SSException(msg, code);
}
