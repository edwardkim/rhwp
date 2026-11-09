// Included once by RhwpMetalPlugin.mm so both backends use the same Native
// loader, bounded request parser, snapshot leases and full-stamp generation gate.
#import <QuartzCore/CAMetalLayer.h>
#import <QuartzCore/CATransaction.h>
#import <AppKit/NSAnimationContext.h>
#include "MetalViewLifecycle.hpp"
#include <chrono>

namespace {
using rhwp::metal::DrawableLifecycle;
// Platform-channel visibility changes do not run inside Flutter's compositor
// transaction. Keep this owned view's AppKit and Core Animation mutations
// immediate even when an ambient caller enables implicit animation.
class ScopedMetalViewAnimations {
 public:
  ScopedMetalViewAnimations() {
    [NSAnimationContext beginGrouping];
    [NSAnimationContext currentContext].duration = 0;
    [NSAnimationContext currentContext].allowsImplicitAnimation = NO;
    [CATransaction begin];
    [CATransaction setDisableActions:YES];
  }
  ~ScopedMetalViewAnimations() {
    [CATransaction commit];
    [NSAnimationContext endGrouping];
  }
  ScopedMetalViewAnimations(const ScopedMetalViewAnimations&) = delete;
  ScopedMetalViewAnimations& operator=(const ScopedMetalViewAnimations&) = delete;
};
struct MetalViewState {
  explicit MetalViewState(std::shared_ptr<State> owner) : owner(std::move(owner)) {}
  std::shared_ptr<State> owner;
  // All fields below are serialized by owner->mutex. AppKit mutations stay on main.
  __strong CAMetalLayer* layer = nil;
  __weak NSView* view = nil;
  __strong id<MTLDevice> device = nil;
  DrawableLifecycle flow;
  rhwp::metal::VisibilityGate visibility;
  std::shared_ptr<Job> displayed;
  bool bound = false, layerReady = false, layerDetached = false;
  uint64_t viewIdentifier = 0, drawableTimeouts = 0, sizeMismatches = 0;
  uint64_t renderedSequence = 0;
  Stamp renderedStamp;
  uint32_t drawableWidth = 0, drawableHeight = 0;
};
struct MetalViewReceipt { __strong NSDictionary* status = nil; };
std::atomic<uint64_t> nextMetalViewHandle{1};
NSDictionary* MetalViewLimits() {
  return @{@"backend":@"appkitMetalLayer",@"presentationKind":@"nativeView",@"maxViews":@2,
    @"maximumDrawableCount":@3,@"maxTargetBytes":@(kMaxFrameBytes),
    @"tileGroups":@YES,@"maxTileGroupTiles":@2,@"maxTileGroupSnapshots":@4,@"sharedSnapshotLeases":@YES,
    @"continuousPresentationSupported":@YES,@"drawableReuseManagedByCoreAnimation":@YES,
    @"consumerGpuCompletionVerified":@NO,@"wholeFrameReadbacks":@0,
    @"nativeCoverSupported":@YES,@"nativeCoverRequired":@YES,
    @"closeAcknowledgementRequired":@YES,@"maxRetainedCloseRecords":@2,@"maxClosedReceipts":@2,
    @"experimental":@YES,@"requiresTargetQualification":@YES};
}
bool MetalJobCurrent(const std::shared_ptr<NativeApi>& api, const std::shared_ptr<Job>& job) {
  NSDictionary* checked = api->read(api->checkFrame(
      reinterpret_cast<const uint8_t*>(job->frameCheck.data()), job->frameCheck.size()));
  return [checked[@"ok"] boolValue] && [checked[@"current"] boolValue];
}
bool MetalProducerComplete(NSDictionary* result, const Job& job, uint64_t generation) {
  NSDictionary* frame = result[@"frame"];
  return [result[@"ok"] boolValue] && [frame isKindOfClass:[NSDictionary class]] &&
    [frame[@"stamp"] isEqual:StampValue(job.stamp)] &&
    [frame[@"target"][@"width"] longLongValue] == job.width &&
    [frame[@"target"][@"height"] longLongValue] == job.height &&
    [frame[@"target"][@"deviceGeneration"] unsignedLongLongValue] == generation &&
    [frame[@"pixelFormat"] isEqual:@"bgra8UnormPremultiplied"] &&
    [frame[@"colorSpace"] isEqual:@"srgb"] && [frame[@"alpha"] isEqual:@"premultiplied"] &&
    [frame[@"origin"] isEqual:@"topLeft"] && [frame[@"backend"] isEqual:@"metalGanesh"] &&
    [frame[@"wholeFrameReadbacks"] unsignedLongLongValue] == 0 && [frame[@"producerGpuComplete"] boolValue];
}
void RetryMetalViewJob(const std::shared_ptr<MetalViewState>& view, const std::shared_ptr<Job>& job) {
  auto state = view->owner;
  std::unique_lock<std::mutex> lock(state->mutex);
  state->inFlight = false;
  if (!state->closed && !state->pending && state->gate.current(job->sequence, job->stamp)) state->pending = job;
  state->changed.wait_for(lock, std::chrono::milliseconds(100), [&] {return state->closed || state->pending != job;});
}
// An unreadable Native completion must not destroy its presenter/TLS owner or
// drawable. This sleeping owner retains at most one target and one of the two
// process-wide producer slots until process exit, rather than guessing a fence.
[[noreturn]] void QuarantineMetalView(const std::shared_ptr<MetalViewState>& view,
                                    const std::shared_ptr<Job>& job,
                                    id<CAMetalDrawable> drawable, id<MTLCommandQueue> queue, CAMetalLayer* layer, NSString* error) {
  auto state = view->owner;
  __attribute__((objc_precise_lifetime)) NSArray* heldResources = @[drawable, queue, queue.device, layer];
  auto heldJob = job;
  state->fail(error);
  state->close();
  std::unique_lock<std::mutex> lock(state->mutex);
  view->flow.unknown_failure(job->sequence);
  // Keep these strong references on the live producer stack. No callback or
  // close message can acknowledge uncertain GPU work on this branch.
  (void)heldResources;
  (void)heldJob;
  state->changed.wait(lock, [] {return false;});
  std::terminate();
}
void StopMetalView(const std::shared_ptr<MetalViewState>& view);
void UpdateMetalViewGeometry(const std::shared_ptr<MetalViewState>& view);
void RunMetalView(const std::shared_ptr<MetalViewState>& view) {
  @autoreleasepool {
    auto state = view->owner;
    auto api = LoadApi(state->path);
    id<MTLDevice> device = MTLCreateSystemDefaultDevice();
    id<MTLCommandQueue> queue = [device newCommandQueue];
    uint64_t presenter = 0;
    if (api && device && queue) {
      NSDictionary* caps = api->read(api->capabilities());
      if ([caps[@"ok"] boolValue] && [caps[@"available"] boolValue] &&
          [caps[@"abiVersion"] unsignedLongLongValue] == 1) {
        NSDictionary* created = api->read(api->create((__bridge const void*)device,
            (__bridge const void*)queue, state->generation, nullptr, 0));
        if ([created[@"ok"] boolValue]) presenter = [created[@"presenter"] unsignedLongLongValue];
      }
    }
    if (!presenter) state->fail(@"Native Metal feature/device/queue is unavailable; use the CPU reference path.");
    {
      std::lock_guard<std::mutex> lock(state->mutex);
      state->api = api;
      state->ready = presenter && !state->closed;
      view->device = device;
    }
    dispatch_async(dispatch_get_main_queue(), ^{UpdateMetalViewGeometry(view);});
    while (presenter) {
      std::shared_ptr<Job> job;
      CAMetalLayer* layer = nil;
      {
        std::unique_lock<std::mutex> lock(state->mutex);
        state->changed.wait(lock, [&] {return state->closed || (state->pending && view->layerReady);});
        if (state->closed) break;
        job = std::move(state->pending);
        layer = view->layer;
        state->inFlight = true;
      }
      @autoreleasepool {
        id<CAMetalDrawable> drawable = nil;
        bool nativeSubmitted = false;
        try {
        drawable = [layer nextDrawable];
        if (!drawable) {
          {std::lock_guard<std::mutex> lock(state->mutex); if (view->drawableTimeouts != UINT64_MAX) ++view->drawableTimeouts;}
          RetryMetalViewJob(view, job);
          continue;
        }
        id<MTLTexture> target = drawable.texture;
        if (!rhwp::metal::DrawableSize{static_cast<uint32_t>(job->width), static_cast<uint32_t>(job->height)}.matches(target.width, target.height)) {
          {std::lock_guard<std::mutex> lock(state->mutex); if (view->sizeMismatches != UINT64_MAX) ++view->sizeMismatches;}
          RetryMetalViewJob(view, job);
          continue;
        }
        const bool validTarget = target && target.device == device && !target.framebufferOnly &&
          target.textureType == MTLTextureType2D && target.pixelFormat == MTLPixelFormatBGRA8Unorm &&
          target.sampleCount == 1 && (target.usage & (MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead)) ==
              (MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead);
        if (!validTarget) {
          state->fail(@"CAMetalLayer drawable does not match the typed Native Metal target contract.");
          state->close();
          {std::lock_guard<std::mutex> lock(state->mutex);state->inFlight = false;}
          break;
        }
        {
          std::lock_guard<std::mutex> lock(state->mutex);
          if (state->closed || !state->gate.current(job->sequence, job->stamp) || !view->flow.acquire(job->sequence)) {
            state->inFlight = false;
            continue;
          }
        }
        if (!MetalJobCurrent(api, job)) {
          std::lock_guard<std::mutex> lock(state->mutex);
          view->flow.discard_unsubmitted(job->sequence);
          state->inFlight = false;
          if (state->staleDrops != UINT64_MAX) ++state->staleDrops;
          continue;
        }
        nativeSubmitted = true;
        NSDictionary* rendered = api->read(api->render(presenter, (__bridge const void*)target,
            reinterpret_cast<const uint8_t*>(job->request.data()), job->request.size()));
        const bool stale = [rendered[@"code"] isEqual:@"staleFrame"] &&
          [rendered[@"unpublishedTargetSafeToDiscard"] boolValue];
        if (stale) {
          std::lock_guard<std::mutex> lock(state->mutex);
          view->flow.discard_unsubmitted(job->sequence);
          state->inFlight = false;
          if (state->staleDrops != UINT64_MAX) ++state->staleDrops;
          continue;
        }
        if (!MetalProducerComplete(rendered, *job, state->generation)) {
          QuarantineMetalView(view, job, drawable, queue, layer, @"Native Metal completion was not proved; drawable/producer quarantined.");
        }
        {
          std::lock_guard<std::mutex> lock(state->mutex);
          view->flow.producer_complete(job->sequence);
        }
        const bool sourceCurrent = MetalJobCurrent(api, job);
        id<MTLCommandBuffer> presentation = [queue commandBuffer];
        if (!presentation) {
          QuarantineMetalView(view, job, drawable, queue, layer, @"Could not create the same-queue Core Animation presentation command.");
        }
        {
          std::lock_guard<std::mutex> lock(state->mutex);
          if (state->closed || !sourceCurrent || !state->gate.current(job->sequence, job->stamp)) {
            view->flow.discard_completed(job->sequence);
            state->inFlight = false;
            if (!sourceCurrent && state->staleDrops != UINT64_MAX) ++state->staleDrops;
            continue;
          }
          // Native's producer completion preceded this same queue. Core Animation
          // owns display/reuse; no FlutterTexture consumer-retire inference exists.
          [presentation presentDrawable:drawable];
          [presentation commit];
        }
        [presentation waitUntilCompleted];
        if (presentation.status != MTLCommandBufferStatusCompleted) {
          QuarantineMetalView(view, job, drawable, queue, layer, @"Core Animation presentation command did not complete; owner quarantined.");
        }
        std::shared_ptr<Job> oldDisplayed;
        {
          std::lock_guard<std::mutex> lock(state->mutex);
          if (!state->closed && state->gate.current(job->sequence, job->stamp)) {
            view->flow.handoff(job->sequence);
            oldDisplayed = std::move(view->displayed);
            view->displayed = job;
            view->renderedSequence = job->sequence;
            view->renderedStamp = job->stamp;
            state->error.clear();
          } else {
            view->flow.discard_completed(job->sequence);
          }
          state->inFlight = false;
        }
        } catch (const std::exception&) {
          if (nativeSubmitted) QuarantineMetalView(view, job, drawable, queue, layer,
              @"Native Metal view allocation/decoding failed after submission; owner quarantined.");
          state->fail(@"Native Metal view allocation failed before submission.");
          state->close();
          {std::lock_guard<std::mutex> lock(state->mutex);
            view->flow.discard_unsubmitted(job->sequence);state->inFlight=false;}
          break;
        }
        // The autorelease pool releases our drawable reference promptly. Only
        // CAMetalLayer may supply a drawable again after display ownership ends.
      }
    }
    if (presenter) {
      static const uint8_t release[] = "{\"op\":\"release\"}";
      NSDictionary* released = api->read(api->request(presenter, release, sizeof(release)-1));
      if (![released[@"ok"] boolValue]) {
        state->fail(@"Native Metal presenter release was not confirmed.");
        state->close();
        std::unique_lock<std::mutex> lock(state->mutex);
        state->changed.wait(lock, [] {return false;});
      }
    }
    {
      std::lock_guard<std::mutex> lock(state->mutex);
      state->ready = false;
      state->inFlight = false;
      view->flow.close();
      state->finished = view->flow.finish_owner();
      // A bound CAMetalLayer also retains this slot until its own lifetime ends.
      state->slot.reset();
      view->device = nil;
    }
  }
}
}  // namespace

