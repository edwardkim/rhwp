#import "RhwpMetalPlugin.h"
#import <CoreGraphics/CoreGraphics.h>
#import <CoreVideo/CoreVideo.h>
#import <IOSurface/IOSurface.h>
#import <Metal/Metal.h>

#include "MetalFrameLifetime.hpp"
#include "TextureCloseRouting.hpp"
#include <atomic>
#include <condition_variable>
#include <cstdio>
#include <cstring>
#include <dlfcn.h>
#include <functional>
#include <exception>
#include <map>
#include <set>
#include <stdexcept>
#include <string>
#include <thread>

namespace {
using rhwp::metal::GenerationGate;
using rhwp::metal::ImmutableFrameCredits;
using rhwp::metal::Stamp;
constexpr size_t kMib = 1024 * 1024, kMaxFrameBytes = 16 * kMib;

// Process lifetime is deliberate: public FlutterTexture has no consumer GPU
// completion hook. Closing/recreating textures cannot replenish these credits.
ImmutableFrameCredits& Credits() {
  static auto* credits = new ImmutableFrameCredits(4, 64 * kMib, kMaxFrameBytes);
  return *credits;
}
std::atomic<int> activeProducers{0};
std::atomic<uint64_t> nextDeviceGeneration{1};
NSDictionary* StampValue(const Stamp& stamp);

struct NativeApi {
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

  NSDictionary* read(char* response) const {
    if (!response) return nil;
    const size_t size = strnlen(response, 64 * 1024 + 1);
    NSDictionary* result = nil;
    if (size <= 64 * 1024) {
      NSData* data = [NSData dataWithBytes:response length:size];
      id value = [NSJSONSerialization JSONObjectWithData:data options:0 error:nullptr];
      if ([value isKindOfClass:[NSDictionary class]]) result = value;
    }
    freeString(response);
    return result;
  }
};

template <class T> bool LoadSymbol(void* library, const char* name, T& output) {
  void* address = dlsym(library, name);
  static_assert(sizeof(address) == sizeof(output), "unsupported loader ABI");
  std::memcpy(&output, &address, sizeof(output));
  return output != nullptr;
}

std::shared_ptr<NativeApi> LoadApi(const std::string& path) {
  // Only producer threads load code. The same canonical dylib is used by the
  // legacy owner dispatcher; P89 snapshot IDs must not cross library copies.
  static std::mutex mutex;
  static std::string pinnedPath;
  static std::shared_ptr<NativeApi> pinned;
  std::lock_guard<std::mutex> lock(mutex);
  if (pinned) return pinnedPath == path ? pinned : nullptr;
  void* library = dlopen(path.c_str(), RTLD_NOW | RTLD_LOCAL);
  if (!library) return {};
  auto api = std::make_shared<NativeApi>();
  const bool loaded = LoadSymbol(library, "rhwp_metal_capabilities", api->capabilities) &&
      LoadSymbol(library, "rhwp_metal_presenter_create", api->create) &&
      LoadSymbol(library, "rhwp_metal_presenter_render", api->render) &&
      LoadSymbol(library, "rhwp_metal_presenter_request", api->request) &&
      LoadSymbol(library, "rhwp_metal_frame_check", api->checkFrame) &&
      LoadSymbol(library, "rhwp_presentation_request", api->presentation) &&
      LoadSymbol(library, "rhwp_string_free", api->freeString);
  if (!loaded) { dlclose(library); return {}; }
  pinnedPath = path;
  pinned = api;  // Deliberately never dlclose a live native/TLS module.
  return api;
}

struct SnapshotLease {
  uint64_t id;
  std::shared_ptr<NativeApi> api;
  bool transferred = false;
  ~SnapshotLease() {
    if (!transferred) return;
    char request[96];
    const int size = snprintf(request, sizeof(request), "{\"op\":\"snapshotRelease\",\"snapshot\":%llu}",
                              static_cast<unsigned long long>(id));
    char* response = api->presentation(reinterpret_cast<const uint8_t*>(request), size);
    if (response) api->freeString(response);
  }
};

struct PixelResources {
  CVPixelBufferRef buffer = nullptr;
  CVMetalTextureRef cvTexture = nullptr;
  id<MTLTexture> texture = nil;
  ~PixelResources() {
    if (cvTexture) CVBufferRelease(cvTexture);
    if (buffer) CVPixelBufferRelease(buffer);
  }
};
struct Job {
  uint64_t sequence = 0;
  Stamp stamp;
  int width = 0, height = 0;
  std::string request;
  std::string frameCheck;
  std::vector<uint64_t> ids;
  std::vector<std::shared_ptr<SnapshotLease>> snapshots;
  std::shared_ptr<ImmutableFrameCredits::Reservation> credit;
};
struct PublishedFrame {
  std::shared_ptr<Job> job;
  std::shared_ptr<PixelResources> pixels;
};

struct State {
  State(uint64_t generation, std::string path, std::shared_ptr<int> slot, bool closeAckRequired = false)
      : generation(generation), path(std::move(path)), slot(std::move(slot)), gate(generation),
        closeAckRequired(closeAckRequired) {}
  std::mutex mutex;
  std::condition_variable changed;
  const uint64_t generation;
  const std::string path;
  std::shared_ptr<int> slot;
  GenerationGate gate;
  const bool closeAckRequired;
  bool closed = false, ready = false, finished = false, inFlight = false, unregistered = false;
  uint64_t staleDrops = 0;
  std::string error;
  std::shared_ptr<NativeApi> api;
  std::shared_ptr<Job> pending;
  std::shared_ptr<PublishedFrame> current;
  std::map<uint64_t, std::weak_ptr<SnapshotLease>> leases;
  Stamp acceptedStamp;
  std::vector<uint64_t> acceptedSnapshots;
  std::function<void()> notify;

