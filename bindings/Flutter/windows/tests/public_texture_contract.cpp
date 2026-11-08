#include "rhwp_native_ffi.h"
#include "SurfaceImportTickets.h"
#include <rhwp_native_reference/rhwp_d3d_plugin.h>
#include <flutter_texture_registrar.h>
#include <type_traits>

using Create = char* (*)(const IDXGIAdapter1*, const ID3D12Device*, const ID3D12CommandQueue*,
                        uint64_t, const uint8_t*, size_t);
using Render = char* (*)(uint64_t, const ID3D12Resource*, const uint8_t*, size_t);
static_assert(std::is_same_v<Create, decltype(&rhwp_d3d_presenter_create)>);
static_assert(std::is_same_v<Render, decltype(&rhwp_d3d_presenter_render)>);
using Registration = void (*)(FlutterDesktopPluginRegistrarRef);
static_assert(std::is_same_v<Registration, decltype(&RhwpD3dPluginRegisterWithRegistrar)>);
namespace {
FlutterDesktopGpuSurfaceDescriptor descriptor{};
const FlutterDesktopGpuSurfaceDescriptor* populate(size_t, size_t, void*) { return &descriptor; }
void opened(void*) { /* Handle opened is not a consumer GPU fence. */ }
}
int main() {
  descriptor.struct_size = sizeof(descriptor);
  descriptor.handle = reinterpret_cast<void*>(uintptr_t(1));
  descriptor.width = descriptor.visible_width = 1;
  descriptor.height = descriptor.visible_height = 1;
  descriptor.format = kFlutterDesktopPixelFormatBGRA8888;
  descriptor.release_callback = opened; descriptor.release_context = nullptr;
  FlutterDesktopTextureInfo texture{}; texture.type = kFlutterDesktopGpuSurfaceTexture;
  texture.gpu_surface_config.struct_size = sizeof(FlutterDesktopGpuSurfaceTextureConfig);
  texture.gpu_surface_config.type = kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle;
  texture.gpu_surface_config.callback = populate; texture.gpu_surface_config.user_data = nullptr;
  if (texture.gpu_surface_config.callback(1, 1, nullptr)->width != 1) return 1;
  using Imports = rhwp::d3d::SurfaceImportTickets<FlutterDesktopGpuSurfaceDescriptor>;
  rhwp::d3d::ImmutableFrames pool;
  auto credit = pool.reserve(128 * 1024, rhwp::d3d::PublicationMode::ImmutableOwnershipTransfer);
  auto frame = std::make_shared<int>(1);
  if (!credit || !credit->begin_gpu(frame) || !credit->producer_complete() || !credit->publish_immutable()) return 2;
  Imports::register_owner(1);
  const FlutterDesktopGpuSurfaceDescriptor* borrowed;
  {
    auto callback = Imports::begin_callback(1);
    borrowed = Imports::offer(1, descriptor, frame, credit);
    if (!callback || !borrowed) return 3;
  }
  borrowed->release_callback(borrowed->release_context);
  // Match the real engine ordering: dimensions are read after release_callback.
  if (borrowed->visible_width != 1 || borrowed->visible_height != 1) return 4;
  Imports::unregistered(1); credit.reset(); frame.reset();
  return pool.stats().outstanding != 0;
}
