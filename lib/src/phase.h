#pragma once
#include <cstdint>

namespace SpriteStudio {
  enum class DocPartType : uint32_t {
    SSPJ,
    SSCE,
    SSAE,
    SSEE,
    SSQE,
    SSSE,
    FNT,
  };

  enum class PhaseType : uint32_t {
    PARSE,
    CONVERT,
    COMPRESS,
    PACKING,

    MAX = PACKING,
  };

  class phase {
  public:
    static std::string getDocPartName(DocPartType docPartType);
    static std::string getPhaseName(PhaseType phaseType);
    static uint32_t getDocPartBitField(DocPartType docPartType);
    static uint32_t getPhaseBitField(PhaseType phaseType);
  };
}
