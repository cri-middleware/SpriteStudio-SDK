#include <iostream>
#include <vector>
#include <filesystem>
#include <future>

#include "SSConverter3Lib/SSConverter3Lib.h"
#include "SSConverter3Lib/sspj.h"
#include "SSConverter3Lib/ssse.h"
#include "SSConverter3Lib/fnt.h"
#include "SSConverter3Lib/ssce.h"
#include "SSConverter3Lib/ssae.h"
#include "SSConverter3Lib/ssee.h"
#include "SSConverter3Lib/ssqe.h"

#include "simpleSemaphore.h"

using namespace SpriteStudio;

struct SSConverter3Lib::Impl {
    std::shared_ptr<SSPJInformation> parseSSPJ(const std::filesystem::path &inputPath);
};

std::shared_ptr<SSPJInformation> SSConverter3Lib::Impl::parseSSPJ(const std::filesystem::path &inputPath) {
    int max_threads = std::thread::hardware_concurrency();
    printf("max_threads:%d\n", max_threads);

    SpriteStudio::SSPJ sspj;
    auto sspjInfo = sspj.parse(inputPath.string());

    // read Audio pack (SSSE)
    std::vector<std::future<std::shared_ptr<SpriteStudio::SSSEInformation>>> ssseFeatures;
    for (int i=0; i<sspjInfo->TableNameSSSE.size(); i++) {
        auto ssseStr = sspjInfo->TableNameSSSE[i];
        simpleSemaphore::getInstance()->aquire();
        ssseFeatures.push_back(
            std::async(std::launch::async, [ssseStr, &sspjInfo] {
                SpriteStudio::SSSE ssse;
                auto item = ssse.parse(ssseStr, sspjInfo);
                simpleSemaphore::getInstance()->release();
                return item;
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
        simpleSemaphore::getInstance()->aquire();
        fntFeatures.push_back(
            std::async(std::launch::async, [fntStr, &sspjInfo] {
                SpriteStudio::Fnt fnt;
                auto item = fnt.parse(fntStr, sspjInfo);
                simpleSemaphore::getInstance()->release();
                return item;
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
        simpleSemaphore::getInstance()->aquire();
        ssceFeatures.push_back(
            std::async(std::launch::async, [ssceStr, &sspjInfo] {
                SpriteStudio::SSCE ssce;
                auto item = ssce.parse(ssceStr, sspjInfo);
                simpleSemaphore::getInstance()->release();
                return item;
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
        simpleSemaphore::getInstance()->aquire();
        ssaeFeatures.push_back(
            std::async(std::launch::async, [ssaeStr, &sspjInfo] {
                SpriteStudio::SSAE ssae;
                auto item = ssae.parse(ssaeStr, sspjInfo);
                simpleSemaphore::getInstance()->release();
                return item;
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
        simpleSemaphore::getInstance()->aquire();
        ssqeFeatures.push_back(
            std::async(std::launch::async, [ssqeStr, &sspjInfo] {
                SpriteStudio::SSQE ssqe;
                auto item = ssqe.parse(ssqeStr, sspjInfo);
                simpleSemaphore::getInstance()->release();
                return item;
            })
        );
    }
    sspjInfo->TableInformationSSQE = {};
    for (auto& f : ssqeFeatures) {
        sspjInfo->TableInformationSSQE.push_back(f.get());
    }

    return sspjInfo;
}

std::unique_ptr<SSConverter3Lib::Impl> SSConverter3Lib::pImpl = std::make_unique<Impl>();

void SSConverter3Lib::convert(const std::string &input, const std::string &outputDir) {
    auto inputPath = std::filesystem::path(input);
    auto outputDirPath = outputDir.empty() ? inputPath.parent_path() : std::filesystem::path(outputDir);
    std::cout << "inputPath: " << inputPath << std::endl;
    std::cout << "outputDirPath: " << outputDirPath << std::endl;

    auto sspjInfo = SSConverter3Lib::pImpl->parseSSPJ(inputPath);
}
