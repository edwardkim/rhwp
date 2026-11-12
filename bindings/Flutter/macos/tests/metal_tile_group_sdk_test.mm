// Actual Apple SDK/AppKit/Flutter method bridge and view cover/close contracts.
// The deliberately unavailable Native library performs no GPU rendering. This
// executable is not evidence of Metal output or Flutter compositor sampling.
#include "../Classes/RhwpMetalPlugin.mm"
#include <iostream>

namespace {
size_t checks = 0;
void RequireTile(bool condition) {
  ++checks;
  if (!condition) throw std::runtime_error("Metal tile SDK check " + std::to_string(checks));
}
id invoke(RhwpMetalViewPlugin* plugin, NSString* method, NSDictionary* args) {
  __block id reply = nil;
  [plugin handleMethodCall:[FlutterMethodCall methodCallWithMethodName:method arguments:args]
      result:^(id value) {reply = value;}];
  RequireTile(reply != nil);
  return reply;
}
bool errorCode(id value, NSString* code) {
  return [value isKindOfClass:[FlutterError class]] && [((FlutterError*)value).code isEqual:code];
}
NSDictionary* closeAndAcknowledge(RhwpMetalViewPlugin* plugin, NSDictionary* group) {
  id closed = invoke(plugin, @"closeGroup", group);
  RequireTile([closed isKindOfClass:[NSDictionary class]]);
  for (NSDictionary* member in closed[@"members"]) {
    RequireTile([member[@"closed"] boolValue] && ![member[@"registered"] boolValue] &&
        [member[@"nativeViewLayerDetached"] boolValue] && [member[@"covered"] boolValue]);
  }
  for (int attempt = 0; attempt < 1000; ++attempt) {
    id ack = invoke(plugin, @"retireGroupAck", group);
    if ([ack isKindOfClass:[NSDictionary class]]) {
      for (NSDictionary* member in ack[@"members"]) RequireTile([member[@"acknowledged"] boolValue] && [member[@"ownerFinished"] boolValue]);
      RequireTile([invoke(plugin, @"statusGroup", group) isEqual:ack]);
      RequireTile([invoke(plugin, @"closeGroup", group) isEqual:ack]);
      return ack;
    }
    RequireTile(errorCode(ack, @"retirePending"));
    [[NSRunLoop currentRunLoop] runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.005]];
  }
  throw std::runtime_error("both native tile owners did not finish within five seconds");
}
}  // namespace

