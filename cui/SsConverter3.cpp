#include <vector>
#include <iostream>
#include <filesystem>

#include <SSConverter3Lib/SSConverter3Lib.h>
#include <argparse/argparse.hpp>

#include <indicators/progress_bar.hpp>

using namespace indicators;

int main(int argc, char *argv[]) {
    auto prog_name = std::filesystem::path(argv[0]);
    argparse::ArgumentParser program(prog_name.filename().string());

    program.add_argument("-o", "--output")
            .help("set output directory path (default: the same parent directory of a sspj file)");

    program.add_argument("files")
            .help("sspj files")
            .remaining();

    program.add_argument("-s", "--split")
            .flag()
            .help("split a ssfb file converted from one sspj file to each animation packs");

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
        if (files.empty()) {
            throw std::logic_error("no files");
        }
    } catch (const std::exception& e) {
        std::cerr << program.help().str() << std::endl;
        return 1;
    }

    // validate
    auto output = program.present("-o");
    std::filesystem::path outPath;
    if (output) {
        outPath = std::filesystem::weakly_canonical(std::filesystem::absolute(*output));
        if (!std::filesystem::exists(outPath)) {
            std::cerr << u8"not found the output directory: " + *output << std::endl;
            return 1;
        }
    }

    SpriteStudio::ConvertOptions convertOptions;
    if (program["--debug"] == true) {
        convertOptions.debugMode = true;
    }
    if (program["--split"] == true) {
        convertOptions.split = true;
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
            convertOptions.outputDir = output ? outPath : filePath.parent_path();

            ProgressBar bar{
                option::BarWidth{40},
                option::Start{"["},
                option::End{"]"},
                option::ForegroundColor{Color::white}  ,
                option::FontStyles{std::vector<FontStyle>{FontStyle::bold}},
                option::PostfixText{filePath.filename().string()},
            };
            convertOptions.progressCallback = [&convertOptions, &bar, &filePath](const std::string& taskName, int progress, int maxProgress) {
                if (convertOptions.debugMode) {
                    std::cout << taskName << " : " << progress << "/" << maxProgress << std::endl;
                } else {
                    bar.set_progress(static_cast<size_t>((static_cast<float>(progress) / static_cast<float>(maxProgress)) * 100.0f));
                    bar.set_option(option::PostfixText{ filePath.filename().string() + " " + std::to_string(progress) + "/" + std::to_string(maxProgress)});
                }
            };

            SpriteStudio::SSConverter3Lib::convert(filePath.string(), convertOptions);
            bar.set_progress(100);
        } catch(const std::exception &e) {
            std::cerr << e.what() << std::endl;
            return 1;
        }
    }

    return 0;
}
