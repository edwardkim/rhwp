#include "Producer.h"
#include "NativeReply.h"
#include <algorithm>
#include <chrono>
#include <stdexcept>
#include <thread>

namespace rhwp::d3d {
namespace {
std::atomic<size_t> producer_count{0};
std::atomic<uint64_t> opened_count{0};
void descriptor_opened(void* context) noexcept {
  auto* pixels = static_cast<Pixels*>(context);
  pixels->opened_notifications.fetch_add(1, std::memory_order_relaxed);
  opened_count.fetch_add(1, std::memory_order_relaxed);
  // Opening transfers creator references only. No GPU fence/reuse is claimed.
}
struct NativeFailure : std::runtime_error {
  std::string code;
  NativeFailure(std::string code, const std::string& message) : std::runtime_error(message), code(std::move(code)) {}
};
void checked(HRESULT result, const char* message) { if (FAILED(result)) throw std::runtime_error(message); }
bool same_luid(const LUID& left, const LUID& right) { return left.HighPart == right.HighPart && left.LowPart == right.LowPart; }
void mark_frame(const std::shared_ptr<State>& state) {
  // Registrar access is serialized with plugin destruction. This public call
  // posts engine work; it does not wait for a GPU queue or document operation.
  std::lock_guard<std::mutex> lock(state->mutex);
  if (!state->closed && state->registrar && state->texture_id >= 0)
    FlutterDesktopTextureRegistrarMarkExternalTextureFrameAvailable(state->registrar, state->texture_id);
}
Stamp reply_stamp(const NativeReply& value) {
  return {value.field("documentId").integer(), value.field("revision").integer(),
      value.field("fontGeneration").integer(), value.field("viewGeneration").integer(),
      value.field("deviceGeneration").integer(), value.field("deviceScale").number()};
}
bool producer_proof(const std::string& text, const Job& job) {
  NativeReply result(text);
  if (!result.field("ok").boolean()) return false;
  auto frame = result.field("frame"); auto target = frame.field("target");
  return frame.field("producerGpuComplete").boolean() && frame.field("resourceCommon").boolean() &&
      !frame.field("consumerGpuComplete").boolean() && frame.field("wholeFrameReadbacks").integer() == 0 &&
      frame.field("producerFenceValue").integer() > 0 && reply_stamp(frame.field("stamp")) == job.stamp &&
      target.field("width").integer() == job.width && target.field("height").integer() == job.height &&
      target.field("deviceGeneration").integer() == job.stamp.device &&
      frame.field("pixelFormat").text() == "bgra8UnormPremultiplied" &&
      frame.field("colorSpace").text() == "srgb" && frame.field("alpha").text() == "premultiplied" &&
      frame.field("origin").text() == "topLeft";
}

struct Device {
  ComPtr<ID3D12Device> device12;
  ComPtr<ID3D12CommandQueue> queue12;
  ComPtr<ID3D11Device1> device11;
  ComPtr<ID3D11DeviceContext> context11;
  explicit Device(IDXGIAdapter1* adapter) {
    DXGI_ADAPTER_DESC1 expected{}; checked(adapter->GetDesc1(&expected), "engine adapter description failed");
    checked(D3D12CreateDevice(adapter, D3D_FEATURE_LEVEL_11_0, IID_PPV_ARGS(&device12)), "D3D12 device unavailable");
    if (!same_luid(expected.AdapterLuid, device12->GetAdapterLuid()) || device12->GetNodeCount() != 1)
      throw std::runtime_error("D3D12 device adapter LUID/node mismatch");
    D3D12_COMMAND_QUEUE_DESC queue{}; queue.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
    checked(device12->CreateCommandQueue(&queue, IID_PPV_ARGS(&queue12)), "D3D12 direct queue unavailable");
    ComPtr<ID3D11Device> base11;
    const D3D_FEATURE_LEVEL levels[] = {D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0};
    checked(D3D11CreateDevice(adapter, D3D_DRIVER_TYPE_UNKNOWN, nullptr, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        levels, 2, D3D11_SDK_VERSION, &base11, nullptr, &context11), "D3D11 bridge device unavailable");
    checked(base11.As(&device11), "D3D11.1 OpenSharedResource1 unavailable");
    ComPtr<IDXGIDevice> dxgi; ComPtr<IDXGIAdapter> actual;
    checked(device11.As(&dxgi), "D3D11 DXGI device unavailable");
    checked(dxgi->GetAdapter(&actual), "D3D11 adapter query failed");
    DXGI_ADAPTER_DESC desc{}; checked(actual->GetDesc(&desc), "D3D11 adapter description failed");
    if (!same_luid(desc.AdapterLuid, expected.AdapterLuid)) throw std::runtime_error("D3D11/D3D12 adapter LUID mismatch");
  }
  std::shared_ptr<Pixels> allocate(const std::shared_ptr<NativeApi>& api, const Job& job) {
    auto pixels = std::make_shared<Pixels>(); pixels->api = api;
    pixels->device12 = device12; pixels->queue12 = queue12;
    pixels->device11 = device11; pixels->context11 = context11;
    D3D12_RESOURCE_DESC desc{};
    desc.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D; desc.Width = job.width; desc.Height = job.height;
    desc.DepthOrArraySize = 1; desc.MipLevels = 1; desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
    desc.SampleDesc.Count = 1; desc.Layout = D3D12_TEXTURE_LAYOUT_UNKNOWN;
    desc.Flags = D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET | D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS;
    const auto allocation = device12->GetResourceAllocationInfo(0, 1, &desc);
    if (!allocation.SizeInBytes || allocation.SizeInBytes == UINT64_MAX || allocation.SizeInBytes > job.credit->bytes() / 2)
      throw std::runtime_error("D3D12 allocation exceeds the reserved two-surface frame budget");
    D3D12_HEAP_PROPERTIES heap{}; heap.Type = D3D12_HEAP_TYPE_DEFAULT;
    heap.CreationNodeMask = heap.VisibleNodeMask = 1;
    checked(device12->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_SHARED, &desc,
        D3D12_RESOURCE_STATE_COMMON, nullptr, IID_PPV_ARGS(&pixels->resource12)), "shared D3D12 target allocation failed");
    checked(device12->CreateSharedHandle(pixels->resource12.Get(), nullptr, GENERIC_ALL, nullptr,
        &pixels->nt_handle), "D3D12 shared NT handle creation failed");
    checked(device11->OpenSharedResource1(pixels->nt_handle, IID_PPV_ARGS(&pixels->opened11)),
        "D3D11 could not open the shared D3D12 resource");
    D3D11_TEXTURE2D_DESC opened{}; pixels->opened11->GetDesc(&opened);
    if (opened.Width != job.width || opened.Height != job.height || opened.Format != DXGI_FORMAT_B8G8R8A8_UNORM ||
        opened.ArraySize != 1 || opened.MipLevels != 1 || opened.SampleDesc.Count != 1 || opened.SampleDesc.Quality ||
        opened.Usage != D3D11_USAGE_DEFAULT || !(opened.BindFlags & D3D11_BIND_SHADER_RESOURCE))
      throw std::runtime_error("D3D11 opened shared target format/flags mismatch");
    D3D11_TEXTURE2D_DESC legacy = opened;
    legacy.BindFlags = D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE;
    legacy.CPUAccessFlags = 0; legacy.MiscFlags = D3D11_RESOURCE_MISC_SHARED;
    checked(device11->CreateTexture2D(&legacy, nullptr, &pixels->legacy11), "legacy D3D11 shared target allocation failed");
    ComPtr<IDXGIResource> resource; checked(pixels->legacy11.As(&resource), "legacy DXGI resource query failed");
    checked(resource->GetSharedHandle(&pixels->legacy_handle), "legacy DXGI shared handle unavailable");
    if (!pixels->legacy_handle) throw std::runtime_error("legacy shared handle is null");
    D3D11_QUERY_DESC query{D3D11_QUERY_EVENT, 0};
    checked(device11->CreateQuery(&query, &pixels->completion11), "D3D11 copy completion query unavailable");
    pixels->descriptor.struct_size = sizeof(FlutterDesktopGpuSurfaceDescriptor);
    pixels->descriptor.handle = pixels->legacy_handle;
    pixels->descriptor.width = pixels->descriptor.visible_width = job.width;
    pixels->descriptor.height = pixels->descriptor.visible_height = job.height;
    pixels->descriptor.format = kFlutterDesktopPixelFormatBGRA8888;
    pixels->descriptor.release_context = pixels.get();
    pixels->descriptor.release_callback = &descriptor_opened;
    return pixels;
  }
  void copy_and_wait(const std::shared_ptr<Pixels>& pixels) {
    context11->CopyResource(pixels->legacy11.Get(), pixels->opened11.Get());
    context11->End(pixels->completion11.Get()); context11->Flush();
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(5);
    for (;;) {
      BOOL completed = FALSE;
      const auto result = context11->GetData(pixels->completion11.Get(), &completed, sizeof(completed), 0);
      if (FAILED(device11->GetDeviceRemovedReason()))
        throw NativeFailure("deviceLost", "D3D11 device removal reported; frame quarantined");
      if (FAILED(result) || std::chrono::steady_clock::now() >= deadline)
        throw NativeFailure("submissionFailed", "D3D11 producer copy completion unknown; frame quarantined");
      if (result == S_OK && completed) return;
      std::this_thread::sleep_for(std::chrono::milliseconds(1));
    }
  }
};
}  // namespace