  bool enqueue(const std::shared_ptr<Job>& job) {
    std::shared_ptr<Job> oldPending;
    std::shared_ptr<PublishedFrame> oldFrame;
    {
      std::lock_guard<std::mutex> lock(mutex);
      if (closed || !ready) return false;
      for (auto it = leases.begin(); it != leases.end();) {
        if (it->second.expired()) it = leases.erase(it); else ++it;
      }
      job->snapshots.reserve(job->ids.size());
      for (const auto id : job->ids) {
        auto lease = leases[id].lock();
        if (!lease) {
          lease = std::make_shared<SnapshotLease>();
          lease->id = id;
          lease->api = api;
          leases[id] = lease;
        }
        job->snapshots.push_back(std::move(lease));
      }
      auto receipt = job->ids;
      job->sequence = gate.accept(job->stamp);
      if (!job->sequence) return false;
      // Receipt storage is allocated before ownership transfer as well.
      acceptedSnapshots = std::move(receipt);
      acceptedStamp = job->stamp;
      // Ownership changes only after every admission/allocation can fail.
      for (const auto& lease : job->snapshots) lease->transferred = true;
      oldPending = std::move(pending);
      oldFrame = std::move(current);
      pending = job;
    }
    changed.notify_one();
    return true;
  }

  void close() {
    std::shared_ptr<Job> oldPending;
    std::shared_ptr<PublishedFrame> oldFrame;
    {
      std::lock_guard<std::mutex> lock(mutex);
      closed = true;
      ready = false;
      gate.close();
      oldPending = std::move(pending);
      oldFrame = std::move(current);
    }
    changed.notify_one();
  }
  void fail(NSString* message) {
    std::lock_guard<std::mutex> lock(mutex);
    const char* text = message.UTF8String;
    error = text ? std::string(text).substr(0, 2048) : "Metal producer failed";
  }
};

std::shared_ptr<PixelResources> CreatePixels(id<MTLDevice> device, CVMetalTextureCacheRef cache,
                                          const Job& job) {
  auto pixels = std::make_shared<PixelResources>();
  NSDictionary* attributes = @{
    (__bridge NSString*)kCVPixelBufferMetalCompatibilityKey: @YES,
    (__bridge NSString*)kCVPixelBufferIOSurfacePropertiesKey: @{},
    (__bridge NSString*)kCVPixelBufferBytesPerRowAlignmentKey: @256,
  };
  if (CVPixelBufferCreate(kCFAllocatorDefault, job.width, job.height,
      kCVPixelFormatType_32BGRA, (__bridge CFDictionaryRef)attributes, &pixels->buffer) != kCVReturnSuccess ||
      !CVPixelBufferGetIOSurface(pixels->buffer)) return {};
  const size_t stride = CVPixelBufferGetBytesPerRow(pixels->buffer);
  const size_t allocated = CVPixelBufferGetDataSize(pixels->buffer);
  if (!stride || !allocated || stride > job.credit->bytes() / static_cast<size_t>(job.height) ||
      allocated > job.credit->bytes()) return {};
  CGColorSpaceRef color = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  if (!color) return {};
  CVBufferSetAttachment(pixels->buffer, kCVImageBufferCGColorSpaceKey, color, kCVAttachmentMode_ShouldPropagate);
  CGColorSpaceRelease(color);
  NSDictionary* textureAttributes = @{
    (__bridge NSString*)kCVMetalTextureUsage: @(MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead),
  };
  if (CVMetalTextureCacheCreateTextureFromImage(kCFAllocatorDefault, cache, pixels->buffer,
      (__bridge CFDictionaryRef)textureAttributes, MTLPixelFormatBGRA8Unorm,
      job.width, job.height, 0, &pixels->cvTexture) != kCVReturnSuccess) return {};
  pixels->texture = CVMetalTextureGetTexture(pixels->cvTexture);
  id<MTLTexture> texture = pixels->texture;
  if (!texture || texture.device != device || texture.width != static_cast<NSUInteger>(job.width) ||
      texture.height != static_cast<NSUInteger>(job.height) || texture.pixelFormat != MTLPixelFormatBGRA8Unorm ||
      texture.textureType != MTLTextureType2D || texture.sampleCount != 1 ||
      (texture.usage & (MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead)) !=
          (MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead)) return {};
  return pixels;
}

void RunProducer(const std::shared_ptr<State>& state) {
  @autoreleasepool {
    auto api = LoadApi(state->path);
    id<MTLDevice> device = MTLCreateSystemDefaultDevice();
    id<MTLCommandQueue> queue = [device newCommandQueue];
    CVMetalTextureCacheRef cache = nullptr;
    uint64_t presenter = 0;
    if (api && device && queue) {
      NSDictionary* capabilities = api->read(api->capabilities());
      if ([capabilities[@"ok"] boolValue] && [capabilities[@"available"] boolValue] &&
          [capabilities[@"abiVersion"] unsignedLongLongValue] == 1 &&
          CVMetalTextureCacheCreate(kCFAllocatorDefault, nullptr, device, nullptr, &cache) == kCVReturnSuccess) {
        NSDictionary* created = api->read(api->create((__bridge const void*)device,
            (__bridge const void*)queue, state->generation, nullptr, 0));
        if ([created[@"ok"] boolValue]) presenter = [created[@"presenter"] unsignedLongLongValue];
      }
    }
    if (!presenter) state->fail(@"Metal feature/device/presenter is unavailable; use the CPU reference path.");
    {
      std::lock_guard<std::mutex> lock(state->mutex);
      state->api = api;
      state->ready = presenter && !state->closed;
    }
    while (presenter) {
      std::shared_ptr<Job> job;
      {
        std::unique_lock<std::mutex> lock(state->mutex);
        state->changed.wait(lock, [&] {return state->closed || state->pending != nullptr;});
        if (state->closed) break;
        job = std::move(state->pending);
        state->inFlight = true;
      }
      @autoreleasepool {
        auto pixels = CreatePixels(device, cache, *job);
        if (!pixels) {
          state->fail(@"CVPixelBuffer/CVMetalTexture allocation or target contract failed.");
        } else {
          NSDictionary* result = api->read(api->render(presenter, (__bridge const void*)pixels->texture,
              reinterpret_cast<const uint8_t*>(job->request.data()), job->request.size()));
          NSDictionary* frameInfo = result[@"frame"];
          const bool frameMatches = [frameInfo isKindOfClass:[NSDictionary class]] &&
              [frameInfo[@"stamp"] isEqual:StampValue(job->stamp)] &&
              [frameInfo[@"target"][@"width"] longLongValue] == job->width &&
              [frameInfo[@"target"][@"height"] longLongValue] == job->height &&
              [frameInfo[@"target"][@"deviceGeneration"] unsignedLongLongValue] == state->generation &&
              [frameInfo[@"pixelFormat"] isEqual:@"bgra8UnormPremultiplied"] &&
              [frameInfo[@"colorSpace"] isEqual:@"srgb"] && [frameInfo[@"alpha"] isEqual:@"premultiplied"] &&
              [frameInfo[@"origin"] isEqual:@"topLeft"];
          const bool producerComplete = [result[@"ok"] boolValue] &&
              frameMatches && [frameInfo[@"producerGpuComplete"] boolValue];
          const bool stale = [result[@"code"] isEqual:@"staleFrame"] &&
              [result[@"unpublishedTargetSafeToDiscard"] boolValue];
          if (stale) {
            std::lock_guard<std::mutex> lock(state->mutex);
            if (state->staleDrops != UINT64_MAX) ++state->staleDrops;
          } else if (!producerComplete) {
            // A submit/device failure proves neither producer nor consumer
            // completion. Keep this allocation and its credit, fail closed.
            job->credit->retain_until_process_exit(pixels);
            state->fail(@"Metal replay failed; target quarantined without GPU-completion evidence.");
            state->close();
          } else {
            std::lock_guard<std::mutex> lock(state->mutex);
            if (state->gate.current(job->sequence, job->stamp)) {
              auto frame = std::make_shared<PublishedFrame>();
              frame->job = job;
              frame->pixels = pixels;
              job->credit->retain_until_process_exit(pixels);
              state->current = std::move(frame);
              state->error.clear();
            }
            // Stale, never-exposed successful targets can safely return credit.
          }
        }
      }
      // Evict unused cache bindings after local unpublished resources drop;
      // published/quarantined CVMetalTexture references remain strongly held.
      CVMetalTextureCacheFlush(cache, 0);
      {
        std::lock_guard<std::mutex> lock(state->mutex);
        state->inFlight = false;
      }
      if (state->notify) state->notify();
    }
    if (presenter) {
      static const uint8_t release[] = "{\"op\":\"release\"}";
      (void)api->read(api->request(presenter, release, sizeof(release)-1));
    }
    if (cache) {CVMetalTextureCacheFlush(cache, 0); CFRelease(cache);}
    {
      std::lock_guard<std::mutex> lock(state->mutex);
      state->ready = false;
      state->finished = true;
      state->slot.reset();
    }
    if (state->notify) state->notify();
  }
}

bool Unsigned(id value, uint64_t& output, uint64_t maximum = INT64_MAX) {
  if (![value isKindOfClass:[NSNumber class]] || CFGetTypeID((__bridge CFTypeRef)value) == CFBooleanGetTypeID()) return false;
  const char type = [value objCType][0];
  if (type == 'f' || type == 'd' || [value longLongValue] < 0 || [value unsignedLongLongValue] > maximum) return false;
  output = [value unsignedLongLongValue];
  return true;
}
bool Number(id value, double& output) {
  if (![value isKindOfClass:[NSNumber class]] || CFGetTypeID((__bridge CFTypeRef)value) == CFBooleanGetTypeID()) return false;
  output = [value doubleValue];
  return std::isfinite(output);
}
std::shared_ptr<Job> ParseJob(NSDictionary* arguments, uint64_t generation, bool reserveImmutable = true) {
  id stampValue = arguments[@"stamp"], placementsValue = arguments[@"placements"];
  if (![stampValue isKindOfClass:[NSDictionary class]] || ![placementsValue isKindOfClass:[NSArray class]]) return {};
  NSDictionary* stamp = stampValue;
  NSArray* placements = placementsValue;
  if (!placements.count || placements.count > 8) return {};
  auto job = std::make_shared<Job>();
  uint64_t width = 0, height = 0, clear = 0;
  if (!Unsigned(arguments[@"width"], width, 4096) || !Unsigned(arguments[@"height"], height, 4096) ||
      !width || !height || !Unsigned(arguments[@"clearColor"], clear, UINT32_MAX) ||
      !Unsigned(stamp[@"documentId"], job->stamp.document) || !Unsigned(stamp[@"revision"], job->stamp.revision) ||
      !Unsigned(stamp[@"fontGeneration"], job->stamp.font) || !Unsigned(stamp[@"viewGeneration"], job->stamp.view) ||
      !Unsigned(stamp[@"deviceGeneration"], job->stamp.device) || !Number(stamp[@"deviceScale"], job->stamp.device_scale) ||
      !job->stamp.valid() || job->stamp.device != generation) return {};
  job->width = static_cast<int>(width);
  job->height = static_cast<int>(height);
  const size_t reserved = ((width * 4 + 255) / 256 * 256) * height;
  if (reserved > kMaxFrameBytes) return {};
  NSMutableArray* normalized = [NSMutableArray arrayWithCapacity:placements.count];
  std::set<uint64_t> ids;
  for (id item in placements) {
    if (![item isKindOfClass:[NSDictionary class]]) return {};
    uint64_t snapshot = 0, background = 0;
    double scale = 0, x = 0, y = 0;
    if (!Unsigned(item[@"snapshot"], snapshot) || !snapshot || !Number(item[@"scale"], scale) ||
        scale <= 0 || scale > 16 || !Number(item[@"originX"], x) || !Number(item[@"originY"], y) ||
        std::abs(x) > 16777216 || std::abs(y) > 16777216) return {};
    id backgroundValue = item[@"backgroundColor"];
    if (backgroundValue && backgroundValue != [NSNull null] && !Unsigned(backgroundValue, background, UINT32_MAX)) return {};
    [normalized addObject:@{@"snapshot":@(snapshot),@"scale":@(scale),@"originX":@(x),@"originY":@(y),
        @"backgroundColor":(!backgroundValue || backgroundValue == [NSNull null]) ? [NSNull null] : @(background)}];
    ids.insert(snapshot);
  }
  NSDictionary* request = @{
    @"stamp":@{@"documentId":@(job->stamp.document),@"revision":@(job->stamp.revision),@"fontGeneration":@(job->stamp.font),
      @"viewGeneration":@(job->stamp.view),@"deviceGeneration":@(job->stamp.device),@"deviceScale":@(job->stamp.device_scale)},
    @"target":@{@"width":@(width),@"height":@(height),@"deviceGeneration":@(generation)},
    @"placements":normalized,@"clearColor":@(clear),
  };
  NSData* data = [NSJSONSerialization dataWithJSONObject:request options:0 error:nullptr];
  if (!data || data.length > 64 * 1024) return {};
  job->request.assign(static_cast<const char*>(data.bytes), data.length);
  job->ids.assign(ids.begin(), ids.end());
  NSMutableArray* snapshots = [NSMutableArray arrayWithCapacity:ids.size()];
  for (const auto snapshot : ids) [snapshots addObject:@(snapshot)];
  data = [NSJSONSerialization dataWithJSONObject:@{@"stamp":request[@"stamp"],@"snapshots":snapshots} options:0 error:nullptr];
  if (!data || data.length > 64 * 1024) return {};
  job->frameCheck.assign(static_cast<const char*>(data.bytes), data.length);
  if (reserveImmutable) job->credit = Credits().reserve(reserved);
  return job;
}
NSDictionary* StampValue(const Stamp& stamp) {
  if (!stamp.valid()) return @{};
  return @{@"documentId":@(stamp.document),@"revision":@(stamp.revision),@"fontGeneration":@(stamp.font),
    @"viewGeneration":@(stamp.view),@"deviceGeneration":@(stamp.device),@"deviceScale":@(stamp.device_scale)};
}
NSDictionary* CreditStatus() {
  const auto stats = Credits().stats();
  return @{@"outstandingFrames":@(stats.outstanding_frames),@"pendingRetireFrames":@(stats.retained_pending_retire),
    @"reservedTargetBytes":@(stats.reserved_bytes),@"maxFrames":@4,@"maxTargetBytes":@(64*kMib),
    @"consumerRetirement":@NO,@"continuousPresentationSupported":@NO,@"cpuFallbackRequiredWhenSaturated":@YES};
}
}  // namespace

