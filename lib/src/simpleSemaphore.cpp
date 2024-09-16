#include "simpleSemaphore.h"

using namespace SpriteStudio;

std::shared_ptr<simpleSemaphore> simpleSemaphore::instance;

bool simpleSemaphore::acquire() {
    std::unique_lock<std::mutex> lock(m);
    if (!cv.wait_for(lock, timeout, [this] { return runningTask < maxThreads; })) {
        return false;
    }
    runningTask++;
    return true;
}

void simpleSemaphore::release() {
    std::unique_lock<std::mutex> lock(m);
    runningTask--;
    cv.notify_one();
}