ImmutableFrames& frames() { static auto* value = new ImmutableFrames; return *value; }
uint64_t descriptor_opened_notifications() { return opened_count.load(std::memory_order_relaxed); }
void register_surface_imports(const std::shared_ptr<State>& state) {
  if (state->publication_mode == PublicationMode::ImmutableOwnershipTransfer)
    SurfaceImports::register_owner(state->generation);
}
void retire_surface_imports(const std::shared_ptr<State>& state) noexcept {
  try {
    if (state->publication_mode == PublicationMode::ImmutableOwnershipTransfer)
      SurfaceImports::unregistered(state->generation);
  } catch (...) { /* Failed CPU retirement remains bounded and pinned. */ }
}
std::shared_ptr<int> reserve_producer() {
  size_t current = producer_count.load();
  do { if (current >= 2) return {}; } while (!producer_count.compare_exchange_weak(current, current + 1));
  int* credit;
  try { credit = new int(0); } catch (...) { producer_count.fetch_sub(1); throw; }
  // shared_ptr invokes this deleter if its own control-block allocation fails.
  return std::shared_ptr<int>(credit, [](int* value) { delete value; producer_count.fetch_sub(1); });
}
bool State::accept(const std::shared_ptr<Job>& job) {
  std::shared_ptr<Job> previous; std::shared_ptr<Published> old_frame;
  {
    std::lock_guard<std::mutex> lock(mutex);
    if (closed || !ready || !api) return false;
    for (auto it = leases.begin(); it != leases.end();) if (it->second.expired()) it = leases.erase(it); else ++it;
    for (const auto id : job->ids) {
      std::shared_ptr<SnapshotLease> lease;
      auto found = leases.find(id); if (found != leases.end()) lease = found->second.lock();
      if (!lease) {
        if (leases.size() >= 32) return false;
        lease = std::make_shared<SnapshotLease>(); lease->id = id; lease->api = api; leases[id] = lease;
      }
      job->snapshots.push_back(std::move(lease));
    }
    auto receipt = job->ids;
    job->serial = gate.accept(job->stamp);
    if (!job->serial) return false;
    accepted_stamp = job->stamp; accepted_serial = job->serial; accepted_ids = std::move(receipt);
    for (const auto& lease : job->snapshots) lease->transferred = true;
    previous = std::move(pending); old_frame = std::move(current); pending = job; error.clear(); error_code.clear(); device_failure = false;
  }
  changed.notify_one(); return true;
}
void State::close() {
  std::shared_ptr<Job> previous; std::shared_ptr<Published> old_frame;
  {
    std::lock_guard<std::mutex> lock(mutex);
    closed = true; ready = false; gate.close(); previous = std::move(pending); old_frame = std::move(current);
  }
  changed.notify_one();
}
void State::fail(const std::string& message, const std::string& code) {
  std::lock_guard<std::mutex> lock(mutex); error = message.substr(0, 2048); error_code = code.substr(0, 64);
  device_failure = code == "deviceLost";
}

