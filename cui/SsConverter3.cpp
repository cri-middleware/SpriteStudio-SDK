#include <vector>
#include <iostream>
#include <filesystem>
#include <future>

#include <SSConverter3Lib/SSConverter3Lib.h>
#include <argparse/argparse.hpp>


void convert(const std::filesystem::path &inputPath, const std::filesystem::path &outputDirPath) {
    auto inputName = inputPath.filename().stem().string();
    auto inputExt = inputPath.filename().extension().stem().string().substr(1);

    std::cout << "inputPath: " << inputPath << std::endl;
    std::cout << "outputDirPath: " << outputDirPath << std::endl;;
    std::cout << "inputName: " << inputName << std::endl;
    std::cout << "inputExt: " << inputExt << std::endl;

    SpriteStudio::SSPJ sspj;
    auto sspjInfo = sspj.parse(inputPath.string());

    // read Audio pack (SSSE)
    std::vector<std::future<std::shared_ptr<SpriteStudio::SSSEInformation>>> ssseFeatures;
    for (int i=0; i<sspjInfo->TableNameSSSE.size(); i++) {
        auto ssseStr = sspjInfo->TableNameSSSE[i];
        ssseFeatures.push_back(
            std::async(std::launch::async, [ssseStr, &sspjInfo] {
                SpriteStudio::SSSE ssse;
                return ssse.parse(ssseStr, sspjInfo);
            })
        );
    }
    sspjInfo->TableInformationSSSE = {};
    for (auto& f : ssseFeatures) {
        sspjInfo->TableInformationSSSE.push_back(f.get());
    }

    // read Font-Bitmap (Fnt)
    std::vector<std::future<std::shared_ptr<SpriteStudio::FntInformation>>> fntFeatures;
    for (int i=0; i<sspjInfo->TableNameFnt.size(); i++) {
        auto fntStr = sspjInfo->TableNameFnt[i];
        fntFeatures.push_back(
            std::async(std::launch::async, [fntStr, &sspjInfo] {
                SpriteStudio::Fnt fnt;
                return fnt.parse(fntStr, sspjInfo);
            })
        );
    }
    sspjInfo->TableInformationFnt = {};
    for (auto & f : fntFeatures) {
        sspjInfo->TableInformationFnt.push_back(f.get());
    }


    // read CellMap(SSCE) & Collect Texture-FileNames
    std::vector<std::future<std::shared_ptr<SpriteStudio::SSCEInformation>>> ssceFeatures;
    for (int i=0; i<sspjInfo->TableNameSSCE.size(); i++) {
        auto ssceStr = sspjInfo->TableNameSSCE[i];
        ssceFeatures.push_back(
            std::async(std::launch::async, [ssceStr, &sspjInfo] {
                SpriteStudio::SSCE ssce;
                return ssce.parse(ssceStr, sspjInfo);
            })
        );
    }
    sspjInfo->TableInformationSSCE = {};
    for (auto& f : ssceFeatures) {
        sspjInfo->TableInformationSSCE.push_back(f.get());
    }

    std::vector<std::future<std::shared_ptr<SpriteStudio::SSAEInformation>>> ssaeFeatures;
    for (int i=0; i<sspjInfo->TableNameSSAE.size(); i++) {
        auto ssaeStr = sspjInfo->TableNameSSAE[i];
        ssaeFeatures.push_back(
            std::async(std::launch::async, [ssaeStr, &sspjInfo] {
                SpriteStudio::SSAE ssae;
                return ssae.parse(ssaeStr, sspjInfo);
            })
        );
    }
    sspjInfo->TableInformationSSAE = {};
    for (auto& f : ssaeFeatures) {
        sspjInfo->TableInformationSSAE.push_back(f.get());
    }

    // TODO: impl
    /*
    std::vector<std::future<std::shared_ptr<SpriteStudio::SSEEInformation>>> sseeFeatures;
    for (int i=0; i<sspjInfo->TableNameSSEE.size(); i++) {
        auto sseeStr = sspjInfo->TableNameSSEE[i];
        sseeFeatures.push_back(
            std::async(std::launch::async, [sseeStr, &sspjInfo] {
                SpriteStudio::SSEE ssee;
                return ssee.parse(sseeStr, sspjInfo);
            })
        );
    }
    sspjInfo->TableInformationSSEE = {};
    for (auto& f : sseeFeatures) {
        sspjInfo->TableInformationSSEE.push_back(f.get());
    }
    */

    std::vector<std::future<std::shared_ptr<SpriteStudio::SSQEInformation>>> ssqeFeatures;
    for (int i=0; i<sspjInfo->TableNameSSQE.size(); i++) {
        auto ssqeStr = sspjInfo->TableNameSSQE[i];
        ssqeFeatures.push_back(
            std::async(std::launch::async, [ssqeStr, &sspjInfo] {
                SpriteStudio::SSQE ssqe;
                return ssqe.parse(ssqeStr, sspjInfo);
            })
        );
    }
    sspjInfo->TableInformationSSQE = {};
    for (auto& f : ssqeFeatures) {
        sspjInfo->TableInformationSSQE.push_back(f.get());
    }

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
            std::cerr << e.what() << std::endl;
            return 1;
        }
    }

    return 0;
}