@interface RhwpOwnedMetalLayer : CAMetalLayer
- (instancetype)initWithOwnerSlot:(std::shared_ptr<int>)slot;
@end
@implementation RhwpOwnedMetalLayer {
  std::shared_ptr<int> _ownerSlot;
}
- (id<CAAction>)actionForKey:(NSString*)event {
  (void)event;
  // Override the complete lookup, rather than a delegate fallback. Returning
  // nil here is the final no-action result; NSNull is only a lookup sentinel
  // for delegates/dictionaries and is not itself a CAAction implementation.
  return nil;
}
- (instancetype)initWithOwnerSlot:(std::shared_ptr<int>)slot {
  if ((self = [super init])) _ownerSlot = std::move(slot);
  return self;
}
- (instancetype)initWithLayer:(id)layer {
  if ((self = [super initWithLayer:layer])) {
    if ([layer isKindOfClass:[RhwpOwnedMetalLayer class]])
      _ownerSlot = ((RhwpOwnedMetalLayer*)layer)->_ownerSlot;
  }
  return self;
}
@end

@interface RhwpNativeMetalView : NSView
- (instancetype)initWithState:(std::shared_ptr<MetalViewState>)state;
- (void)updateMetalGeometry;
- (void)detachMetalLayer;
@end
@implementation RhwpNativeMetalView {
  std::shared_ptr<MetalViewState> _metalState;
}
- (instancetype)initWithState:(std::shared_ptr<MetalViewState>)state {
  if ((self = [super initWithFrame:NSZeroRect])) {
    _metalState = std::move(state);
    ScopedMetalViewAnimations immediate;
    self.wantsLayer = YES;
    self.hidden = YES;
    auto owner = _metalState->owner;
    std::shared_ptr<int> slot;
    {std::lock_guard<std::mutex> lock(owner->mutex);slot=owner->slot;}
    RhwpOwnedMetalLayer* layer = [[RhwpOwnedMetalLayer alloc] initWithOwnerSlot:std::move(slot)];
    layer.pixelFormat = MTLPixelFormatBGRA8Unorm;
    layer.framebufferOnly = NO;
    layer.maximumDrawableCount = 3;
    layer.allowsNextDrawableTimeout = YES;
    layer.presentsWithTransaction = NO;
    layer.opacity = 1;
    layer.opaque = YES;
    layer.wantsExtendedDynamicRangeContent = NO;
    CGColorSpaceRef color = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    layer.colorspace = color;
    if (color) CGColorSpaceRelease(color);
    self.layer = layer;
    {std::lock_guard<std::mutex> lock(owner->mutex);_metalState->layer = layer;_metalState->view = self;}
  }
  return self;
}
- (BOOL)isFlipped { return YES; }
- (NSView*)hitTest:(NSPoint)point { (void)point; return nil; }
- (BOOL)acceptsFirstResponder { return NO; }
- (void)setHidden:(BOOL)hidden { ScopedMetalViewAnimations immediate; [super setHidden:hidden]; }
- (void)setFrameSize:(NSSize)size {
  ScopedMetalViewAnimations immediate;
  [super setFrameSize:size];
  [self updateMetalGeometry];
}
- (void)viewDidMoveToWindow { [super viewDidMoveToWindow]; [self updateMetalGeometry]; }
- (void)viewDidChangeBackingProperties { [super viewDidChangeBackingProperties]; [self updateMetalGeometry]; }
- (void)updateMetalGeometry {
  if (!_metalState) return;
  ScopedMetalViewAnimations immediate;
  auto owner = _metalState->owner;
  CAMetalLayer* layer = nil;
  id<MTLDevice> device = nil;
  {std::lock_guard<std::mutex> lock(owner->mutex);if(owner->closed)return;layer=_metalState->layer;device=_metalState->device;}
  const NSRect backing = [self convertRectToBacking:self.bounds];
  const auto checkedSize = rhwp::metal::DrawableSize::FromBacking(backing.size.width, backing.size.height);
  const bool geometry = self.window && checkedSize.has_value();
  const auto width = checkedSize ? checkedSize->width : 0;
  const auto height = checkedSize ? checkedSize->height : 0;
  if (layer && device && geometry) {
    layer.device = device;
    layer.contentsScale = self.window.backingScaleFactor;
    layer.drawableSize = CGSizeMake(width, height);
  }
  {std::lock_guard<std::mutex> lock(owner->mutex);
    _metalState->layerReady = !owner->closed && layer && device && geometry;
    _metalState->drawableWidth = geometry ? static_cast<uint32_t>(width) : 0;
    _metalState->drawableHeight = geometry ? static_cast<uint32_t>(height) : 0;}
  owner->changed.notify_one();
}
- (void)detachMetalLayer {
  ScopedMetalViewAnimations immediate;
  self.hidden = YES;
  self.layer = nil;
  self.wantsLayer = NO;
}
- (void)dealloc { if (_metalState) StopMetalView(_metalState); }
@end

