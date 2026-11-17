#ifndef RHWP_METAL_NATIVE_API_HPP_
#define RHWP_METAL_NATIVE_API_HPP_

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <dlfcn.h>
#include <memory>
#include <mutex>
#include <string>

namespace rhwp::metal {
struct MetalApiSymbols {
  using Capabilities = char* (*)();
  using Create = char* (*)(const void*, const void*, uint64_t, const uint8_t*, size_t);
  using Render = char* (*)(uint64_t, const void*, const uint8_t*, size_t);
  using Request = char* (*)(uint64_t, const uint8_t*, size_t);
  using Presentation = char* (*)(const uint8_t*, size_t);
  using Free = void (*)(char*);

  Capabilities capabilities = nullptr;
  Create create = nullptr;
  Render render = nullptr;
  Request request = nullptr;
  Presentation presentation = nullptr;
  Presentation checkFrame = nullptr;
  Free freeString = nullptr;
};

template <class Function>
bool LoadMetalSymbol(void* library, const char* name, Function& output) {
  void* address = dlsym(library, name);
  static_assert(sizeof(address) == sizeof(output), "unsupported loader ABI");
  std::memcpy(&output, &address, sizeof(output));
  return output != nullptr;
}

template <class Api>
struct MetalApiLoad {
  std::shared_ptr<Api> api;
  const char* code = nullptr;
  const char* message = nullptr;
};

template <class Api>
MetalApiLoad<Api> LoadMetalApi(const std::string& path) {
  // P89 snapshot IDs cannot cross native module instances.
  static std::mutex mutex;
  static std::string pinnedPath;
  static std::shared_ptr<Api> pinned;
  std::lock_guard<std::mutex> lock(mutex);
  if (pinned) {
    if (pinnedPath == path) return {pinned};
    return {{}, "nativeTextureError", "One canonical Native dylib is allowed per process."};
  }
  auto closeLibrary = [](void* library) { dlclose(library); };
  std::unique_ptr<void, decltype(closeLibrary)> library(
      dlopen(path.c_str(), RTLD_NOW | RTLD_LOCAL), closeLibrary);
  if (!library)
    return {{}, "nativeTextureError", "The Native dylib could not be opened."};

  auto api = std::make_shared<Api>();
  const auto handle = library.get();
  // Owners need these exports even when Metal is unavailable.
  if (!LoadMetalSymbol(handle, "rhwp_presentation_request", api->presentation) ||
      !LoadMetalSymbol(handle, "rhwp_string_free", api->freeString))
    return {{}, "nativeTextureError", "The Native dylib is missing required owner exports."};
  if (!LoadMetalSymbol(handle, "rhwp_metal_capabilities", api->capabilities) ||
      !LoadMetalSymbol(handle, "rhwp_metal_presenter_create", api->create) ||
      !LoadMetalSymbol(handle, "rhwp_metal_presenter_render", api->render) ||
      !LoadMetalSymbol(handle, "rhwp_metal_presenter_request", api->request) ||
      !LoadMetalSymbol(handle, "rhwp_metal_frame_check", api->checkFrame))
    return {{}, "unsupported", "The Native Metal ABI is unavailable."};

  pinnedPath = path;
  pinned = api;
  (void)library.release();  // Keep the shared Native and TLS module loaded.
  return {pinned};
}
}  // namespace rhwp::metal

#endif