#include "MetalNativeView.hpp"

namespace {
struct CVCloseState { uint64_t textureId; std::shared_ptr<State> state; };
struct CVCloseReceipt { uint64_t textureId, generation; __strong NSDictionary* status; };
NSDictionary* CVLimits(bool negotiated) {
  NSMutableDictionary* limits = [CreditStatus() mutableCopy];
  limits[@"closeAcknowledgementSupported"] = @YES;
  limits[@"closeAcknowledgementRequired"] = @(negotiated);
  limits[@"consumerGpuCompletionVerified"] = @NO;
  limits[@"closeAckRefundsFrameCredit"] = @NO;
  if (negotiated) {
    limits[@"maxRetainedCloseRecords"] = @2;
    limits[@"maxClosedReceipts"] = @2;
  }
  return limits;
}
NSDictionary* CVStatus(uint64_t textureId, const std::shared_ptr<State>& state, bool registered) {
  std::lock_guard<std::mutex> lock(state->mutex);
  NSString* error = [[NSString alloc] initWithBytes:state->error.data() length:state->error.size() encoding:NSUTF8StringEncoding];
  NSMutableArray* ids = [NSMutableArray arrayWithCapacity:state->acceptedSnapshots.size()];
  for (const auto snapshot : state->acceptedSnapshots) [ids addObject:@(snapshot)];
  return @{@"textureId":@(textureId),@"deviceGeneration":@(state->generation),
    @"closeAcknowledgementRequired":@(state->closeAckRequired),@"registered":@(registered),
    @"ready":@(state->ready),@"closed":@(state->closed),@"ownerFinished":@(state->finished),
    @"pending":@(state->pending!=nullptr),@"inFlight":@(state->inFlight),@"hasFrame":@(state->current!=nullptr),
    @"staleFramesDropped":@(state->staleDrops),@"textureUnregisteredNotification":@(state->unregistered),
    @"error":error?:@"",@"acceptedStamp":StampValue(state->acceptedStamp),@"acceptedSnapshots":ids,
    @"limits":CVLimits(state->closeAckRequired)};
}
NSDictionary* CVRetireAck(uint64_t textureId, uint64_t generation) {
  return @{@"textureId":@(textureId),@"deviceGeneration":@(generation),@"acknowledged":@YES,
    @"closeAcknowledgementRequired":@YES,@"closed":@YES,@"ownerFinished":@YES,@"registered":@NO};
}
NSDictionary* CVClosedStatus(const CVCloseReceipt& receipt) {
  NSMutableDictionary* value = [receipt.status mutableCopy];
  // Terminal owner facts are immutable; process-wide allocation counters are live.
  value[@"limits"] = CVLimits(true);
  return value;
}
}  // namespace