void run_producer(const std::shared_ptr<State>& state) noexcept {
  std::shared_ptr<NativeApi> api;
  uint64_t presenter = 0;
  try {
    api = load_api(state->path);
    const auto capabilities = api->read(api->capabilities());
    NativeReply caps(capabilities);
    if (!caps.field("ok").boolean() || !caps.field("available").boolean() || caps.field("abiVersion").integer() != 1)
      throw std::runtime_error("native-skia-d3d is unavailable; select the CPU reference renderer");
    Device device(state->adapter.Get());
    const auto created = api->read(api->create(state->adapter.Get(), device.device12.Get(), device.queue12.Get(),
        state->generation, nullptr, 0));
    NativeReply creation(created);
    if (!creation.field("ok").boolean()) throw std::runtime_error("native D3D presenter creation failed");
    presenter = creation.field("presenter").integer();
    if (!presenter) throw std::runtime_error("native D3D presenter handle is zero");
    {
      std::lock_guard<std::mutex> lock(state->mutex); state->api = api; state->ready = !state->closed;
    }
    for (;;) {
      std::shared_ptr<Job> job;
      {
        std::unique_lock<std::mutex> lock(state->mutex);
        state->changed.wait(lock, [&] { return state->closed || state->pending; });
        if (state->closed) break;
        job = std::move(state->pending); state->in_flight = true;
      }
      bool completed = false;
      try {
        if (!api->current(job->check_json)) {
          std::lock_guard<std::mutex> lock(state->mutex); ++state->stale_drops;
        } else {
          auto pixels = device.allocate(api, *job);
          if (!job->credit->begin_gpu(pixels)) throw std::runtime_error("frame resource reservation is invalid");
          const auto rendered = api->read(api->render(presenter, pixels->resource12.Get(),
              reinterpret_cast<const uint8_t*>(job->render_json.data()), job->render_json.size()));
          NativeReply reply(rendered);
          if (!reply.field("ok").boolean()) {
            const bool stale = reply.field("code").text() == "staleFrame" &&
                reply.field("unpublishedTargetSafeToDiscard").boolean();
            if (stale) { completed = true; job->credit->producer_complete(); }
            else throw NativeFailure(reply.field("code").text(), reply.field("error").text());
          } else {
            if (!producer_proof(rendered, *job)) throw std::runtime_error("native D3D frame proof/metadata mismatch");
            completed = true;  // D3D12 is complete; D3D11 copy has not started.
            if (api->current(job->check_json)) {
              completed = false; device.copy_and_wait(pixels); completed = true;
              job->credit->producer_complete();
              if (api->current(job->check_json)) {
                auto frame = std::make_shared<Published>(); frame->job = job; frame->pixels = pixels;
                {
                  std::lock_guard<std::mutex> lock(state->mutex);
                  if (state->gate.current(job->serial, job->stamp)) {
                    if (!job->credit->publish_immutable()) throw std::runtime_error("frame was not producer-complete");
                    state->current = std::move(frame); state->error.clear(); state->error_code.clear(); state->device_failure = false;
                  } else ++state->stale_drops;
                }
                mark_frame(state);
              }
            } else job->credit->producer_complete();
          }
        }
      } catch (const NativeFailure& error) {
        if (completed) job->credit->producer_complete();
        state->fail(error.what(), error.code); state->close();
      } catch (const std::exception& error) {
        if (completed) job->credit->producer_complete();
        state->fail(error.what(), "resourceError"); state->close();
      }
      {
        std::lock_guard<std::mutex> lock(state->mutex); state->in_flight = false;
      }
    }
  } catch (const std::exception& error) { try { state->fail(error.what()); } catch (...) {} }
  try {
    if (api && presenter) {
      const std::string release = "{\"op\":\"release\"}";
      const auto text = api->read(api->request(presenter, reinterpret_cast<const uint8_t*>(release.data()), release.size()));
      if (!NativeReply(text).field("ok").boolean()) {
        const std::string abandon = "{\"op\":\"abandon\"}";
        (void)api->read(api->request(presenter, reinterpret_cast<const uint8_t*>(abandon.data()), abandon.size()));
      }
    }
  } catch (...) { /* Native owner TLS quarantines any unproven D3D12 context. */ }
  {
    std::lock_guard<std::mutex> lock(state->mutex);
    state->ready = false; state->finished = true; state->producer_slot.reset();
  }
}

