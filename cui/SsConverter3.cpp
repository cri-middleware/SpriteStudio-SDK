#include <vector>
#include <iostream>
#include <filesystem>

#include <SSConverter3Lib/SSConverter3Lib.h>
#include <argparse/argparse.hpp>

int main(int argc, char *argv[]) {
    argparse::ArgumentParser program("program_name");

    program.add_argument("-o", "--output")
            .help("set output directory path (default: the same parent directory of a sspj file)");

    program.add_argument("files")
            .help("sspj files")
            .remaining();

    program.add_argument("-d", "--debug")
            .flag()
            .help("debug flag");

    try {
        program.parse_args(argc, argv);
    }
    catch (const std::exception& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        return 1;
    }

    std::vector<std::string> files;
    try {
        files = program.get<std::vector<std::string>>("files");
    } catch (std::logic_error& e) {
        return 0;
    }

    // validate
    if (auto fn = program.present("-o")) {
        auto outPath = std::filesystem::weakly_canonical(std::filesystem::absolute(*fn));
        if (!std::filesystem::exists(outPath)) {
            std::cerr << u8"not found the output directory: " + *fn << std::endl;
            return 1;
        }
    }
    bool debugMode = false;
    if (program["--debug"] == true) {
        debugMode = true;
    }
    for(const auto& file : files) {
        auto filePath = std::filesystem::weakly_canonical(std::filesystem::absolute(file));
        if (!std::filesystem::exists(filePath)) {
            std::cerr << u8"not found the file : " + file << std::endl;
            return 1;
        }
    }

    auto output = program.present("-o");
    for(const auto& file : files) {
        try {
            auto filePath = std::filesystem::weakly_canonical(std::filesystem::absolute(file));
            auto outputPath = output ? std::filesystem::weakly_canonical(std::filesystem::absolute(*output)) : filePath.parent_path();
            SpriteStudio::SSConverter3Lib::convert(filePath.string(), SpriteStudio::ConvertOptions{debugMode, outputPath.string()});
        } catch(const std::exception &e) {
            std::cerr << e.what() << std::endl;
            return 1;
        }
    }

    return 0;
}