@interface RhwpMetalTexture : NSObject <FlutterTexture>
- (instancetype)initWithState:(std::shared_ptr<State>)state;
- (std::shared_ptr<State>)state;
@end
@implementation RhwpMetalTexture {
  std::shared_ptr<State> _state;
}
- (instancetype)initWithState:(std::shared_ptr<State>)state {
  if ((self = [super init])) _state = std::move(state);
  return self;
}
- (std::shared_ptr<State>)state { return _state; }
- (CVPixelBufferRef)copyPixelBuffer {
  std::shared_ptr<PublishedFrame> frame;
  std::shared_ptr<NativeApi> api;
  {
    std::lock_guard<std::mutex> lock(_state->mutex);
    frame = _state->current;
    if (!frame || !_state->gate.current(frame->job->sequence, frame->job->stamp)) return nullptr;
    api = _state->api;
  }
  @autoreleasepool {
    NSDictionary* result = api->read(api->checkFrame(
        reinterpret_cast<const uint8_t*>(frame->job->frameCheck.data()), frame->job->frameCheck.size()));
    if (![result[@"ok"] boolValue] || ![result[@"current"] boolValue]) return nullptr;
  }
  std::lock_guard<std::mutex> lock(_state->mutex);
  if (!_state->gate.current(frame->job->sequence, frame->job->stamp)) return nullptr;
  // Copy naming means +1 CF ownership. No pixel read/copy occurs here.
  return CVPixelBufferRetain(frame->pixels->buffer);
}
- (void)onTextureUnregistered:(NSObject<FlutterTexture>*)texture {
  (void)texture;
  std::lock_guard<std::mutex> lock(_state->mutex);
  _state->unregistered = true;  // Notification only, never a GPU fence/credit.
}
- (void)dealloc { if (_state) _state->close(); }
@end

