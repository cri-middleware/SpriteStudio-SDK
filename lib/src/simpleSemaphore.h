#pragma once

#include <thread>
#include <mutex>
#include <condition_variable>

namespace SpriteStudio {
    class simpleSemaphore {
    public:
        static std::shared_ptr<simpleSemaphore> getInstance() {
            static std::once_flag onceFlag;
            std::call_once(onceFlag, [] {
                instance = std::make_shared<simpleSemaphore>();
            });
            return instance;
        }

        bool acquire();
        void release();

    private:
        const int maxThreads = std::thread::hardware_concurrency();
        std::condition_variable cv;
        int runningTask{0};
        std::mutex m;
        std::chrono::milliseconds timeout{std::chrono::milliseconds::max()};

        static std::shared_ptr<simpleSemaphore> instance;
    };
}
