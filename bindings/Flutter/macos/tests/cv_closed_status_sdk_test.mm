// Actual Apple SDK/Foundation regression. This logical-credit check performs no
// Metal rendering and must not be reported as GPU/compositor qualification.
#include "../Classes/RhwpMetalPlugin.mm"
#include <iostream>

int main() {
  @autoreleasepool {
    auto state = std::make_shared<State>(1,"unused",nullptr,true);
    state->close();
    {std::lock_guard<std::mutex> lock(state->mutex);state->finished=true;}
    CVCloseReceipt cached{7,1,CVStatus(7,state,false)};
    if ([cached.status[@"limits"][@"outstandingFrames"] unsignedLongLongValue] != 0) return 1;
    auto frame = Credits().reserve(16*kMib);
    if (!frame) return 2;
    NSDictionary* reply = CVClosedStatus(cached);
    if ([reply[@"textureId"] unsignedLongLongValue] != 7 ||
        [reply[@"deviceGeneration"] unsignedLongLongValue] != 1 ||
        ![reply[@"ownerFinished"] boolValue] || [reply[@"registered"] boolValue] ||
        [reply[@"limits"][@"outstandingFrames"] unsignedLongLongValue] != 1 ||
        [reply[@"limits"][@"reservedTargetBytes"] unsignedLongLongValue] != 16*kMib ||
        [reply[@"limits"][@"consumerGpuCompletionVerified"] boolValue]) return 3;
    // Refreshing a response must not mutate cached completion metadata.
    if ([cached.status[@"limits"][@"outstandingFrames"] unsignedLongLongValue] != 0) return 4;
    frame.reset();
    if ([CVClosedStatus(cached)[@"limits"][@"outstandingFrames"] unsignedLongLongValue] != 0) return 5;
    std::cout << "CV terminal status: live global limits, immutable completion metadata PASS\n";
  }
}
