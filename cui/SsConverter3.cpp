#include <vector>
#include <iostream>
#include <filesystem>

#include <SSConverter3Lib/SSConverter3Lib.h>
#include <argparse/argparse.hpp>


void convert(const std::filesystem::path &inputPath, const std::filesystem::path &outputDirPath) {
    std::cout << inputPath << " " << outputDirPath;
    auto inputName = inputPath.filename().stem().string();
    auto inputExt = inputPath.filename().extension().string();

    SpriteStudio::SSPJ sspj;
    sspj.parser(inputPath);
}

int main(int argc, char *argv[]) {
    argparse::ArgumentParser program("program_name");

    program.add_argument("-o", "--output")
            .help("set output directory path (default: the same parent directory of a sspj file)");

    program.add_argument("files")
            .help("sspj files")
            .remaining();

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
    for(const auto& file : files) {
        auto filePath = std::filesystem::weakly_canonical(std::filesystem::absolute(file));
        if (!std::filesystem::exists(filePath)) {
            std::cerr << u8"not found the file : " + file << std::endl;
            return 1;
        }
    }

    for(const auto& file : files) {
        try {
            auto filePath = std::filesystem::weakly_canonical(std::filesystem::absolute(file));
            auto output = program.present("-o");
            auto outputPath = output ? std::filesystem::weakly_canonical(std::filesystem::absolute(*output)) : filePath.parent_path();
            convert(filePath, outputPath);
        } catch(std::exception &e) {
            return 1;
        }
    }

    return 0;
}