namespace {
void UpdateMetalViewGeometry(const std::shared_ptr<MetalViewState>& view) {
  NSView* nativeView = nil;
  {std::lock_guard<std::mutex> lock(view->owner->mutex);nativeView=view->view;}
  if ([nativeView isKindOfClass:[RhwpNativeMetalView class]]) [(RhwpNativeMetalView*)nativeView updateMetalGeometry];
}
NSDictionary* MetalViewStatus(const std::shared_ptr<MetalViewState>& view, bool registered) {
  auto owner=view->owner;
  std::shared_ptr<Job> displayed;std::shared_ptr<NativeApi> api;
  {std::lock_guard<std::mutex> lock(owner->mutex);displayed=view->displayed;api=owner->api;}
  const bool current=displayed&&api&&MetalJobCurrent(api,displayed);
  std::unique_lock<std::mutex> lock(owner->mutex);
  const bool hasFrame=current&&!owner->closed&&owner->gate.current(displayed->sequence,displayed->stamp)&&view->displayed==displayed;
  if(!hasFrame)view->visibility.invalidate();
  NSMutableArray* ids=[NSMutableArray arrayWithCapacity:owner->acceptedSnapshots.size()];
  for(const auto id:owner->acceptedSnapshots)[ids addObject:@(id)];
  NSString* error=[[NSString alloc] initWithBytes:owner->error.data() length:owner->error.size() encoding:NSUTF8StringEncoding];
  NSDictionary* status=@{@"ready":@(owner->ready&&view->layerReady),@"apiReady":@(owner->ready),@"closed":@(owner->closed),@"ownerFinished":@(owner->finished),
    @"registered":@(registered),@"pending":@(owner->pending!=nullptr),@"inFlight":@(owner->inFlight),
    @"hasFrame":@(hasFrame),@"renderedStamp":hasFrame?StampValue(view->renderedStamp):@{},
    @"acceptedStamp":StampValue(owner->acceptedStamp),@"acceptedSnapshots":ids,@"deviceGeneration":@(owner->generation),
    @"presentationKind":@"nativeView",@"viewBound":@(view->bound),@"layerReady":@(view->layerReady),
    @"nativeViewLayerDetached":@(view->layerDetached),@"covered":@(view->visibility.covered()),
    @"visibilitySerial":@(view->visibility.serial()),
    @"drawableWidth":@(view->drawableWidth),@"drawableHeight":@(view->drawableHeight),
    @"frames":@(view->flow.scheduled()),@"discardedFrames":@(view->flow.discarded()),
    @"quarantined":@(view->flow.quarantined()),@"drawableTimeouts":@(view->drawableTimeouts),
    @"sizeMismatches":@(view->sizeMismatches),@"staleFramesDropped":@(owner->staleDrops),
    @"wholeFrameReadbacks":@0,@"error":error?:@"",@"limits":MetalViewLimits()};
  NSView* staleView=hasFrame?nil:view->view;
  lock.unlock();
  staleView.hidden=YES;
  return status;
}
NSDictionary* MetalRetireAck(uint64_t handle) {
  return @{@"textureId":@(handle),@"acknowledged":@YES,@"closed":@YES,@"ownerFinished":@YES,
    @"registered":@NO,@"nativeViewLayerDetached":@YES};
}
void StopMetalView(const std::shared_ptr<MetalViewState>& view) {
  auto owner = view->owner;
  owner->close();
  auto detach = ^{
    std::shared_ptr<Job> oldDisplayed;
    NSView* nativeView = nil;
    {std::lock_guard<std::mutex> lock(owner->mutex);
      view->flow.close();view->visibility.close();view->layerReady=false;view->layerDetached=true;
      nativeView=view->view;view->view=nil;view->layer=nil;oldDisplayed=std::move(view->displayed);}
    if ([nativeView isKindOfClass:[RhwpNativeMetalView class]]) [(RhwpNativeMetalView*)nativeView detachMetalLayer];
  };
  if ([NSThread isMainThread]) detach(); else dispatch_async(dispatch_get_main_queue(), detach);
  owner->changed.notify_one();
}
}  // namespace