const FlutterDesktopGpuSurfaceDescriptor* surface_callback(size_t, size_t, void* user) {
  auto state = static_cast<Registration*>(user)->state;
  try {
    const bool transfer = state->publication_mode == PublicationMode::ImmutableOwnershipTransfer;
    // The next callback on this same raster OS thread proves prior descriptor
    // CPU reads ended. Closing alone does not retire that descriptor storage.
    auto callback = transfer ? SurfaceImports::begin_callback(state->generation) : SurfaceImports::Callback{};
    if (transfer && !callback) return nullptr;
    std::shared_ptr<Published> frame;
    {
      std::lock_guard<std::mutex> lock(state->mutex);
      frame = state->current;
      if (!frame || !state->gate.current(frame->job->serial, frame->job->stamp)) return nullptr;
    }
    if (!frame->pixels->api->current(frame->job->check_json)) {
      state->fail("source changed; hide Texture and refresh owner snapshots", "staleFrame"); return nullptr;
    }
    std::lock_guard<std::mutex> lock(state->mutex);
    if (!state->gate.current(frame->job->serial, frame->job->stamp)) return nullptr;
    if (!transfer) return &frame->pixels->descriptor;
    const auto* descriptor = SurfaceImports::offer(state->generation, frame->pixels->descriptor,
        frame, frame->job->credit, &descriptor_opened, frame->pixels.get());
    if (!descriptor) throw std::runtime_error("bounded surface import ticket capacity exceeded");
    return descriptor;
  } catch (const std::exception& error) {
    if (state->publication_mode == PublicationMode::ImmutableOwnershipTransfer) {
      state->fail(error.what(), "consumerImportError"); state->close();
    }
    return nullptr;
  } catch (...) {
    if (state->publication_mode == PublicationMode::ImmutableOwnershipTransfer) {
      state->fail("surface import callback failed", "consumerImportError"); state->close();
    }
    return nullptr;
  }
}
}  // namespace rhwp::d3d
