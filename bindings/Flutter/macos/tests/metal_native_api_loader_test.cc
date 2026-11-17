// Exercise the production loader with real shared libraries and no GPU calls.
#include "../Classes/MetalNativeApi.hpp"
#include <iostream>
#include <string_view>

int main(int argc, char** argv) {
  if (argc != 5) return 1;
  using rhwp::metal::LoadMetalApi;
  using rhwp::metal::MetalApiSymbols;
  const auto matches = [](const auto& loaded, std::string_view code) {
    return !loaded.api && loaded.code && loaded.message && loaded.code == code;
  };
  if (!matches(LoadMetalApi<MetalApiSymbols>(argv[1]), "nativeTextureError")) {
    std::cerr << "Missing dylib did not preserve a loader error.\n";
    return 2;
  }
  if (!matches(LoadMetalApi<MetalApiSymbols>(argv[2]), "nativeTextureError")) {
    std::cerr << "Missing required owner exports were treated as optional.\n";
    return 3;
  }
  if (!matches(LoadMetalApi<MetalApiSymbols>(argv[3]), "unsupported")) {
    std::cerr << "Missing optional Metal exports did not allow CPU fallback.\n";
    return 4;
  }
  const auto loaded = LoadMetalApi<MetalApiSymbols>(argv[4]);
  const auto repeated = LoadMetalApi<MetalApiSymbols>(argv[4]);
  if (!loaded.api || loaded.code || loaded.message || repeated.api != loaded.api ||
      !loaded.api->presentation || !loaded.api->freeString || !loaded.api->capabilities ||
      !loaded.api->create || !loaded.api->render || !loaded.api->request || !loaded.api->checkFrame) {
    std::cerr << "A complete Native module was not process-pinned with its ABI.\n";
    return 5;
  }
  if (!matches(LoadMetalApi<MetalApiSymbols>(argv[3]), "nativeTextureError")) {
    std::cerr << "A second module path did not preserve a pinning error.\n";
    return 6;
  }
}
