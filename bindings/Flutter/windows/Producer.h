#ifndef RHWP_WINDOWS_D3D_PRODUCER_H_
#define RHWP_WINDOWS_D3D_PRODUCER_H_
#include "FrameLifetime.h"
#include "SurfaceImportTickets.h"
#include "NativeApi.h"
#include <flutter_texture_registrar.h>
#include <d3d11_1.h>
#include <d3d12.h>
#include <dxgi1_2.h>
#include <wrl/client.h>
#include <atomic>
#include <condition_variable>
#include <map>
#include <vector>

namespace rhwp::d3d {
using Microsoft::WRL::ComPtr;
struct SnapshotLease {
  std::shared_ptr<NativeApi> api;
  uint64_t id = 0;
  bool transferred = false;
  ~SnapshotLease() { if (transferred) api->release_snapshot(id); }
};
struct Job {
  uint64_t serial = 0;
  Stamp stamp;
  uint32_t width = 0, height = 0;
  std::string render_json, check_json;
  std::vector<uint64_t> ids;
  std::vector<std::shared_ptr<SnapshotLease>> snapshots;
  std::shared_ptr<ImmutableFrames::Reservation> credit;
};
struct Pixels {
  std::shared_ptr<NativeApi> api;
  ComPtr<ID3D12Device> device12;
  ComPtr<ID3D12CommandQueue> queue12;
  ComPtr<ID3D12Resource> resource12;
  ComPtr<ID3D11Device1> device11;
  ComPtr<ID3D11DeviceContext> context11;
  ComPtr<ID3D11Texture2D> opened11, legacy11;
  ComPtr<ID3D11Query> completion11;
  HANDLE nt_handle = nullptr;
  HANDLE legacy_handle = nullptr;  // Borrowed GetSharedHandle value: never CloseHandle.
  FlutterDesktopGpuSurfaceDescriptor descriptor{};
  std::atomic<uint64_t> opened_notifications{0};
  ~Pixels() { if (nt_handle) CloseHandle(nt_handle); }
};
struct Published { std::shared_ptr<Job> job; std::shared_ptr<Pixels> pixels; };
using SurfaceImports = SurfaceImportTickets<FlutterDesktopGpuSurfaceDescriptor>;
struct State : std::enable_shared_from_this<State> {
  State(uint64_t generation, std::string path, ComPtr<IDXGIAdapter1> adapter,
        FlutterDesktopTextureRegistrarRef registrar, std::shared_ptr<int> producer_slot,
        PublicationMode publication_mode = PublicationMode::LegacyImmutable, bool close_ack_required = false)
      : generation(generation), path(std::move(path)), adapter(std::move(adapter)),
        registrar(registrar), producer_slot(std::move(producer_slot)), gate(generation),
        publication_mode(publication_mode), close_ack_required(close_ack_required) {}
  std::mutex mutex;
  std::condition_variable changed;
  const uint64_t generation;
  const std::string path;
  ComPtr<IDXGIAdapter1> adapter;
  FlutterDesktopTextureRegistrarRef registrar;
  std::shared_ptr<int> producer_slot;
  GenerationGate gate;
  const PublicationMode publication_mode;
  const bool close_ack_required;
  bool ready = false, closed = false, finished = false, in_flight = false, unregistered = false;
  int64_t texture_id = -1;
  std::string error;
  std::string error_code;
  bool device_failure = false;
  std::shared_ptr<NativeApi> api;
  std::shared_ptr<Job> pending;
  std::shared_ptr<Published> current;
  std::map<uint64_t, std::weak_ptr<SnapshotLease>> leases;
  Stamp accepted_stamp;
  std::vector<uint64_t> accepted_ids;
  uint64_t accepted_serial = 0, stale_drops = 0;
  bool accept(const std::shared_ptr<Job>& job);
  void close();
  void fail(const std::string& message, const std::string& code = "backendUnavailable");
};
ImmutableFrames& frames();
std::shared_ptr<int> reserve_producer();
uint64_t descriptor_opened_notifications();
void register_surface_imports(const std::shared_ptr<State>& state);
void retire_surface_imports(const std::shared_ptr<State>& state) noexcept;
void run_producer(const std::shared_ptr<State>& state) noexcept;
const FlutterDesktopGpuSurfaceDescriptor* surface_callback(size_t, size_t, void* user);
struct Registration { std::shared_ptr<State> state; std::shared_ptr<int> credit; };
}  // namespace rhwp::d3d
#endif
