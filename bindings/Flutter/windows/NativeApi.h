#ifndef RHWP_WINDOWS_NATIVE_API_H_
#define RHWP_WINDOWS_NATIVE_API_H_
#include "rhwp_native_ffi.h"
#include <windows.h>
#include <memory>
#include <string>

namespace rhwp::d3d {
struct NativeApi {
  HMODULE library = nullptr;
  decltype(&rhwp_d3d_capabilities) capabilities = nullptr;
  decltype(&rhwp_d3d_presenter_create) create = nullptr;
  decltype(&rhwp_d3d_presenter_render) render = nullptr;
  decltype(&rhwp_d3d_presenter_request) request = nullptr;
  decltype(&rhwp_d3d_frame_check) check = nullptr;
  decltype(&rhwp_presentation_request) presentation = nullptr;
  decltype(&rhwp_string_free) free_string = nullptr;
  std::string read(char* raw) const;
  bool current(const std::string& frame_check) const;
  void release_snapshot(uint64_t snapshot) const noexcept;
};
// One canonical DLL instance is process-pinned, matching P97's owner dispatcher.
std::shared_ptr<NativeApi> load_api(const std::string& canonical_path);
}  // namespace rhwp::d3d
#endif
