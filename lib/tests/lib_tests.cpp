#include <iostream>
#include <fstream>
#include <vector>
#include <SSConverter3Lib/SSConverter3Lib.h>
#include <catch2/catch_test_macros.hpp>

#include "test_config.h"

TEST_CASE( "convert sspj files", "[SSConverter3Lib]" ) {
    for (const auto& sspj_file : sspj_files) {
        auto sspj_path = std::filesystem::path(TESTDATADIR) / std::filesystem::path(sspj_file);
        SpriteStudio::SSConverter3Lib::convert(sspj_path);
    }
}
