#include "rhwp_native_reference/rhwp_d3d_plugin.h"
#include "Producer.h"
#include "CloseRecordRegistry.h"
#include "../common/bounded_tile_group.h"
#include <optional>
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
PublicationMode publication_mode(const Map& args) {
  auto found = args.find(Value("publicationMode"));
  if (found == args.end()) return PublicationMode::LegacyImmutable;
  const auto* mode = std::get_if<std::string>(&found->second);
  if (mode && *mode == "legacyImmutable") return PublicationMode::LegacyImmutable;
  if (mode && *mode == "immutableOwnershipTransfer") return PublicationMode::ImmutableOwnershipTransfer;
  throw MethodError("invalidArgument", "publicationMode must be legacyImmutable or immutableOwnershipTransfer");
}
uint64_t integer(const Value& value) {
  int64_t result;
  if (auto* wide = std::get_if<int64_t>(&value)) result = *wide;
  else if (auto* narrow = std::get_if<int32_t>(&value)) result = *narrow;
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
  else if (const auto* wide = std::get_if<int64_t>(&value)) result = double(*wide);
  else if (const auto* narrow = std::get_if<int32_t>(&value)) result = double(*narrow);
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
Value limits(PublicationMode mode = PublicationMode::LegacyImmutable, bool close_ack = false) {
  const auto stats = frames().stats();
  const auto imports = SurfaceImports::stats();
  const bool transfer = mode == PublicationMode::ImmutableOwnershipTransfer;
  return Value(Map{{Value("publicationMode"), Value(publication_mode_name(mode))},
      {Value("closeAcknowledgementRequired"), Value(close_ack)},
      {Value("maxRetainedCloseRecords"), Value(2)}, {Value("maxClosedReceipts"), Value(2)},
      {Value("consumerRetirement"), Value(false)}, {Value("continuousPresentationSupported"), Value(transfer)},
      {Value("consumerGpuCompletionVerified"), Value(false)}, {Value("targetReuse"), Value(false)},
      {Value("backpressureRetryable"), Value(transfer && stats.permanent < 4)},
      {Value("frameAccountingScope"), Value("legacyRetainedAndPluginOwnedReferences")},
      {Value("consumerAndDriverMemoryIncluded"), Value(false)},
      {Value("cpuFallbackRequiredWhenSaturated"), Value(true)}, {Value("outstandingFrames"), unsigned_value(stats.outstanding)},
      {Value("immutableFrames"), unsigned_value(stats.immutable)}, {Value("quarantinedFrames"), unsigned_value(stats.quarantined)},
      {Value("pendingRetireFrames"), unsigned_value(transfer ? stats.outstanding : stats.immutable + stats.quarantined)},
      {Value("legacyRetainedFrames"), unsigned_value(stats.legacy_retained)},
      {Value("producerOwnedFrames"), unsigned_value(stats.producer_owned)},
      {Value("producerOwnedBytes"), unsigned_value(stats.producer_owned_bytes)},
      {Value("importPendingFrames"), unsigned_value(stats.import_pending)},
      {Value("importPendingBytes"), unsigned_value(stats.import_pending_bytes)},
      {Value("handedOffFrames"), unsigned_value(stats.handed_off)},
      {Value("discardedUnexposedFrames"), unsigned_value(stats.unexposed_discarded)},
      {Value("descriptorTickets"), unsigned_value(imports.tickets)},
      {Value("descriptorCpuPendingTickets"), unsigned_value(imports.cpu_pending)},
      {Value("descriptorAwaitingOpenedTickets"), unsigned_value(imports.awaiting_opened)},
      {Value("descriptorDuplicateOrLateNotifications"), unsigned_value(imports.ignored)},
      {Value("maxDescriptorTickets"), Value(int64_t(SurfaceImports::kMaxTickets))},
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
      {Value("publicationMode"), Value(publication_mode_name(state->publication_mode))},
      {Value("closeAcknowledgementRequired"), Value(state->close_ack_required)},
      {Value("ready"), Value(state->ready && !state->closed)}, {Value("closed"), Value(state->closed)},
      {Value("ownerFinished"), Value(state->finished)}, {Value("inFlight"), Value(state->in_flight)},
      {Value("unregistered"), Value(state->unregistered)}, {Value("registered"), Value(!state->unregistered)},
      {Value("error"), Value(state->error)},
      {Value("errorCode"), Value(state->error_code)}, {Value("deviceFailure"), Value(state->device_failure)},
      {Value("acceptedSerial"), unsigned_value(state->accepted_serial)}, {Value("acceptedStamp"), Value(stamp_value(state->accepted_stamp))},
      {Value("acceptedSnapshots"), Value(ids)}, {Value("staleDrops"), unsigned_value(state->stale_drops)},
      {Value("hasFrame"), Value(has_frame)},
      {Value("descriptorOpenedSerial"), unsigned_value(has_frame && state->current->pixels->opened_notifications.load() ?
          state->current->job->serial : 0)},
      {Value("renderedSerial"), unsigned_value(has_frame ? state->current->job->serial : 0)},
      {Value("limits"), limits(state->publication_mode, state->close_ack_required)}};
  value[Value("renderedStamp")] = has_frame ? Value(stamp_value(state->current->job->stamp)) : Value();
  return Value(std::move(value));
}
std::shared_ptr<Job> parse_job(const Map& args, PublicationMode mode) {
  auto job = std::make_shared<Job>();
  const auto& source = map(&field(args, "stamp"));
  job->stamp = {integer(source, "documentId"), integer(source, "revision"), integer(source, "fontGeneration"),
      integer(source, "viewGeneration"), integer(source, "deviceGeneration"), number(source, "deviceScale")};
  if (!job->stamp.valid()) throw MethodError("invalidArgument", "invalid source/view/device stamp");
  job->width = integer32(args, "width"); job->height = integer32(args, "height");
  const auto charge = frame_charge(job->width, job->height);
  if (!charge) throw MethodError(mode == PublicationMode::ImmutableOwnershipTransfer ? "targetTooLarge" : "backpressure",
      "two-surface viewport exceeds the immutable frame budget; use CPU");
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
  job->credit = frames().reserve(charge, mode);
  if (!job->credit) throw MethodError("backpressure", "immutable GPU frame credits are exhausted; use CPU");
  return job;
}

class RhwpD3dPlugin final : public flutter::Plugin {
 public:
  RhwpD3dPlugin(flutter::PluginRegistrarWindows* registrar,
      FlutterDesktopTextureRegistrarRef texture_registrar) : registrar_(registrar),
      texture_registrar_(texture_registrar) {
    channel_ = std::make_unique<flutter::MethodChannel<Value>>(registrar->messenger(), "rhwp_native/texture_windows",
        &flutter::StandardMethodCodec::GetInstance());
    channel_->SetMethodCallHandler([this](const auto& call, auto result) {
      try { result->Success(invoke(call.method_name(), map(call.arguments()))); }
      catch (const MethodError& error) { result->Error(error.code, error.what(), error_limits(call.arguments())); }
      catch (const std::invalid_argument& error) { result->Error("invalidArgument", error.what(), error_limits(call.arguments())); }
      catch (const std::exception& error) { result->Error("nativeError", error.what(), error_limits(call.arguments())); }
    });
  }
  ~RhwpD3dPlugin() override {
    channel_->SetMethodCallHandler(nullptr);
    for (auto& item : textures_.records()) close(item.second);
  }
 private:
  struct Record { std::shared_ptr<State> state; Registration* registration; };
  struct ClosedReceipt { Value value; PublicationMode mode; bool close_ack; uint64_t generation; };
  flutter::PluginRegistrarWindows* registrar_;
  FlutterDesktopTextureRegistrarRef texture_registrar_;
  std::unique_ptr<flutter::MethodChannel<Value>> channel_;
  CloseRecordRegistry<Record, ClosedReceipt> textures_;
  struct TileGroup { rhwp::tiles::Identity identity; Value terminal; };
  std::optional<TileGroup> group_;
  std::set<int64_t> unexposed_creations_;
  void close(Record& record) {
    record.state->close();
    if (!record.registration) return;
    auto* cookie = record.registration; record.registration = nullptr;
    FlutterDesktopTextureRegistrarUnregisterExternalTexture(texture_registrar_, record.state->texture_id, [](void* value) {
      auto* cookie = static_cast<Registration*>(value);
      // Finish our descriptor/creator bookkeeping before exposing retirement.
      // The engine has already removed this registration at this point.
      retire_surface_imports(cookie->state);
      cookie->credit.reset();
      {
        std::lock_guard<std::mutex> lock(cookie->state->mutex);
        cookie->state->unregistered = true; cookie->state->registrar = nullptr;
      }
      delete cookie;  // CPU registration/descriptor quiescence is not a GPU fence.
    }, cookie);
  }
  void prune() {
    // Old clients never negotiated retireAck. Preserve their existing automatic
    // create-time retirement; new clients retain closing records until ack.
    for (auto it = textures_.records().begin(); it != textures_.records().end();) {
      const auto id = (it++)->first;
      auto* item = record(uint64_t(id));
      if (item && (!item->state->close_ack_required || unexposed_creations_.count(id))) {
        if (retire(id, false)) unexposed_creations_.erase(id);
      }
    }
  }
  Record* record(uint64_t id) {
    return textures_.find(int64_t(id));
  }
  bool retire(int64_t id, bool acknowledged) {
    return textures_.retire(id, [](const Record& record) {
      std::lock_guard<std::mutex> lock(record.state->mutex);
      return CloseCompletion{record.state->closed, record.state->finished, record.state->unregistered, record.state->in_flight}.ready();
    }, [acknowledged](const Record& record) {
      auto value = status(record.state);
      std::get_if<Map>(&value)->emplace(Value("acknowledged"), Value(acknowledged));
      return ClosedReceipt{std::move(value), record.state->publication_mode, record.state->close_ack_required, record.state->generation};
    });
  }
  Value closed_status(const ClosedReceipt& receipt) {
    auto value = receipt.value;
    (*std::get_if<Map>(&value))[Value("limits")] = limits(receipt.mode, receipt.close_ack);
    return value;
  }
  void check_generation(const Map& args, uint64_t generation, bool required) {
    const auto found = args.find(Value("deviceGeneration"));
    const bool present = found != args.end();
    if (!close_generation_matches(generation, present ? integer(found->second) : 0, present, required)) {
      throw MethodError(present ? "contextChanged" : "invalidArgument", "texture calls require the correct deviceGeneration");
    }
  }
  Value error_limits(const Value* arguments) {
    PublicationMode mode = PublicationMode::LegacyImmutable;
    try {
      const auto& args = map(arguments);
      auto found = args.find(Value("textureId"));
      if (found != args.end()) {
        if (auto* item = record(integer(found->second))) return limits(item->state->publication_mode, item->state->close_ack_required);
        else if (const auto* receipt = textures_.completed(int64_t(integer(found->second)))) return limits(receipt->mode, receipt->close_ack);
      } else if (auto group = args.find(Value("groupId")); group != args.end()) {
        const auto generation = args.find(Value("deviceGeneration"));
        if (group_ && group_->identity.matches(integer(group->second)) &&
            generation != args.end() && integer(generation->second) == group_->identity.generation) {
          // Group calls carry no textureId/publicationMode. Derive retry policy
          // from this exact retained owner, never the caller's requested mode.
          const auto member = group_->identity.textures[0];
          if (auto* item = record(member); item && item->state->generation == group_->identity.generation)
            return limits(item->state->publication_mode, item->state->close_ack_required);
          if (const auto* receipt = textures_.completed(int64_t(member)); receipt && receipt->generation == group_->identity.generation)
            return limits(receipt->mode, receipt->close_ack);
        }
      } else mode = publication_mode(args);
    } catch (...) { /* Invalid request metadata cannot choose a transfer mode. */ }
    return limits(mode);
  }
  Value invoke(const std::string& method, const Map& args, uint64_t group_generation = 0) {
    if (method == "capabilities") return Value(Map{{Value("tileGroups"), Value(true)},
        {Value("maxTileGroupTiles"),Value(2)},{Value("maxTileGroupSnapshots"),Value(4)},
        {Value("sharedSnapshotLeases"),Value(true)},{Value("consumerGpuCompletionVerified"),Value(false)}});
    if (method == "createGroup") {
      prune();
      if (group_ && std::holds_alternative<std::monostate>(group_->terminal))
        throw MethodError("backpressure", "one two-tile group is already retained");
      const auto generation = next_generation.fetch_add(1);
      if (!generation) throw MethodError("backpressure", "tile group generation exhausted");
      rhwp::tiles::Identity identity; identity.generation = generation;
      List members; members.reserve(2);
      try {
        for (size_t index=0; index<2; ++index) {
          auto created = invoke("create",args,generation);
          identity.textures[index] = integer(std::get<Map>(created),"textureId");
          members.push_back(std::move(created));
        }
      } catch (...) {
        for (const auto id : identity.textures) if (id) {
          auto* item=record(id); if (item) {unexposed_creations_.insert(int64_t(id));close(*item);}
        }
        throw;
      }
      group_=TileGroup{identity,Value()};
      return Value(Map{{Value("groupId"),unsigned_value(identity.id())},{Value("deviceGeneration"),unsigned_value(generation)},
          {Value("members"),Value(std::move(members))}});
    }
    if (method == "submitGroup" || method == "statusGroup" || method == "closeGroup" || method == "retireGroupAck") {
      const auto id=integer(args,"groupId");
      if (!group_ || !group_->identity.matches(id) || integer(args,"deviceGeneration")!=group_->identity.generation)
        throw MethodError("contextChanged","unknown or stale tile group");
      const auto identity=group_->identity;
      if (!std::holds_alternative<std::monostate>(group_->terminal)) {
        if (method=="submitGroup") throw MethodError("closed","tile group is closed");
        return group_->terminal;
      }
      if (method=="submitGroup") {
        const auto* tiles=std::get_if<List>(&field(args,"tiles"));
        if (!tiles || tiles->size()!=2) throw MethodError("invalidArgument","two tile descriptions required");
        std::array<std::shared_ptr<Job>,2> jobs;
        std::array<std::shared_ptr<State>,2> states;
        for(size_t index=0;index<2;++index) {
          auto* item=record(identity.textures[index]);
          if(!item) throw MethodError("closed","tile member no longer exists");
          auto tile=map(&(*tiles)[index]);
          if(integer(tile,"textureId")!=identity.textures[index]) throw MethodError("invalidArgument","tile member order/identity differs");
          tile[Value("stamp")]=field(args,"stamp");tile[Value("clearColor")]=field(args,"clearColor");
          states[index]=item->state;jobs[index]=parse_job(tile,item->state->publication_mode);
          if(jobs[index]->stamp.device!=identity.generation) throw MethodError("contextChanged","tile stamp device differs");
          std::shared_ptr<NativeApi> api;
          {std::lock_guard<std::mutex> lock(item->state->mutex);api=item->state->api;}
          if(!api || !api->current(jobs[index]->check_json)) throw MethodError("staleFrame","tile full stamp is not current");
        }
        const auto ids=rhwp::tiles::unique_snapshots({jobs[0]->ids,jobs[1]->ids});
        List accepted;for(const auto snapshot:ids)accepted.emplace_back(unsigned_value(snapshot));
        Map reply{{Value("accepted"),Value(true)},{Value("acceptedStamp"),Value(stamp_value(jobs[0]->stamp))},
            {Value("acceptedSnapshots"),Value(std::move(accepted))},
            {Value("limits"),limits(states[0]->publication_mode,true)}};
        if(!State::accept_group(states[0],states[1],jobs[0],jobs[1]))
          throw MethodError("staleFrame","tile group did not accept either member");
        return Value(std::move(reply));
      }
      List members;members.reserve(2);
      for(const auto member:identity.textures) {
        Map child{{Value("textureId"),unsigned_value(member)},{Value("deviceGeneration"),unsigned_value(identity.generation)}};
        members.push_back(invoke(method=="statusGroup"?"status":method=="closeGroup"?"close":"retireAck",child));
      }
      auto reply=Value(Map{{Value("groupId"),unsigned_value(identity.id())},{Value("deviceGeneration"),unsigned_value(identity.generation)},
          {Value("members"),Value(std::move(members))}});
      if(method=="retireGroupAck")group_->terminal=reply;
      return reply;
    }
    if (method == "create") {
      prune();
      if (!textures_.can_create()) throw MethodError("backpressure", "two GPU textures are active or awaiting close acknowledgement");
      const auto mode = publication_mode(args);
      bool close_ack = false;
      const auto request_ack = args.find(Value("closeAcknowledgementRequired"));
      if (request_ack != args.end()) {
        const auto* requested = std::get_if<bool>(&request_ack->second);
        if (!requested) throw MethodError("invalidArgument", "closeAcknowledgementRequired must be a boolean");
        close_ack = *requested;
      }
      const auto* path = std::get_if<std::string>(&field(args, "libraryPath"));
      if (!path || path->empty() || path->size() > 32768 || !std::filesystem::u8path(*path).is_absolute())
        throw MethodError("invalidArgument", "libraryPath must be a bounded canonical absolute path");
      auto registration_credit = reserve_registration(); auto producer_credit = reserve_producer();
      if (!registration_credit || !producer_credit) throw MethodError("backpressure", "GPU producer/registration slots are full");
      ComPtr<IDXGIAdapter> adapter; ComPtr<IDXGIAdapter1> adapter1;
      if (!registrar_->GetGraphicsAdapter(&adapter) || !adapter || FAILED(adapter.As(&adapter1)))
        throw MethodError("notReady", "Flutter graphics adapter is unavailable; select CPU");
      const auto import_owner = next_generation.fetch_add(1);
      const auto generation = group_generation ? group_generation : import_owner;
      if (!generation || !import_owner) throw MethodError("backpressure", "device generation exhausted");
      auto state = std::make_shared<State>(generation, *path, adapter1, texture_registrar_, producer_credit, mode, close_ack, import_owner);
      auto cookie = std::make_unique<Registration>(Registration{state, registration_credit});
      FlutterDesktopTextureInfo info{}; info.type = kFlutterDesktopGpuSurfaceTexture;
      info.gpu_surface_config.struct_size = sizeof(FlutterDesktopGpuSurfaceTextureConfig);
      info.gpu_surface_config.type = kFlutterDesktopGpuSurfaceTypeDxgiSharedHandle;
      info.gpu_surface_config.callback = surface_callback; info.gpu_surface_config.user_data = cookie.get();
      register_surface_imports(state);
      state->texture_id = FlutterDesktopTextureRegistrarRegisterExternalTexture(texture_registrar_, &info);
      if (state->texture_id < 0) {
        retire_surface_imports(state);
        throw MethodError("notReady", "Flutter D3D11/DXGI texture registration failed");
      }
      Record inserted{state, cookie.release()};
      try {
        if (!textures_.insert(state->texture_id, inserted))
          throw MethodError("backpressure", "texture ID is retained by an unacknowledged owner");
        std::thread([state] { run_producer(state); }).detach();
      } catch (...) {
        close(inserted);
        // A rejected reused ID still belongs to its previous unacknowledged
        // record. Erase only a record that actually owns this new generation.
        auto* rollback = record(uint64_t(state->texture_id));
        if (rollback && rollback->state == state) textures_.erase_failed_creation(state->texture_id);
        throw;
      }
      return status(state);
    }
    const auto id = integer(args, "textureId");
    auto* item = record(id);
    if (!item) {
      if (const auto* receipt = textures_.completed(int64_t(id))) {
        check_generation(args, receipt->generation, receipt->close_ack);
        if (method == "status" || method == "close" || (method == "retireAck" && receipt->close_ack)) return closed_status(*receipt);
      }
      throw MethodError("invalidArgument", "unknown GPU texture ID");
    }
    check_generation(args, item->state->generation, item->state->close_ack_required);
    if (method == "status") return status(item->state);
    if (method == "close") { close(*item); return status(item->state); }
    if (method == "retireAck") {
      if (!item->state->close_ack_required) throw MethodError("unsupported", "close acknowledgement was not negotiated");
      if (!retire(int64_t(id), true)) throw MethodError("retirePending", "producer and unregister completion are still pending");
      return closed_status(*textures_.completed(int64_t(id)));
    }
    if (method == "submit") {
      std::shared_ptr<NativeApi> api;
      {
        std::lock_guard<std::mutex> lock(item->state->mutex);
        if (item->state->closed) throw MethodError("closing", "texture is closing");
        if (!item->state->ready) throw MethodError("notReady", "GPU producer is not ready");
        api = item->state->api;
      }
      auto job = parse_job(args, item->state->publication_mode);
      if (!api->current(job->check_json)) throw MethodError("staleFrame", "source/view/device stamp is stale");
      List ids; for (auto snapshot : job->ids) ids.emplace_back(unsigned_value(snapshot));
      Map receipt{{Value("accepted"), Value(true)}, {Value("acceptedSerial"), Value(int64_t(0))},
          {Value("acceptedStamp"), Value(stamp_value(job->stamp))}, {Value("acceptedSnapshots"), Value(ids)},
          {Value("limits"), limits(item->state->publication_mode, item->state->close_ack_required)}};
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
  // The C registration entry owns the public raw registrar. The C++ wrapper's
  // registrar() accessor is protected and is not a plugin-facing API.
  auto* texture_registrar = FlutterDesktopRegistrarGetTextureRegistrar(registrar);
  wrapper->AddPlugin(std::make_unique<rhwp::d3d::RhwpD3dPlugin>(wrapper, texture_registrar));
}