@interface RhwpMetalViewPlugin : NSObject <FlutterPlugin, FlutterPlatformViewFactory>
- (void)handleCall:(FlutterMethodCall*)call result:(FlutterResult)result;
- (void)handleTileGroupCall:(FlutterMethodCall*)call result:(FlutterResult)result;
@end
@implementation RhwpMetalViewPlugin {
  rhwp::metal::RetirementRegistry<MetalViewState,MetalViewReceipt> _views;
  rhwp::tiles::Identity _tileGroup;
  uint64_t _creatingGroupGeneration;
  __strong NSDictionary* _tileTerminal;
  std::vector<uint64_t> _unexposedCreations;
}
+ (void)registerWithRegistrar:(id<FlutterPluginRegistrar>)registrar {
  RhwpMetalViewPlugin* plugin = [[RhwpMetalViewPlugin alloc] init];
  FlutterMethodChannel* channel = [FlutterMethodChannel methodChannelWithName:@"rhwp_native/metal_view_macos"
      binaryMessenger:registrar.messenger];
  [registrar addMethodCallDelegate:plugin channel:channel];
  [registrar registerViewFactory:plugin withId:@"rhwp_native/metal_view"];
}
- (NSObject<FlutterMessageCodec>*)createArgsCodec { return [FlutterStandardMessageCodec sharedInstance]; }
- (NSView*)createWithViewIdentifier:(int64_t)viewId arguments:(id)args {
  uint64_t handle = 0;
  if (![NSThread isMainThread] || viewId < 0 || ![args isKindOfClass:[NSDictionary class]] ||
      !Unsigned(args[@"handle"], handle) || !handle) return [[NSView alloc] initWithFrame:NSZeroRect];
  if (!_views.registered(handle)) return [[NSView alloc] initWithFrame:NSZeroRect];
  auto state = _views.lookup(handle);
  {std::lock_guard<std::mutex> lock(state->owner->mutex);
    if (state->bound || state->owner->closed) return [[NSView alloc] initWithFrame:NSZeroRect];
    state->bound=true;state->viewIdentifier=static_cast<uint64_t>(viewId);}
  return [[RhwpNativeMetalView alloc] initWithState:state];
}
- (void)handleMethodCall:(FlutterMethodCall*)call result:(FlutterResult)result {
  try { [self handleCall:call result:result]; }
  catch (const std::invalid_argument& error) {result([FlutterError errorWithCode:@"arguments" message:[NSString stringWithUTF8String:error.what()] details:nil]);}
  catch (const std::exception&) {result([FlutterError errorWithCode:@"nativeAllocation" message:@"Native Metal view allocation failed." details:nil]);}
}
- (void)handleCall:(FlutterMethodCall*)call result:(FlutterResult)result {
  if (![NSThread isMainThread]) {result([FlutterError errorWithCode:@"platformThread" message:@"AppKit view calls require the platform thread." details:nil]);return;}
  if ([call.method isEqualToString:@"capabilities"]) {result(MetalViewLimits());return;}
  if (![call.arguments isKindOfClass:[NSDictionary class]]) {result([FlutterError errorWithCode:@"arguments" message:@"Expected an argument map." details:nil]);return;}
  NSDictionary* arguments = call.arguments;
  if ([call.method isEqualToString:@"createGroup"] || [call.method isEqualToString:@"submitGroup"] ||
      [call.method isEqualToString:@"statusGroup"] || [call.method isEqualToString:@"closeGroup"] ||
      [call.method isEqualToString:@"retireGroupAck"] || [call.method isEqualToString:@"setCoveredGroup"]) {
    [self handleTileGroupCall:call result:result];return;
  }
  if ([call.method isEqualToString:@"create"]) {
    NSString* input=arguments[@"libraryPath"];
    if (![input isKindOfClass:[NSString class]] || !input.isAbsolutePath || input.length>4096) {
      result([FlutterError errorWithCode:@"arguments" message:@"Absolute libraryPath and at most two native views are required." details:nil]);return;}
    if(_views.size()>=2){result([FlutterError errorWithCode:@"backpressure" message:@"Two native views or unacknowledged close records are retained." details:nil]);return;}
    auto slot=std::shared_ptr<int>(new int(0),[](int* value){if(*value)--activeProducers;delete value;});
    int active=activeProducers.load();
    while(active<2&&!activeProducers.compare_exchange_weak(active,active+1)){}
    if(active>=2){result([FlutterError errorWithCode:@"backpressure" message:@"Two native producers/layers are retained." details:nil]);return;}
    *slot=1;
    uint64_t generation=_creatingGroupGeneration;
    if(!generation) {
      generation=nextDeviceGeneration.load();
      while(generation<=INT64_MAX&&!nextDeviceGeneration.compare_exchange_weak(generation,generation+1)){}
    }
    uint64_t handle=nextMetalViewHandle.load();
    while(handle<=INT64_MAX&&!nextMetalViewHandle.compare_exchange_weak(handle,handle+1)){}
    NSString* path=input.stringByResolvingSymlinksInPath;
    if(!generation||generation>INT64_MAX||!handle||handle>INT64_MAX||!path.UTF8String){result([FlutterError errorWithCode:@"generation" message:@"Native view identity is exhausted or path invalid." details:nil]);return;}
    auto owner=std::make_shared<State>(generation,path.UTF8String,std::move(slot));
    auto view=std::make_shared<MetalViewState>(owner);
    if(!_views.insert(handle,view)){result([FlutterError errorWithCode:@"backpressure" message:@"Native view registry capacity is exhausted." details:nil]);return;}
    try {std::thread([view] {
      try {RunMetalView(view);}
      catch (const std::exception&) {view->owner->fail(@"Native Metal view owner stopped unexpectedly; completion remains unconfirmed.");view->owner->close();}
    }).detach();}
    catch(const std::exception&){_views.erase_unstarted(handle);owner->close();throw;}
    result(@{@"textureId":@(handle),@"deviceGeneration":@(generation),@"initializing":@YES,@"ready":@NO,
      @"presentationKind":@"nativeView",@"viewType":@"rhwp_native/metal_view",@"limits":MetalViewLimits()});return;
  }
  uint64_t handle = 0;
  if(!Unsigned(arguments[@"textureId"],handle)||!handle){result([FlutterError errorWithCode:@"arguments" message:@"Invalid native view handle." details:nil]);return;}
  auto view=_views.lookup(handle);
  if(!view){
    const auto* receipt=_views.receipt(handle);
    if(receipt&&([call.method isEqualToString:@"status"]||[call.method isEqualToString:@"close"])){
      result(receipt->status);return;
    }
    if(receipt&&[call.method isEqualToString:@"retireAck"]){result(MetalRetireAck(handle));return;}
    result([FlutterError errorWithCode:@"closed" message:@"Unknown native view or expired acknowledged close receipt." details:nil]);return;
  }
  auto owner=view->owner;
  if([call.method isEqualToString:@"status"]){result(MetalViewStatus(view,_views.registered(handle)));return;}
  if([call.method isEqualToString:@"retireAck"]){
    bool complete;
    {std::lock_guard<std::mutex> lock(owner->mutex);
      complete=owner->closed&&owner->finished&&!owner->inFlight&&view->layerDetached&&!view->flow.quarantined();}
    if(_views.registered(handle)||!complete){
      result([FlutterError errorWithCode:@"retirePending" message:@"Producer completion and layer detach must be observed before close acknowledgment." details:nil]);return;
    }
    NSDictionary* status=MetalViewStatus(view,false);
    if(!_views.acknowledge(handle,complete,MetalViewReceipt{status})){
      result([FlutterError errorWithCode:@"retirePending" message:@"Native view close acknowledgment is not ready." details:nil]);return;
    }
    result(MetalRetireAck(handle));return;
  }
  if([call.method isEqualToString:@"submit"]){
    uint64_t clear = 0;
    if(!Unsigned(arguments[@"clearColor"],clear,UINT32_MAX)||(clear>>24)!=0xff){
      result([FlutterError errorWithCode:@"arguments" message:@"AppKit Metal presentation requires an opaque ARGB clear color." details:nil]);return;
    }
    auto job=ParseJob(arguments,owner->generation,false);
    if(!job){result([FlutterError errorWithCode:@"arguments" message:@"Invalid full stamp, placements, or bounded geometry." details:nil]);return;}
    const bool accepted=_views.registered(handle)&&owner->enqueue(job);
    if(accepted){
      NSView* nativeView=nil;
      {std::lock_guard<std::mutex> lock(owner->mutex);view->visibility.invalidate();nativeView=view->view;}
      nativeView.hidden=YES;
    }
    result(@{@"accepted":@(accepted),@"acceptedStamp":accepted?StampValue(job->stamp):@{},
      @"reason":accepted?@"accepted":@"notReadyOrStale",@"limits":MetalViewLimits()});return;
  }
  if([call.method isEqualToString:@"close"]){
    _views.close(handle);
    StopMetalView(view);
    bool finished;{std::lock_guard<std::mutex> lock(owner->mutex);finished=owner->finished;}
    result(@{@"closed":@YES,@"ownerCompletionPending":@(!finished),@"limits":MetalViewLimits()});return;
  }
  if([call.method isEqualToString:@"setCovered"]){
    std::shared_ptr<Job> displayed;std::shared_ptr<NativeApi> api;
    {std::lock_guard<std::mutex> lock(owner->mutex);displayed=view->displayed;api=owner->api;}
    const bool current=displayed&&api&&MetalJobCurrent(api,displayed);
    std::unique_lock<std::mutex> lock(owner->mutex);
    const bool hasFrame=current&&!owner->closed&&owner->gate.current(displayed->sequence,displayed->stamp)&&view->displayed==displayed;
    if(!hasFrame)view->visibility.invalidate();
    id covered=arguments[@"covered"];
    if(![covered isKindOfClass:[NSNumber class]]||CFGetTypeID((__bridge CFTypeRef)covered)!=CFBooleanGetTypeID()){
      result([FlutterError errorWithCode:@"arguments" message:@"covered must be a boolean." details:nil]);return;}
    uint64_t serial = 0;
    if(!Unsigned(arguments[@"visibilitySerial"],serial)||!serial){
      result([FlutterError errorWithCode:@"arguments" message:@"visibilitySerial must be a positive integer." details:nil]);return;}
    const bool wantCover=[covered boolValue];
    const bool stampMatches=hasFrame&&[arguments[@"stamp"] isKindOfClass:[NSDictionary class]]&&
      [arguments[@"stamp"] isEqual:StampValue(displayed->stamp)];
    const bool accepted=view->visibility.apply(serial,wantCover,stampMatches);
    NSView* nativeView=view->view;
    const bool hide=view->visibility.covered();
    const auto appliedSerial=view->visibility.serial();
    NSDictionary* renderedStamp=hasFrame?StampValue(view->renderedStamp):@{};
    lock.unlock();
    if(accepted||appliedSerial==serial)nativeView.hidden=hide;
    result(@{@"textureId":@(handle),@"covered":@(hide),@"visibilitySerial":@(appliedSerial),
      @"accepted":@(accepted),@"renderedStamp":renderedStamp});return;
  }
  result(FlutterMethodNotImplemented);
}
- (void)handleTileGroupCall:(FlutterMethodCall*)call result:(FlutterResult)result {
  NSDictionary* args=call.arguments;
  if([call.method isEqualToString:@"createGroup"]) {
    // Unexposed partial creation has no Dart owner which could acknowledge it.
    // Its normal producer/layer completion must still be proven before pruning.
    for(auto it=_unexposedCreations.begin();it!=_unexposedCreations.end();) {
      __block id ack=nil;
      [self handleCall:[FlutterMethodCall methodCallWithMethodName:@"retireAck" arguments:@{@"textureId":@(*it)}]
          result:^(id value){ack=value;}];
      if([ack isKindOfClass:[NSDictionary class]]&&[ack[@"acknowledged"] boolValue])it=_unexposedCreations.erase(it);else ++it;
    }
    if(_tileGroup.valid()&&!_tileTerminal){result([FlutterError errorWithCode:@"backpressure" message:@"One tile group is still retained." details:nil]);return;}
    uint64_t generation=nextDeviceGeneration.load();
    while(generation<=INT64_MAX&&!nextDeviceGeneration.compare_exchange_weak(generation,generation+1)){}
    if(!generation||generation>INT64_MAX){result([FlutterError errorWithCode:@"generation" message:@"Tile group generation exhausted." details:nil]);return;}
    rhwp::tiles::Identity identity;identity.generation=generation;
    NSMutableArray* members=[NSMutableArray arrayWithCapacity:2];
    id creationError=nil;
    _unexposedCreations.reserve(_unexposedCreations.size()+2);
    struct ResetGeneration {uint64_t& value;~ResetGeneration(){value=0;}} reset{_creatingGroupGeneration};
    _creatingGroupGeneration=generation;
    auto rollback=[&] {
      for(const auto handle:identity.textures)if(handle){auto view=_views.lookup(handle);if(view){_views.close(handle);StopMetalView(view);_unexposedCreations.push_back(handle);}}
    };
    try {
      for(size_t index=0;index<2;++index) {
        __block id created=nil;
        [self handleCall:[FlutterMethodCall methodCallWithMethodName:@"create" arguments:args] result:^(id value){created=value;}];
        uint64_t handle=0;
        if(![created isKindOfClass:[NSDictionary class]]||!Unsigned(created[@"textureId"],handle)||!handle){creationError=created;break;}
        identity.textures[index]=handle;[members addObject:created];
      }
    } catch (...) {rollback();throw;}
    if(!identity.valid()) {
      rollback();
      result(creationError?:[FlutterError errorWithCode:@"nativeAllocation" message:@"Tile group creation failed." details:nil]);return;
    }
    _tileGroup=identity;_tileTerminal=nil;
    result(@{@"groupId":@(identity.id()),@"deviceGeneration":@(generation),@"members":members});return;
  }
  uint64_t group=0,generation=0;
  if(!Unsigned(args[@"groupId"],group)||!_tileGroup.matches(group)||!Unsigned(args[@"deviceGeneration"],generation)||generation!=_tileGroup.generation) {
    result([FlutterError errorWithCode:@"contextChanged" message:@"Unknown or stale tile group." details:nil]);return;
  }
  if(_tileTerminal) {
    if([call.method isEqualToString:@"submitGroup"]){result([FlutterError errorWithCode:@"closed" message:@"Tile group is closed." details:nil]);return;}
    result(_tileTerminal);return;
  }
  std::array<std::shared_ptr<MetalViewState>,2> views;
  for(size_t index=0;index<2;++index) {
    views[index]=_views.lookup(_tileGroup.textures[index]);
    if(!views[index]&&![call.method isEqualToString:@"retireGroupAck"]&&![call.method isEqualToString:@"statusGroup"]){
      result([FlutterError errorWithCode:@"closed" message:@"Tile member is not active." details:nil]);return;
    }
  }
  if([call.method isEqualToString:@"submitGroup"]) {
    id tilesValue=args[@"tiles"];uint64_t clear=0;
    if(![tilesValue isKindOfClass:[NSArray class]]||[tilesValue count]!=2||!Unsigned(args[@"clearColor"],clear,UINT32_MAX)||(clear>>24)!=0xff) {
      result([FlutterError errorWithCode:@"arguments" message:@"Two tiles and opaque clear color are required." details:nil]);return;
    }
    NSArray* tiles=tilesValue;std::array<std::shared_ptr<Job>,2> jobs;
    for(size_t index=0;index<2;++index) {
      uint64_t handle=0;
      if(![tiles[index] isKindOfClass:[NSDictionary class]]||!Unsigned(tiles[index][@"textureId"],handle)||handle!=_tileGroup.textures[index]) {
        result([FlutterError errorWithCode:@"arguments" message:@"Tile order/identity differs." details:nil]);return;
      }
      NSMutableDictionary* tile=[tiles[index] mutableCopy];tile[@"stamp"]=args[@"stamp"];tile[@"clearColor"]=args[@"clearColor"];
      jobs[index]=ParseJob(tile,generation,false);
      if(!jobs[index]){result([FlutterError errorWithCode:@"arguments" message:@"Invalid tile target/stamp/placements." details:nil]);return;}
      std::shared_ptr<NativeApi> api;
      {std::lock_guard<std::mutex> lock(views[index]->owner->mutex);api=views[index]->owner->api;}
      if(!api||!MetalJobCurrent(api,jobs[index])){result([FlutterError errorWithCode:@"staleFrame" message:@"Tile source/view/device stamp is stale." details:nil]);return;}
    }
    const auto ids=rhwp::tiles::unique_snapshots({jobs[0]->ids,jobs[1]->ids});
    NSMutableArray* accepted=[NSMutableArray arrayWithCapacity:ids.size()];for(const auto snapshot:ids)[accepted addObject:@(snapshot)];
    NSDictionary* reply=@{@"accepted":@YES,@"acceptedStamp":StampValue(jobs[0]->stamp),@"acceptedSnapshots":accepted,@"limits":MetalViewLimits()};
    if(!EnqueueTileGroup(views[0]->owner,views[1]->owner,jobs[0],jobs[1])) {
      result([FlutterError errorWithCode:@"staleFrame" message:@"Neither tile was accepted." details:nil]);return;
    }
    ScopedMetalViewAnimations animations;
    for(const auto& view:views){NSView* native=nil;{std::lock_guard<std::mutex> lock(view->owner->mutex);view->visibility.invalidate();native=view->view;}native.hidden=YES;}
    result(reply);return;
  }
  if([call.method isEqualToString:@"setCoveredGroup"]) {
    id covered=args[@"covered"];uint64_t serial=0;
    if(![covered isKindOfClass:[NSNumber class]]||CFGetTypeID((__bridge CFTypeRef)covered)!=CFBooleanGetTypeID()||!Unsigned(args[@"visibilitySerial"],serial)||!serial) {
      result([FlutterError errorWithCode:@"arguments" message:@"Boolean cover and positive visibility serial required." details:nil]);return;
    }
    const bool hide=[covered boolValue];std::array<std::shared_ptr<Job>,2> displayed;
    std::array<bool,2> current{};
    for(size_t index=0;index<2;++index) {
      auto owner=views[index]->owner;std::shared_ptr<NativeApi> api;
      {std::lock_guard<std::mutex> lock(owner->mutex);displayed[index]=views[index]->displayed;api=owner->api;}
      current[index]=displayed[index]&&api&&MetalJobCurrent(api,displayed[index]);
    }
    std::array<NSView*,2> native{};bool accepted=true;NSMutableArray* members=[NSMutableArray arrayWithCapacity:2];
    {
      std::scoped_lock lock(views[0]->owner->mutex,views[1]->owner->mutex);
      for(size_t index=0;index<2;++index) {
        auto view=views[index];auto owner=view->owner;
        current[index]=current[index]&&!owner->closed&&view->displayed==displayed[index]&&owner->gate.current(displayed[index]->sequence,displayed[index]->stamp)&&
            [args[@"stamp"] isKindOfClass:[NSDictionary class]]&&[args[@"stamp"] isEqual:StampValue(displayed[index]->stamp)];
        accepted=accepted&&serial>view->visibility.serial()&&(!owner->closed)&&(hide||current[index]);
      }
      for(size_t index=0;index<2;++index) {
        auto view=views[index];native[index]=view->view;
        if(accepted)view->visibility.apply(serial,hide,current[index]);else view->visibility.invalidate();
        [members addObject:@{@"textureId":@(_tileGroup.textures[index]),@"covered":@(view->visibility.covered()),
            @"visibilitySerial":@(view->visibility.serial()),@"accepted":@(accepted),
            @"renderedStamp":current[index]?StampValue(view->renderedStamp):@{}}];
      }
    }
    ScopedMetalViewAnimations animations;for(const auto view:native)view.hidden=!accepted||hide;
    result(@{@"groupId":@(group),@"deviceGeneration":@(generation),@"members":members});return;
  }
  NSString* memberMethod=[call.method isEqualToString:@"statusGroup"]?@"status":[call.method isEqualToString:@"closeGroup"]?@"close":@"retireAck";
  NSMutableArray* members=[NSMutableArray arrayWithCapacity:2];
  for(const auto handle:_tileGroup.textures) {
    __block id reply=nil;
    [self handleCall:[FlutterMethodCall methodCallWithMethodName:memberMethod arguments:@{@"textureId":@(handle)}] result:^(id value){reply=value;}];
    if(![reply isKindOfClass:[NSDictionary class]]){result(reply);return;}
    NSMutableDictionary* identified=[reply mutableCopy];identified[@"textureId"]=@(handle);[members addObject:identified];
  }
  NSDictionary* reply=@{@"groupId":@(group),@"deviceGeneration":@(generation),@"members":members};
  if([call.method isEqualToString:@"retireGroupAck"])_tileTerminal=reply;
  result(reply);
}
- (void)dealloc {for(const auto& item:_views.records())StopMetalView(item.second.value);}
@end