int main() {
  @autoreleasepool {
    RequireTile([NSThread isMainThread]);
    NSApplication* application = [NSApplication sharedApplication];
    [application setActivationPolicy:NSApplicationActivationPolicyProhibited];
    [application finishLaunching];
    RhwpMetalViewPlugin* plugin = [[RhwpMetalViewPlugin alloc] init];
    NSDictionary* capabilities = invoke(plugin, @"capabilities", @{});
    RequireTile([capabilities[@"tileGroups"] boolValue] && [capabilities[@"sharedSnapshotLeases"] boolValue]);
    RequireTile([capabilities[@"maxViews"] intValue] == 2 && [capabilities[@"maxTileGroupTiles"] intValue] == 2);
    RequireTile(![capabilities[@"consumerGpuCompletionVerified"] boolValue]);
    RequireTile(![CVLimits(false)[@"continuousPresentationSupported"] boolValue]);
    const auto credits = Credits().stats();
    NSDictionary* createArgs = @{@"libraryPath":@"/__rhwp_sdk_test_missing_native__.dylib"};
    NSDictionary* created = invoke(plugin, @"createGroup", createArgs);
    RequireTile([created isKindOfClass:[NSDictionary class]] && [created[@"members"] count] == 2);
    NSArray* members = created[@"members"];
    const uint64_t first = [members[0][@"textureId"] unsignedLongLongValue];
    const uint64_t second = [members[1][@"textureId"] unsignedLongLongValue];
    RequireTile(first && second && first != second && [created[@"groupId"] unsignedLongLongValue] == first);
    RequireTile([members[0][@"deviceGeneration"] isEqual:members[1][@"deviceGeneration"]]);
    NSDictionary* group = @{@"groupId":created[@"groupId"],@"deviceGeneration":created[@"deviceGeneration"]};
    RequireTile(errorCode(invoke(plugin, @"createGroup", createArgs), @"backpressure"));
    RequireTile(errorCode(invoke(plugin, @"create", createArgs), @"backpressure"));
    for (NSString* method in @[@"submit",@"setCovered",@"close",@"retireAck",@"status"]) {
      RequireTile(errorCode(invoke(plugin, method, @{@"textureId":@(first)}), @"contextChanged"));
    }
    RequireTile(errorCode(invoke(plugin, @"statusGroup", @{@"groupId":group[@"groupId"],@"deviceGeneration":@0}), @"contextChanged"));
    RequireTile(errorCode(invoke(plugin, @"retireGroupAck", group), @"retirePending"));
    RhwpNativeMetalView* a = (RhwpNativeMetalView*)[plugin createWithViewIdentifier:1 arguments:@{@"handle":@(first)}];
    RhwpNativeMetalView* b = (RhwpNativeMetalView*)[plugin createWithViewIdentifier:2 arguments:@{@"handle":@(second)}];
    RequireTile([a isKindOfClass:[RhwpNativeMetalView class]] && [b isKindOfClass:[RhwpNativeMetalView class]]);
    RequireTile(a.hidden && b.hidden);
    NSWindow* window = [[NSWindow alloc] initWithContentRect:NSMakeRect(0, 0, 80, 40)
        styleMask:NSWindowStyleMaskBorderless backing:NSBackingStoreBuffered defer:NO];
    window.releasedWhenClosed = NO;
    NSView* container = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, 80, 40)];
    window.contentView = container;
    a.frame = NSMakeRect(0, 0, 40, 40);
    b.frame = NSMakeRect(40, 0, 40, 40);
    [container addSubview:a];[container addSubview:b];
    a.hidden = NO;b.hidden = NO;
    const auto unchangedBounds = a.bounds;
    [a removeFromSuperview]; // No dealloc/close and no change to point bounds.
    RequireTile(NSEqualRects(a.bounds, unchangedBounds) && a.hidden && b.hidden);
    [container addSubview:a];
    a.hidden = NO;b.hidden = NO;
    NSDictionary* status = invoke(plugin, @"statusGroup", group);
    RequireTile(a.hidden && b.hidden);
    for (NSDictionary* member in status[@"members"]) RequireTile(![member[@"hasFrame"] boolValue] && [member[@"covered"] boolValue]);
    NSMutableDictionary* cover = [group mutableCopy];
    cover[@"covered"] = @NO;cover[@"visibilitySerial"] = @1;cover[@"stamp"] = @{};
    NSDictionary* rejected = invoke(plugin, @"setCoveredGroup", cover);
    for (NSDictionary* member in rejected[@"members"]) RequireTile(![member[@"accepted"] boolValue] && [member[@"covered"] boolValue] && [member[@"visibilitySerial"] intValue] == 1);
    RequireTile(a.hidden && b.hidden);
    cover[@"covered"] = @YES;cover[@"visibilitySerial"] = @2;
    NSDictionary* covered = invoke(plugin, @"setCoveredGroup", cover);
    for (NSDictionary* member in covered[@"members"]) RequireTile([member[@"accepted"] boolValue] && [member[@"covered"] boolValue]);
    closeAndAcknowledge(plugin, group);
    RequireTile(!a.layer && !b.layer && a.hidden && b.hidden);
    window.contentView = nil;[window close];
    RequireTile(errorCode(invoke(plugin, @"submitGroup", group), @"closed"));
    NSDictionary* next = invoke(plugin, @"createGroup", createArgs);
    RequireTile([next isKindOfClass:[NSDictionary class]] && ![next[@"deviceGeneration"] isEqual:group[@"deviceGeneration"]]);
    RequireTile(errorCode(invoke(plugin, @"closeGroup", group), @"contextChanged"));
    closeAndAcknowledge(plugin, @{@"groupId":next[@"groupId"],@"deviceGeneration":next[@"deviceGeneration"]});
    RequireTile(Credits().stats().outstanding_frames == credits.outstanding_frames);
    RequireTile(Credits().stats().reserved_bytes == credits.reserved_bytes);
    std::cout << "Apple SDK tile group bridge: " << checks << " checks passed; no GPU/compositor claim\n";
  }
}
