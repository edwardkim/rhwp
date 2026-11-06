#include "rhwp_native_ffi.h"
#include "include/rhwp_native_reference/rhwp_d3d_plugin_c_api.h"
#include <flutter_texture_registrar.h>
#include <type_traits>

using Create = char* (*)(const IDXGIAdapter1*, const ID3D12Device*, const ID3D12CommandQueue*,
                        uint64_t, const uint8_t*, size_t);
using Render = char* (*)(uint64_t, const ID3D12Resource*, const uint8_t*, size_t);
static_assert(std::is_same_v<Create, decltype(&rhwp_d3d_presenter_create)>);
static_assert(std::is_same_v<Render, decltype(&rhwp_d3d_presenter_render)>);
namespace {
FlutterDesktopGpuSurfaceDescriptor descriptor{};
const FlutterDesktopGpuSurfaceDescriptor* populate(size_t, size_t, void*) { return &descriptor; }
void opened(void*) { /* Handle opened is not a consumer GPU fence. */ }
}
int main() {
  descriptor.struct_size = sizeof(descriptor);
  descriptor.handle = nullptr;
  descriptor.width = descriptor.visible_width = 1;
  descriptor.height = descriptor.visible_height = 1;
  descriptor.format = kFlutterDesktopPixelFormatBGRA8888;
  descriptor.release_callback = opened; descriptor.release_context = nullptr;
  FlutterDesktopTextureInfo texture{}; texture.type = kFlutterDesktopGpuSurfaceTexture;
  texture.gpu_surface_config.struct_size = sizeof(FlutterDesktopGpuSurfaceTextureConfig);
  texture.gpu_surface_config.type = kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle;
  texture.gpu_surface_config.callback = populate; texture.gpu_surface_config.user_data = nullptr;
  return texture.gpu_surface_config.callback(1, 1, nullptr)->width != 1;
}
