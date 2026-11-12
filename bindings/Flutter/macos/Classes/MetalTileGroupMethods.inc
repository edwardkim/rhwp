// Included inside RhwpMetalViewPlugin's implementation. All calls run on the
// platform thread; producer-visible state is serialized by both owner locks.
- (void)handleTileGroupCall:(FlutterMethodCall*)call result:(FlutterResult)result {
  NSDictionary* args = call.arguments;
  if ([call.method isEqualToString:@"createGroup"]) {
    // An unexposed partial start has no Dart owner. Reap only after actual
    // producer completion and layer detach, never merely after cancellation.
    for (auto& handle : _unexposedCreations) {
      if (!handle) continue;
      auto view = _views.lookup(handle);
      bool complete = false;
      if (view) {
        std::lock_guard<std::mutex> lock(view->owner->mutex);
        complete = view->owner->closed && view->owner->finished && !view->owner->inFlight &&
            view->layerDetached && !view->flow.quarantined();
      }
      if (complete && _views.acknowledge(handle, true, MetalViewReceipt{MetalViewStatus(view, false)})) handle = 0;
    }
    if ((_tileGroup.valid() && !_tileTerminal) || _views.size()) {
      result([FlutterError errorWithCode:@"backpressure" message:@"A tile pair needs both existing view/producer slots." details:nil]);return;
    }
    NSString* input = args[@"libraryPath"];
    if (![input isKindOfClass:[NSString class]] || !input.isAbsolutePath || input.length > 4096) {
      result([FlutterError errorWithCode:@"arguments" message:@"An absolute Native library path is required." details:nil]);return;
    }
    NSString* path = input.stringByResolvingSymlinksInPath;
    if (!path.UTF8String) {
      result([FlutterError errorWithCode:@"arguments" message:@"The Native library path must be UTF-8." details:nil]);return;
    }
    std::array<std::shared_ptr<int>, 2> slots;
    for (auto& slot : slots) slot = std::shared_ptr<int>(new int(0), [](int* value) {
      if (*value) --activeProducers;
      delete value;
    });
    int active = 0;
    if (!activeProducers.compare_exchange_strong(active, 2)) {
      result([FlutterError errorWithCode:@"backpressure" message:@"Two process-wide producer/layer slots are required." details:nil]);return;
    }
    for (const auto& slot : slots) *slot = 1;
    uint64_t generation = nextDeviceGeneration.load();
    while (generation <= INT64_MAX && !nextDeviceGeneration.compare_exchange_weak(generation, generation + 1)) {}
    uint64_t handle = nextMetalViewHandle.load();
    while (handle < INT64_MAX && !nextMetalViewHandle.compare_exchange_weak(handle, handle + 2)) {}
    if (!generation || generation > INT64_MAX || !handle || handle >= INT64_MAX) {
      result([FlutterError errorWithCode:@"generation" message:@"The tile group identity space is exhausted." details:nil]);return;
    }
    const rhwp::tiles::Identity identity{generation, {handle, handle + 1}};
    std::array<std::shared_ptr<MetalViewState>, 2> views;
    auto records = _views;
    NSMutableArray* members = [NSMutableArray arrayWithCapacity:2];
    for (size_t i = 0; i < 2; ++i) {
      auto owner = std::make_shared<State>(generation, path.UTF8String, std::move(slots[i]));
      views[i] = std::make_shared<MetalViewState>(owner);
      if (!records.insert(identity.textures[i], views[i])) throw std::runtime_error("tile view registry is full");
      [members addObject:@{@"textureId":@(identity.textures[i]),@"deviceGeneration":@(generation),
          @"initializing":@YES,@"ready":@NO,@"presentationKind":@"nativeView",
          @"closeAcknowledgementRequired":@YES,
          @"viewType":@"rhwp_native/metal_view",@"limits":MetalViewLimits()}];
    }
    views[0]->tilePeer = views[1];
    views[1]->tilePeer = views[0];
    NSDictionary* reply = @{@"groupId":@(identity.id()),@"deviceGeneration":@(generation),@"members":members};
    _views = std::move(records);
    _unexposedCreations = identity.textures;
    std::array<bool, 2> started{};
    try {
      for (size_t i = 0; i < 2; ++i) {
        const auto view = views[i];
        std::thread([view] {
          try {RunMetalView(view);}
          catch (const std::exception&) {
            view->owner->fail(@"Native Metal tile owner stopped unexpectedly; completion remains unconfirmed.");
            view->owner->close();
          }
        }).detach();
        started[i] = true;
      }
    } catch (...) {
      for (size_t i = 0; i < 2; ++i) {
        _views.close(identity.textures[i]);
        StopMetalView(views[i]);
        if (!started[i]) {
          std::lock_guard<std::mutex> lock(views[i]->owner->mutex);
          views[i]->owner->finished = views[i]->flow.finish_owner();
          views[i]->owner->slot.reset();
        }
      }
      throw;
    }
    _tileGroup = identity;
    _tileTerminal = nil;
    _unexposedCreations = {};
    result(reply);return;
  }

  uint64_t group = 0, generation = 0;
  if (!Unsigned(args[@"groupId"], group) || !_tileGroup.matches(group) ||
      !Unsigned(args[@"deviceGeneration"], generation) || generation != _tileGroup.generation) {
    result([FlutterError errorWithCode:@"contextChanged" message:@"The tile group ID and device generation must match." details:nil]);return;
  }
  if (_tileTerminal) {
    if ([call.method isEqualToString:@"statusGroup"] || [call.method isEqualToString:@"closeGroup"] ||
        [call.method isEqualToString:@"retireGroupAck"]) result(_tileTerminal);
    else result([FlutterError errorWithCode:@"closed" message:@"The tile group has retired." details:nil]);
    return;
  }
  std::array<std::shared_ptr<MetalViewState>, 2> views{
      _views.lookup(_tileGroup.textures[0]), _views.lookup(_tileGroup.textures[1])};
  if (!views[0] || !views[1]) {
    result([FlutterError errorWithCode:@"closed" message:@"Both tile owners must remain observable." details:nil]);return;
  }
  if ([call.method isEqualToString:@"submitGroup"]) {
    id tilesValue = args[@"tiles"];
    uint64_t clear = 0;
    if (![tilesValue isKindOfClass:[NSArray class]] || [tilesValue count] != 2 ||
        !Unsigned(args[@"clearColor"], clear, UINT32_MAX) || (clear >> 24) != 0xff) {
      result([FlutterError errorWithCode:@"arguments" message:@"Two bounded tiles and an opaque clear color are required." details:nil]);return;
    }
    if (!_views.registered(_tileGroup.textures[0]) || !_views.registered(_tileGroup.textures[1])) {
      result(@{@"accepted":@NO,@"reason":@"closed",@"limits":MetalViewLimits()});return;
    }
    NSArray* tiles = tilesValue;
    std::array<std::shared_ptr<Job>, 2> jobs;
    for (size_t i = 0; i < 2; ++i) {
      uint64_t handle = 0;
      if (![tiles[i] isKindOfClass:[NSDictionary class]] || !Unsigned(tiles[i][@"textureId"], handle) ||
          handle != _tileGroup.textures[i]) {
        result([FlutterError errorWithCode:@"arguments" message:@"Tile identity and order must match creation." details:nil]);return;
      }
      NSMutableDictionary* tile = [tiles[i] mutableCopy];
      tile[@"stamp"] = args[@"stamp"];
      tile[@"clearColor"] = args[@"clearColor"];
      jobs[i] = ParseJob(tile, generation, false);
      if (!jobs[i]) {
        result([FlutterError errorWithCode:@"arguments" message:@"Invalid full stamp, placements or bounded tile geometry." details:nil]);return;
      }
      std::shared_ptr<NativeApi> api;
      {std::lock_guard<std::mutex> lock(views[i]->owner->mutex);api = views[i]->owner->api;}
      if (!api || !MetalJobCurrent(api, jobs[i])) {
        result([FlutterError errorWithCode:@"staleFrame" message:@"Tile source/view/device stamp is not current." details:nil]);return;
      }
    }
    const auto ids = rhwp::tiles::unique_snapshots({jobs[0]->ids, jobs[1]->ids});
    NSMutableArray* acceptedIds = [NSMutableArray arrayWithCapacity:ids.size()];
    for (const auto id : ids) [acceptedIds addObject:@(id)];
    NSDictionary* acceptedReply = @{@"accepted":@YES,@"acceptedStamp":StampValue(jobs[0]->stamp),
        @"acceptedSnapshots":acceptedIds,@"limits":MetalViewLimits()};
    // Hide both actual AppKit views before either producer can acquire its new
    // pending job. Hiding after enqueue can expose a fast tile before its peer.
    CoverMetalViews(views[0]);
    const bool accepted = rhwp::metal::EnqueueMetalTileGroup(
        views[0]->owner, views[1]->owner, jobs[0], jobs[1],
        [](uint64_t id, const std::shared_ptr<NativeApi>& api) {
          auto lease = std::make_shared<SnapshotLease>();
          lease->id = id;
          lease->api = api;
          return lease;
        });
    result(accepted ? acceptedReply : @{@"accepted":@NO,@"reason":@"notReadyOrStale",@"limits":MetalViewLimits()});return;
  }

  if ([call.method isEqualToString:@"closeGroup"]) {
    CoverMetalViews(views[0]);
    std::array<std::shared_ptr<Job>, 2> pending;
    {
      std::scoped_lock lock(views[0]->owner->mutex, views[1]->owner->mutex);
      for (size_t i = 0; i < 2; ++i) {
        auto owner = views[i]->owner;
        owner->closed = true;
        owner->ready = false;
        owner->gate.close();
        pending[i] = std::move(owner->pending);
        views[i]->visibility.close();
        _views.close(_tileGroup.textures[i]);
      }
    }
    ScopedMetalViewAnimations immediate;
    for (const auto& view : views) StopMetalView(view);
  }

  NSMutableArray* members = [NSMutableArray arrayWithCapacity:2];
  std::array<NSView*, 2> native{};
  bool hide = false;
  const bool covering = [call.method isEqualToString:@"setCoveredGroup"];
  const bool retiring = [call.method isEqualToString:@"retireGroupAck"];
  uint64_t serial = 0;
  bool wantCover = false;
  if (covering) {
    id covered = args[@"covered"];
    if (![covered isKindOfClass:[NSNumber class]] || CFGetTypeID((__bridge CFTypeRef)covered) != CFBooleanGetTypeID() ||
        !Unsigned(args[@"visibilitySerial"], serial) || !serial) {
      result([FlutterError errorWithCode:@"arguments" message:@"A boolean cover and positive visibility serial are required." details:nil]);return;
    }
    wantCover = [covered boolValue];
  }
  {
    std::scoped_lock lock(views[0]->owner->mutex, views[1]->owner->mutex);
    const bool current = rhwp::metal::TileFramesCurrent(
        MetalViewCurrentLocked(views[0]), views[0]->renderedStamp,
        MetalViewCurrentLocked(views[1]), views[1]->renderedStamp);
    if (!current) for (const auto& view : views) view->visibility.invalidate();
    bool accepted = false;
    if (covering) {
      const bool stampMatches = current && [args[@"stamp"] isKindOfClass:[NSDictionary class]] &&
          [args[@"stamp"] isEqual:StampValue(views[0]->renderedStamp)];
      accepted = rhwp::metal::ApplyTileCover(views[0]->visibility, views[1]->visibility, serial,
          wantCover, stampMatches, views[0]->owner->closed || views[1]->owner->closed).accepted;
    }
    if (retiring) {
      bool complete = true;
      for (size_t i = 0; i < 2; ++i) {
        const auto& view = views[i];
        const auto& owner = view->owner;
        complete = complete && !_views.registered(_tileGroup.textures[i]) && owner->closed && owner->finished &&
            !owner->inFlight && view->layerDetached && !view->flow.quarantined();
      }
      if (!complete) {
        result([FlutterError errorWithCode:@"retirePending" message:@"Both producer completions and layer detachments must be confirmed." details:nil]);return;
      }
    }
    hide = views[0]->visibility.covered() || views[1]->visibility.covered();
    if (hide) for (const auto& view : views) view->visibility.invalidate();
    for (size_t i = 0; i < 2; ++i) {
      native[i] = views[i]->view;
      NSMutableDictionary* member = [MetalViewStatusLocked(views[i], _views.registered(_tileGroup.textures[i]), current) mutableCopy];
      member[@"textureId"] = @(_tileGroup.textures[i]);
      if (covering) member[@"accepted"] = @(accepted);
      if (retiring) member[@"acknowledged"] = @YES;
      [members addObject:member];
    }
  }
  // One CA transaction applies both changes; producer readiness remains
  // distinct from Core Animation/consumer GPU completion.
  {ScopedMetalViewAnimations immediate;for (NSView* view : native) view.hidden = hide;}
  NSDictionary* reply = @{@"groupId":@(group),@"deviceGeneration":@(generation),@"members":members};
  if (retiring) {
    // Allocate both receipts in a private copy before retiring either owner.
    // A failure cannot leave one member acknowledged and the other live.
    if (!rhwp::metal::AcknowledgeMetalTileGroup(_views, _tileGroup, true,
        std::array<MetalViewReceipt, 2>{MetalViewReceipt{members[0]}, MetalViewReceipt{members[1]}}))
      throw std::runtime_error("tile retirement changed during acknowledgment");
    _tileTerminal = reply;
  }
  result(reply);
}