@interface RhwpMetalPlugin ()
- (void)handleCall:(FlutterMethodCall*)call result:(FlutterResult)result;
@end

@implementation RhwpMetalPlugin {
  id<FlutterTextureRegistry> _registry;
  NSMutableDictionary<NSNumber*, RhwpMetalTexture*>* _textures;
  std::shared_ptr<State> _lastClosed;
  uint64_t _lastClosedId;
  rhwp::metal::RetirementRegistry<CVCloseState,CVCloseReceipt> _closeRecords;
}
+ (void)registerWithRegistrar:(id<FlutterPluginRegistrar>)registrar {
  [RhwpMetalViewPlugin registerWithRegistrar:registrar];
  RhwpMetalPlugin* plugin = [[RhwpMetalPlugin alloc] init];
  plugin->_registry = registrar.textures;
  plugin->_textures = [NSMutableDictionary dictionary];
  FlutterMethodChannel* channel = [FlutterMethodChannel methodChannelWithName:@"rhwp_native/texture_macos"
      binaryMessenger:registrar.messenger];
  [registrar addMethodCallDelegate:plugin channel:channel];
}
- (void)handleMethodCall:(FlutterMethodCall*)call result:(FlutterResult)result {
  try {
    [self handleCall:call result:result];
  } catch (const std::exception&) {
    result([FlutterError errorWithCode:@"nativeAllocation" message:@"The bounded native request could not be allocated." details:nil]);
  }
}
- (void)handleCall:(FlutterMethodCall*)call result:(FlutterResult)result {
  if (![NSThread isMainThread]) {
    result([FlutterError errorWithCode:@"platformThread" message:@"Texture registration requires the platform thread." details:nil]);
    return;
  }
  if ([call.method isEqualToString:@"capabilities"]) { result(CVLimits(false)); return; }
  if (![call.arguments isKindOfClass:[NSDictionary class]]) {
    result([FlutterError errorWithCode:@"arguments" message:@"Expected a bounded argument map." details:nil]); return;
  }
  NSDictionary* arguments = call.arguments;
  if ([call.method isEqualToString:@"create"]) {
    id requestedAck = arguments[@"closeAcknowledgementRequired"];
    if (requestedAck && (![requestedAck isKindOfClass:[NSNumber class]] ||
        CFGetTypeID((__bridge CFTypeRef)requestedAck) != CFBooleanGetTypeID())) {
      result([FlutterError errorWithCode:@"arguments" message:@"closeAcknowledgementRequired must be a boolean." details:nil]);return;
    }
    const bool negotiated = [requestedAck boolValue];
    size_t closing = 0;
    for (const auto& record : _closeRecords.records()) if (record.second.closing) ++closing;
    NSString* input = arguments[@"libraryPath"];
    if (![input isKindOfClass:[NSString class]] || !input.isAbsolutePath || input.length > 4096) {
      result([FlutterError errorWithCode:@"arguments" message:@"Absolute libraryPath and at most two texture registrations are required." details:nil]); return;
    }
    if (_textures.count + closing >= 2) {
      result([FlutterError errorWithCode:@"backpressure" message:@"Two active or unacknowledged texture close records are retained." details:nil]);return;
    }
    auto slot = std::shared_ptr<int>(new int(0), [](int* value) {if (*value) --activeProducers; delete value;});
    int active = activeProducers.load();
    while (active < 2 && !activeProducers.compare_exchange_weak(active, active+1)) {}
    if (active >= 2) {result([FlutterError errorWithCode:@"backpressure" message:@"Two producer owners are already active." details:nil]);return;}
    *slot = 1;
    auto generation = nextDeviceGeneration.load();
    while (generation <= INT64_MAX && !nextDeviceGeneration.compare_exchange_weak(generation, generation+1)) {}
    if (!generation || generation > INT64_MAX) {result([FlutterError errorWithCode:@"generation" message:@"Device generation exhausted." details:nil]);return;}
    NSString* path = input.stringByResolvingSymlinksInPath;
    if (!path.UTF8String) {result([FlutterError errorWithCode:@"arguments" message:@"Library path is not UTF-8." details:nil]);return;}
    auto state = std::make_shared<State>(generation, path.UTF8String, std::move(slot), negotiated);
    RhwpMetalTexture* texture = [[RhwpMetalTexture alloc] initWithState:state];
    const int64_t textureId = [_registry registerTexture:texture];
    if (!textureId) {state->close();result([FlutterError errorWithCode:@"registration" message:@"Flutter rejected the texture registration." details:nil]);return;}
    try {
      if (negotiated && !_closeRecords.insert(generation,
          std::make_shared<CVCloseState>(CVCloseState{static_cast<uint64_t>(textureId),state})))
        throw std::runtime_error("bounded texture close registry is full");
      _textures[@(textureId)] = texture;
      __weak id<FlutterTextureRegistry> weakRegistry = _registry;
      std::weak_ptr<State> weakState = state;
      state->notify = [weakRegistry, weakState, textureId] {
        dispatch_async(dispatch_get_main_queue(), ^{
          auto current = weakState.lock();
          if (!current) return;
          {std::lock_guard<std::mutex> lock(current->mutex);if (current->closed) return;}
          [weakRegistry textureFrameAvailable:textureId];
        });
      };
      std::thread([state] {
        try {RunProducer(state);} catch (const std::exception&) {
          state->fail(@"Native producer allocation failed; owner thread stopped.");
          state->close();
          std::lock_guard<std::mutex> lock(state->mutex);
          state->inFlight = false;
          state->finished = true;
          state->slot.reset();
        }
      }).detach();
    } catch (const std::exception&) {
      state->close();
      if (negotiated) _closeRecords.erase_unstarted(generation);
      [_registry unregisterTexture:textureId];
      [_textures removeObjectForKey:@(textureId)];
      throw;
    }
    result(@{@"textureId":@(textureId),@"deviceGeneration":@(generation),@"initializing":@YES,
      @"closeAcknowledgementRequired":@(negotiated),@"limits":CVLimits(negotiated)});
    return;
  }
  uint64_t textureId = 0;
  if (!Unsigned(arguments[@"textureId"], textureId) || !textureId) {
    result([FlutterError errorWithCode:@"arguments" message:@"Invalid textureId." details:nil]);return;
  }
  RhwpMetalTexture* texture = _textures[@(textureId)];
  auto active = texture ? [texture state] : nullptr;
  uint64_t requestedGeneration = 0;
  if (arguments[@"deviceGeneration"] &&
      (!Unsigned(arguments[@"deviceGeneration"], requestedGeneration) || !requestedGeneration)) {
    result([FlutterError errorWithCode:@"invalidArgument" message:@"deviceGeneration must be a positive integer." details:nil]);return;
  }
  auto retained = requestedGeneration ? _closeRecords.lookup(requestedGeneration) : nullptr;
  const auto* completed = requestedGeneration ? _closeRecords.receipt(requestedGeneration) : nullptr;
  bool knownRetainedHandle = false;
  for (const auto& record : _closeRecords.records())
    if (record.second.value->textureId == textureId) knownRetainedHandle = true;
  for (const auto& receipt : _closeRecords.receipts())
    if (receipt.second.textureId == textureId) knownRetainedHandle = true;
  using rhwp::metal::TextureCloseIdentity;
  using rhwp::metal::TextureCloseRoute;
  const auto route = rhwp::metal::ResolveTextureClose(textureId, requestedGeneration,
    active ? TextureCloseIdentity{textureId,active->generation,active->closeAckRequired} : TextureCloseIdentity{},
    retained ? TextureCloseIdentity{retained->textureId,retained->state->generation,true} : TextureCloseIdentity{},
    completed ? TextureCloseIdentity{completed->textureId,completed->generation,true} : TextureCloseIdentity{},
    _lastClosed ? TextureCloseIdentity{_lastClosedId,_lastClosed->generation,false} : TextureCloseIdentity{},
    knownRetainedHandle);
  if (route == TextureCloseRoute::requiresGeneration || route == TextureCloseRoute::stale) {
    NSString* code = route == TextureCloseRoute::requiresGeneration ? @"invalidArgument" : @"contextChanged";
    result([FlutterError errorWithCode:code message:@"Texture handle and deviceGeneration must match the negotiated owner." details:nil]);return;
  }
  if (route == TextureCloseRoute::completed) {
    if ([call.method isEqualToString:@"status"] || [call.method isEqualToString:@"close"]) {result(CVClosedStatus(*completed));return;}
    if ([call.method isEqualToString:@"retireAck"]) {result(CVRetireAck(textureId,requestedGeneration));return;}
    result([FlutterError errorWithCode:@"closed" message:@"Texture retirement was already acknowledged." details:nil]);return;
  }
  std::shared_ptr<State> state;
  if (route == TextureCloseRoute::negotiated) {
    state = retained->state;
    // A late close of generation A must never unregister a new B at the same address.
    if (!active || active->generation != state->generation) texture = nil;
  } else if (route == TextureCloseRoute::legacyActive) state = active;
  else if (route == TextureCloseRoute::legacyClosed) {state = _lastClosed;texture = nil;}
  if (!state) {result([FlutterError errorWithCode:@"closed" message:@"Unknown texture or expired acknowledged close receipt." details:nil]);return;}
  const bool registered = texture && (!state->closeAckRequired || _closeRecords.registered(state->generation));
  if ([call.method isEqualToString:@"retireAck"]) {
    bool complete;
    {std::lock_guard<std::mutex> lock(state->mutex);complete=state->closed&&state->finished&&!state->inFlight;}
    if (!state->closeAckRequired || registered || !complete) {
      result([FlutterError errorWithCode:@"retirePending" message:@"Negotiated producer close completion is required before acknowledgment." details:nil]);return;
    }
    NSDictionary* status = CVStatus(textureId,state,false);
    if (!_closeRecords.acknowledge(state->generation,complete,CVCloseReceipt{textureId,state->generation,status})) {
      result([FlutterError errorWithCode:@"retirePending" message:@"Texture close acknowledgment is not ready." details:nil]);return;
    }
    result(CVRetireAck(textureId,state->generation));return;
  }
  if ([call.method isEqualToString:@"submit"]) {
    if (!registered) {result(@{@"accepted":@NO,@"reason":@"closed",@"limits":CVLimits(state->closeAckRequired)});return;}
    auto job = ParseJob(arguments, state->generation);
    if (!job) {result([FlutterError errorWithCode:@"arguments" message:@"Invalid stamp, placements, or bounded target geometry." details:nil]);return;}
    if (!job->credit) {result(@{@"accepted":@NO,@"reason":@"backpressure",@"limits":CVLimits(state->closeAckRequired)});return;}
    const bool accepted = state->enqueue(job);
    if (!accepted) job.reset();
    result(@{@"accepted":@(accepted),@"acceptedStamp":accepted?StampValue(job->stamp):@{},
      @"reason":accepted?@"accepted":@"notReadyOrStale",@"limits":CVLimits(state->closeAckRequired)});
  } else if ([call.method isEqualToString:@"status"]) {
    result(CVStatus(textureId,state,registered));
  } else if ([call.method isEqualToString:@"close"]) {
    if (state->closeAckRequired) _closeRecords.close(state->generation);
    state->close();
    if (texture) {
      [_registry unregisterTexture:static_cast<int64_t>(textureId)];
      [_textures removeObjectForKey:@(textureId)];
      if (!state->closeAckRequired) {_lastClosed = state;_lastClosedId = textureId;}
    }
    bool finished;
    {std::lock_guard<std::mutex> lock(state->mutex);finished = state->finished;}
    result(@{@"textureId":@(textureId),@"deviceGeneration":@(state->generation),@"closed":@YES,
      @"registered":@NO,@"ownerFinished":@(finished),@"ownerCompletionPending":@(!finished),
      @"closeAcknowledgementRequired":@(state->closeAckRequired),@"limits":CVLimits(state->closeAckRequired)});
  } else {result(FlutterMethodNotImplemented);}
}
- (void)dealloc {
  NSDictionary<NSNumber*, RhwpMetalTexture*>* textures = [_textures copy];
  id<FlutterTextureRegistry> registry = _registry;
  for (RhwpMetalTexture* texture in textures.allValues) ([texture state])->close();
  dispatch_block_t unregister = ^{for (NSNumber* key in textures) [registry unregisterTexture:key.longLongValue];};
  if ([NSThread isMainThread]) unregister(); else dispatch_async(dispatch_get_main_queue(), unregister);
}
@end
