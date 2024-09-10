#include "simpleSemaphore.h"

using namespace SpriteStudio;

std::shared_ptr<simpleSemaphore> simpleSemaphore::instance;

void simpleSemaphore::aquire() {
    std::unique_lock<std::mutex> lock(m);
    cv.wait(lock, [this] { return runningTask < maxThreads; });
    runningTask++;
}

void simpleSemaphore::release() {
    std::unique_lock<std::mutex> lock(m);
    runningTask--;
    cv.notify_one();
}


