#include "include/rhwp_native_reference/rhwp_d3d_plugin_c_api.h"
#include "Producer.h"
#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>
#include <flutter/standard_method_codec.h>
#include <cmath>
#include <filesystem>
#include <iomanip>
#include <locale>
#include <set>
#include <sstream>
#include <thread>

namespace rhwp::d3d {
namespace {
using Value = flutter::EncodableValue;
using Map = flutter::EncodableMap;
using List = flutter::EncodableList;
struct MethodError : std::runtime_error {
  std::string code;
  MethodError(std::string code, const char* message) : std::runtime_error(message), code(std::move(code)) {}
};
std::atomic<size_t> registration_count{0};
std::atomic<uint64_t> next_generation{1};
std::shared_ptr<int> reserve_registration() {
  size_t count = registration_count.load();
  do { if (count >= 2) return {}; } while (!registration_count.compare_exchange_weak(count, count + 1));
  int* credit;
  try { credit = new int(0); } catch (...) { registration_count.fetch_sub(1); throw; }
  return std::shared_ptr<int>(credit, [](int* value) { delete value; registration_count.fetch_sub(1); });
}
const Map& map(const Value* value) {
  if (!value || !std::holds_alternative<Map>(*value)) throw MethodError("invalidArgument", "expected argument map");
  return std::get<Map>(*value);
}
const Value& field(const Map& values, const char* name) {
  auto found = values.find(Value(name));
  if (found == values.end()) throw MethodError("invalidArgument", "missing argument field");
  return found->second;
}
uint64_t integer(const Value& value) {
  int64_t result;
  if (auto* number = std::get_if<int64_t>(&value)) result = *number;
  else if (auto* number = std::get_if<int32_t>(&value)) result = *number;
  else throw MethodError("invalidArgument", "expected integer");
  if (result < 0) throw MethodError("invalidArgument", "negative integer");
  return uint64_t(result);
}
uint64_t integer(const Map& values, const char* name) { return integer(field(values, name)); }
uint32_t integer32(const Map& values, const char* name) {
  auto value = integer(values, name);
  if (value > UINT32_MAX) throw MethodError("invalidArgument", "integer exceeds u32");
  return uint32_t(value);
}
double number(const Map& values, const char* name) {
  const auto& value = field(values, name);
  double result;
  if (const auto* real = std::get_if<double>(&value)) result = *real;
  else if (const auto* whole = std::get_if<int64_t>(&value)) result = double(*whole);
  else if (const auto* whole = std::get_if<int32_t>(&value)) result = double(*whole);
  else throw MethodError("invalidArgument", "expected numeric value");
  if (!std::isfinite(result)) throw MethodError("invalidArgument", "nonfinite numeric value");
  return result;
}
Value unsigned_value(uint64_t value) { return Value(int64_t(value)); }
Map stamp_value(const Stamp& stamp) {
  return {{Value("documentId"), unsigned_value(stamp.document)}, {Value("revision"), unsigned_value(stamp.revision)},
      {Value("fontGeneration"), unsigned_value(stamp.font)}, {Value("viewGeneration"), unsigned_value(stamp.view)},
      {Value("deviceGeneration"), unsigned_value(stamp.device)}, {Value("deviceScale"), Value(stamp.scale)}};
}
std::string stamp_json(const Stamp& stamp) {
  std::ostringstream out; out.imbue(std::locale::classic()); out << std::setprecision(17);
  out << "{\"documentId\":" << stamp.document << ",\"revision\":" << stamp.revision << ",\"fontGeneration\":" << stamp.font
      << ",\"viewGeneration\":" << stamp.view << ",\"deviceGeneration\":" << stamp.device << ",\"deviceScale\":" << stamp.scale << '}';
  return out.str();
}
Value limits() {
  const auto stats = frames().stats();
  return Value(Map{{Value("consumerRetirement"), Value(false)}, {Value("continuousPresentationSupported"), Value(false)},
      {Value("cpuFallbackRequiredWhenSaturated"), Value(true)}, {Value("outstandingFrames"), unsigned_value(stats.outstanding)},
      {Value("immutableFrames"), unsigned_value(stats.immutable)}, {Value("quarantinedFrames"), unsigned_value(stats.quarantined)},
      {Value("pendingRetireFrames"), unsigned_value(stats.immutable + stats.quarantined)},
      {Value("reservedTargetBytes"), unsigned_value(stats.bytes)}, {Value("maxFrames"), Value(4)},
      {Value("maxTargetBytes"), Value(int64_t(64 * 1024 * 1024))}, {Value("maxSingleFrameBytes"), Value(16 * 1024 * 1024)},
      {Value("descriptorOpenedNotifications"), unsigned_value(descriptor_opened_notifications())},
      {Value("descriptorReleaseIsGpuComplete"), Value(false)}, {Value("wholeFrameReadbacks"), Value(0)}});
}
Value status(const std::shared_ptr<State>& state) {
  std::lock_guard<std::mutex> lock(state->mutex);
  List ids; for (auto id : state->accepted_ids) ids.emplace_back(unsigned_value(id));
  const bool has_frame = state->current && state->error.empty() &&
      state->gate.current(state->current->job->serial, state->current->job->stamp);
  Map value{{Value("textureId"), Value(state->texture_id)}, {Value("deviceGeneration"), unsigned_value(state->generation)},
      {Value("ready"), Value(state->ready && !state->closed)}, {Value("closed"), Value(state->closed)},
      {Value("ownerFinished"), Value(state->finished)}, {Value("inFlight"), Value(state->in_flight)},
      {Value("unregistered"), Value(state->unregistered)}, {Value("error"), Value(state->error)},
      {Value("errorCode"), Value(state->error_code)}, {Value("deviceFailure"), Value(state->device_failure)},
      {Value("acceptedSerial"), unsigned_value(state->accepted_serial)}, {Value("acceptedStamp"), Value(stamp_value(state->accepted_stamp))},
      {Value("acceptedSnapshots"), Value(ids)}, {Value("staleDrops"), unsigned_value(state->stale_drops)},
      {Value("hasFrame"), Value(has_frame)},
      {Value("renderedSerial"), unsigned_value(has_frame ? state->current->job->serial : 0)}, {Value("limits"), limits()}};
  value[Value("renderedStamp")] = has_frame ? Value(stamp_value(state->current->job->stamp)) : Value();
  return Value(std::move(value));
}
std::shared_ptr<Job> parse_job(const Map& args) {
  auto job = std::make_shared<Job>();
  const auto& source = map(&field(args, "stamp"));
  job->stamp = {integer(source, "documentId"), integer(source, "revision"), integer(source, "fontGeneration"),
      integer(source, "viewGeneration"), integer(source, "deviceGeneration"), number(source, "deviceScale")};
  if (!job->stamp.valid()) throw MethodError("invalidArgument", "invalid source/view/device stamp");
  job->width = integer32(args, "width"); job->height = integer32(args, "height");
  const auto charge = frame_charge(job->width, job->height);
  if (!charge) throw MethodError("backpressure", "two-surface viewport exceeds the immutable frame budget; use CPU");
  const auto* pages = std::get_if<List>(&field(args, "placements"));
  if (!pages || pages->empty() || pages->size() > 8) throw MethodError("invalidArgument", "expected 1..8 placements");
  std::ostringstream out; out.imbue(std::locale::classic()); out << std::setprecision(17);
  out << "{\"stamp\":" << stamp_json(job->stamp) << ",\"target\":{\"width\":" << job->width
      << ",\"height\":" << job->height << ",\"deviceGeneration\":" << job->stamp.device
      << "},\"clearColor\":" << integer32(args, "clearColor") << ",\"placements\":[";
  std::set<uint64_t> snapshots;
  bool first = true;
  for (const auto& value : *pages) {
    const auto& page = map(&value);
    const auto snapshot = integer(page, "snapshot");
    const auto scale = number(page, "scale"), x = number(page, "originX"), y = number(page, "originY");
    if (!snapshot || scale <= 0 || scale > 16 || static_cast<float>(scale) <= 0 || std::abs(x) > 16777216 || std::abs(y) > 16777216)
      throw MethodError("invalidArgument", "placement is outside its bounds");
    if (!first) out << ','; first = false;
    out << "{\"snapshot\":" << snapshot << ",\"scale\":" << scale << ",\"originX\":" << x << ",\"originY\":" << y
        << ",\"backgroundColor\":" << integer32(page, "backgroundColor") << '}';
    snapshots.insert(snapshot);
  }
  out << "]}"; job->render_json = out.str();
  job->ids.assign(snapshots.begin(), snapshots.end());
  std::ostringstream check; check << "{\"stamp\":" << stamp_json(job->stamp) << ",\"snapshots\":[";
  first = true;
  for (const auto id : job->ids) { if (!first) check << ','; first = false; check << id; }
  check << "]}"; job->check_json = check.str();
  if (job->render_json.size() > 64 * 1024 || job->check_json.size() > 64 * 1024)
    throw MethodError("invalidArgument", "request metadata exceeds 64 KiB");
  job->credit = frames().reserve(charge);
  if (!job->credit) throw MethodError("backpressure", "immutable GPU frame credits are exhausted; use CPU");
  return job;
}

class RhwpD3dPlugin final : public flutter::Plugin {
 public:
  explicit RhwpD3dPlugin(flutter::PluginRegistrarWindows* registrar) : registrar_(registrar),
      texture_registrar_(FlutterDesktopRegistrarGetTextureRegistrar(registrar->registrar())) {
    channel_ = std::make_unique<flutter::MethodChannel<Value>>(registrar->messenger(), "rhwp_native/texture_windows",
        &flutter::StandardMethodCodec::GetInstance());
    channel_->SetMethodCallHandler([this](const auto& call, auto result) {
      try { result->Success(invoke(call.method_name(), map(call.arguments()))); }
      catch (const MethodError& error) { result->Error(error.code, error.what(), limits()); }
      catch (const std::exception& error) { result->Error("nativeError", error.what(), limits()); }
    });
  }
  ~RhwpD3dPlugin() override {
    channel_->SetMethodCallHandler(nullptr);
    for (auto& item : textures_) close(item.second);
  }
 private:
  struct Record { std::shared_ptr<State> state; Registration* registration; };
  flutter::PluginRegistrarWindows* registrar_;
  FlutterDesktopTextureRegistrarRef texture_registrar_;
  std::unique_ptr<flutter::MethodChannel<Value>> channel_;
  std::map<int64_t, Record> textures_;
  std::shared_ptr<State> last_closed_;
  void close(Record& record) {
    record.state->close();
    if (!record.registration) return;
    auto* cookie = record.registration; record.registration = nullptr;
    FlutterDesktopTextureRegistrarUnregisterExternalTexture(texture_registrar_, record.state->texture_id, [](void* value) {
      auto* cookie = static_cast<Registration*>(value);
      {
        std::lock_guard<std::mutex> lock(cookie->state->mutex);
        cookie->state->unregistered = true; cookie->state->registrar = nullptr;
      }
      delete cookie;  // Callback userdata lifetime only, never a frame/GPU credit.
    }, cookie);
  }
  void prune() {
    for (auto it = textures_.begin(); it != textures_.end();) {
      bool done;
      { std::lock_guard<std::mutex> lock(it->second.state->mutex); done = it->second.state->closed && it->second.state->unregistered && it->second.state->finished; }
      if (done) { last_closed_ = it->second.state; it = textures_.erase(it); } else ++it;
    }
  }
  Record* record(uint64_t id) {
    auto found = textures_.find(int64_t(id)); return found == textures_.end() ? nullptr : &found->second;
  }
  Value invoke(const std::string& method, const Map& args) {
    if (method == "create") {
      prune();
      if (textures_.size() >= 2) throw MethodError("backpressure", "two GPU textures are already active or closing");
      const auto* path = std::get_if<std::string>(&field(args, "libraryPath"));
      if (!path || path->empty() || path->size() > 32768 || !std::filesystem::u8path(*path).is_absolute())
        throw MethodError("invalidArgument", "libraryPath must be a bounded canonical absolute path");
      auto registration_credit = reserve_registration(); auto producer_credit = reserve_producer();
      if (!registration_credit || !producer_credit) throw MethodError("backpressure", "GPU producer/registration slots are full");
      ComPtr<IDXGIAdapter> adapter; ComPtr<IDXGIAdapter1> adapter1;
      if (!registrar_->GetGraphicsAdapter(&adapter) || !adapter || FAILED(adapter.As(&adapter1)))
        throw MethodError("notReady", "Flutter graphics adapter is unavailable; select CPU");
      const auto generation = next_generation.fetch_add(1);
      if (!generation) throw MethodError("backpressure", "device generation exhausted");
      auto state = std::make_shared<State>(generation, *path, adapter1, texture_registrar_, producer_credit);
      auto cookie = std::make_unique<Registration>(Registration{state, registration_credit});
      FlutterDesktopTextureInfo info{}; info.type = kFlutterDesktopGpuSurfaceTexture;
      info.gpu_surface_config.struct_size = sizeof(FlutterDesktopGpuSurfaceTextureConfig);
      info.gpu_surface_config.type = kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle;
      info.gpu_surface_config.callback = surface_callback; info.gpu_surface_config.user_data = cookie.get();
      state->texture_id = FlutterDesktopTextureRegistrarRegisterExternalTexture(texture_registrar_, &info);
      if (state->texture_id < 0) throw MethodError("notReady", "Flutter D3D11/DXGI texture registration failed");
      Record inserted{state, cookie.release()};
      try {
        textures_.emplace(state->texture_id, inserted);
        std::thread([state] { run_producer(state); }).detach();
      } catch (...) { close(inserted); textures_.erase(state->texture_id); throw; }
      return status(state);
    }
    const auto id = integer(args, "textureId");
    auto* item = record(id);
    if (!item) {
      if (method == "status" && last_closed_ && last_closed_->texture_id == int64_t(id)) return status(last_closed_);
      throw MethodError("invalidArgument", "unknown GPU texture ID");
    }
    if (method == "status") return status(item->state);
    if (method == "close") { close(*item); return status(item->state); }
    if (method == "submit") {
      std::shared_ptr<NativeApi> api;
      {
        std::lock_guard<std::mutex> lock(item->state->mutex);
        if (item->state->closed) throw MethodError("closing", "texture is closing");
        if (!item->state->ready) throw MethodError("notReady", "GPU producer is not ready");
        api = item->state->api;
      }
      auto job = parse_job(args);
      if (!api->current(job->check_json)) throw MethodError("staleFrame", "source/view/device stamp is stale");
      List ids; for (auto id : job->ids) ids.emplace_back(unsigned_value(id));
      Map receipt{{Value("accepted"), Value(true)}, {Value("acceptedSerial"), Value(int64_t(0))},
          {Value("acceptedStamp"), Value(stamp_value(job->stamp))}, {Value("acceptedSnapshots"), Value(ids)},
          {Value("limits"), limits()}};
      auto serial = receipt.find(Value("acceptedSerial"));
      if (!item->state->accept(job)) throw MethodError("staleFrame", "closed, superseded or invalid frame generation");
      // All response storage was allocated before ID ownership transferred.
      *std::get_if<int64_t>(&serial->second) = int64_t(job->serial);
      return Value(std::move(receipt));
    }
    throw MethodError("notImplemented", "unknown Windows GPU texture method");
  }
};
}  // namespace
}  // namespace rhwp::d3d

void RhwpD3dPluginRegisterWithRegistrar(FlutterDesktopPluginRegistrarRef registrar) {
  HMODULE module;
  if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
      reinterpret_cast<LPCWSTR>(&RhwpD3dPluginRegisterWithRegistrar), &module)) return;
  auto* wrapper = flutter::PluginRegistrarManager::GetInstance()->GetRegistrar<flutter::PluginRegistrarWindows>(registrar);
  wrapper->AddPlugin(std::make_unique<rhwp::d3d::RhwpD3dPlugin>(wrapper));
}
