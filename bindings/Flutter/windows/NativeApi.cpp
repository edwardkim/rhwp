#include "NativeApi.h"
#include "NativeReply.h"
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <mutex>

namespace rhwp::d3d {
namespace {
template <typename Function> void load(HMODULE library, const char* name, Function& output) {
  auto address = GetProcAddress(library, name);
  static_assert(sizeof(address) == sizeof(output), "unsupported Windows loader ABI");
  std::memcpy(&output, &address, sizeof(output));
  if (!output) throw std::runtime_error(std::string("native symbol unavailable: ") + name);
}
}
std::string NativeApi::read(char* raw) const {
  if (!raw) throw std::runtime_error("native returned null JSON");
  std::unique_ptr<char, decltype(free_string)> retained(raw, free_string);
  size_t length = 0;
  while (length <= 64 * 1024 && raw[length]) ++length;
  if (length > 64 * 1024) throw std::runtime_error("native JSON exceeds 64 KiB");
  return std::string(raw, length);
}
bool NativeApi::current(const std::string& frame_check) const {
  const auto text = read(check(reinterpret_cast<const uint8_t*>(frame_check.data()), frame_check.size()));
  NativeReply result(text);
  return result.field("ok").boolean() && result.field("current").boolean();
}
void NativeApi::release_snapshot(uint64_t snapshot) const noexcept {
  try {
    char payload[96];
    const auto length = std::snprintf(payload, sizeof(payload), "{\"op\":\"snapshotRelease\",\"snapshot\":%llu}",
        static_cast<unsigned long long>(snapshot));
    auto result = read(presentation(reinterpret_cast<const uint8_t*>(payload), size_t(length)));
    (void)result;
  } catch (...) { /* Process P89 quotas still bound a failed registry release. */ }
}
std::shared_ptr<NativeApi> load_api(const std::string& input) {
  const auto requested = std::filesystem::u8path(input);
  if (!requested.is_absolute()) throw std::runtime_error("native DLL path must be absolute");
  auto path = std::filesystem::canonical(requested);
  static auto* mutex = new std::mutex;
  static auto* pinned = new std::shared_ptr<NativeApi>;
  static auto* pinned_path = new std::filesystem::path;
  std::lock_guard<std::mutex> lock(*mutex);
  if (*pinned) {
    if (*pinned_path != path) throw std::runtime_error("one canonical native DLL path is allowed per process");
    return *pinned;
  }
  HMODULE library = LoadLibraryW(path.c_str());
  if (!library) throw std::runtime_error("native DLL could not be loaded");
  try {
    auto api = std::make_shared<NativeApi>(); api->library = library;
    load(library, "rhwp_d3d_capabilities", api->capabilities);
    load(library, "rhwp_d3d_presenter_create", api->create);
    load(library, "rhwp_d3d_presenter_render", api->render);
    load(library, "rhwp_d3d_presenter_request", api->request);
    load(library, "rhwp_d3d_frame_check", api->check);
    load(library, "rhwp_presentation_request", api->presentation);
    load(library, "rhwp_string_free", api->free_string);
    *pinned_path = std::move(path); *pinned = api; return api;
  } catch (...) { FreeLibrary(library); throw; }
}
}  // namespace rhwp::d3d
