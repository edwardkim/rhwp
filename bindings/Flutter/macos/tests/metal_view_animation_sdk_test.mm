// Actual Apple SDK/AppKit/Core Animation regression, not GPU/display evidence.
// An old plugin can be compiled with this same test by overriding this include
// macro. Both versions execute real CA action lookup under an animated context.
#import <AppKit/AppKit.h>
#import <QuartzCore/CAAnimation.h>
#import <QuartzCore/CATransaction.h>
#ifndef RHWP_METAL_ANIMATION_PLUGIN_SOURCE
#define RHWP_METAL_ANIMATION_PLUGIN_SOURCE "../Classes/RhwpMetalPlugin.mm"
#endif
#include RHWP_METAL_ANIMATION_PLUGIN_SOURCE
#include <iostream>

static bool noImplicitActions(CALayer* layer) {
  for (NSString* key in @[@"opacity",@"bounds",@"position",@"hidden",@"contents",
                          @"drawableSize",@"onOrderIn",@"onOrderOut"]) {
    if ([layer actionForKey:key] != nil) {
      std::cerr << "implicit action lookup remained enabled: " << key.UTF8String << "\n";
      return false;
    }
  }
  return true;
}

int main() {
  @autoreleasepool {
    if (![NSThread isMainThread]) return 2;
    NSApplication* application = [NSApplication sharedApplication];
    (void)[application setActivationPolicy:NSApplicationActivationPolicyProhibited];
    if (application.activationPolicy != NSApplicationActivationPolicyProhibited || application.active) return 3;
    [application finishLaunching];
    if (application.activationPolicy != NSApplicationActivationPolicyProhibited || application.active) return 4;

    // Seed a real CA fallback action so an inherited nil/default lookup cannot
    // accidentally pass because an unattached test layer has no presentation.
    RhwpOwnedMetalLayer* standalone = [[RhwpOwnedMetalLayer alloc] initWithOwnerSlot:nullptr];
    standalone.actions = @{@"opacity":[CABasicAnimation animationWithKeyPath:@"opacity"],
                           @"bounds":[CABasicAnimation animationWithKeyPath:@"bounds"]};
    [NSAnimationContext beginGrouping];
    [NSAnimationContext currentContext].duration = 2;
    [NSAnimationContext currentContext].allowsImplicitAnimation = YES;
    [CATransaction begin];
    [CATransaction setDisableActions:NO];
    [CATransaction setAnimationDuration:2];
    const bool standaloneClosed = noImplicitActions(standalone);
    standalone.opacity = .25;
    standalone.opacity = 1;
    standalone.bounds = CGRectMake(0,0,80,60);
    [CATransaction commit];
    [NSAnimationContext endGrouping];
    [CATransaction flush];
    if (!standaloneClosed) return 10;
    if (standalone.animationKeys.count) return 11;

    auto owner = std::make_shared<State>(1,"unused",nullptr);
    auto state = std::make_shared<MetalViewState>(owner);
    state->device = MTLCreateSystemDefaultDevice();
    if (!state->device) return 12;
    RhwpNativeMetalView* view = [[RhwpNativeMetalView alloc] initWithState:state];
    NSWindow* window = [[NSWindow alloc] initWithContentRect:NSMakeRect(0,0,80,60)
        styleMask:NSWindowStyleMaskBorderless backing:NSBackingStoreBuffered defer:NO];
    window.releasedWhenClosed = NO;
    window.contentView = view;
    CAMetalLayer* layer = (CAMetalLayer*)view.layer;

    [NSAnimationContext beginGrouping];
    [NSAnimationContext currentContext].duration = 2;
    [NSAnimationContext currentContext].allowsImplicitAnimation = YES;
    [CATransaction begin];
    [CATransaction setDisableActions:NO];
    [CATransaction setAnimationDuration:2];
    [view setHidden:NO];
    [view setFrameSize:NSMakeSize(40,30)];
    [view updateMetalGeometry];
    const bool ownedClosed = noImplicitActions(layer);
    const bool modelReady = !view.hidden && layer.opacity == 1 &&
        layer.drawableSize.width == 40 * window.backingScaleFactor &&
        layer.drawableSize.height == 30 * window.backingScaleFactor;
    const bool contextRestored = [NSAnimationContext currentContext].duration == 2 &&
        [NSAnimationContext currentContext].allowsImplicitAnimation && ![CATransaction disableActions];
    [view setHidden:YES];
    [view setHidden:NO];
    [view detachMetalLayer];
    const bool detached = view.hidden && !view.layer;
    [CATransaction commit];
    [NSAnimationContext endGrouping];
    [CATransaction flush];
    const bool noAnimations = !layer.animationKeys.count && !standalone.animationKeys.count;
    window.contentView = nil;
    [window close];
    if (!ownedClosed) return 20;
    if (!modelReady) return 21;
    if (!contextRestored) return 22;
    if (!detached || !noAnimations) return 23;
    std::cout << "Owned Metal view: implicit actions blocked, immediate geometry/visibility, ambient context restored PASS\n";
  }
}
