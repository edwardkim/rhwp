#include "../../common/producer_ready.h"
#include <atomic>

@interface RhwpProducerReadyEvents : NSObject <FlutterStreamHandler>
- (instancetype)initWithMessenger:(id<FlutterBinaryMessenger>)messenger name:(NSString*)name;
- (void)offer:(const rhwp::ProducerReadyFrame&)frame;
- (void)shutdown;
@end
@implementation RhwpProducerReadyEvents {
  rhwp::ProducerReadyMailbox _mailbox;
  FlutterEventChannel* _channel;
  FlutterEventSink _sink;
  std::atomic<bool> _stopping;
}
- (instancetype)initWithMessenger:(id<FlutterBinaryMessenger>)messenger name:(NSString*)name {
  self = [super init];
  if (self) {
    _stopping.store(false);
    _channel = [FlutterEventChannel eventChannelWithName:name binaryMessenger:messenger];
    [_channel setStreamHandler:self];
  }
  return self;
}
- (FlutterError*)onListenWithArguments:(id)arguments eventSink:(FlutterEventSink)events {
  (void)arguments;
  if (_stopping.load()) return nil;
  _sink = [events copy]; _mailbox.listen(true); return nil;
}
- (FlutterError*)onCancelWithArguments:(id)arguments {
  (void)arguments;
  _mailbox.listen(false); _sink = nil; return nil;
}
- (void)shutdown {
  _stopping = true;
  _mailbox.listen(false);
  // Plugin teardown can originate off-main. Retain the event owner until the
  // platform messenger and sink are detached on their owning thread.
  dispatch_block_t detach = ^{
    self->_mailbox.listen(false);
    self->_sink = nil;
    [self->_channel setStreamHandler:nil]; self->_channel = nil;
  };
  if ([NSThread isMainThread]) detach();
  else dispatch_async(dispatch_get_main_queue(), detach);
}
- (void)offer:(const rhwp::ProducerReadyFrame&)frame {
  try {
    if (!_mailbox.offer(frame)) return;
    __weak RhwpProducerReadyEvents* weakSelf = self;
    dispatch_async(dispatch_get_main_queue(), ^{
      RhwpProducerReadyEvents* owner = weakSelf;
      if (!owner) return;
      try {
        for (const auto& ready : owner->_mailbox.take()) {
          if (!owner->_sink) break;
          owner->_sink(@{@"event":@"producerReady",@"textureId":@(ready.texture),
              @"deviceGeneration":@(ready.device),@"serial":@(ready.serial),
              @"stamp":@{@"documentId":@(ready.document),@"revision":@(ready.revision),
                @"fontGeneration":@(ready.font),@"viewGeneration":@(ready.view),
                @"deviceGeneration":@(ready.device),@"deviceScale":@(ready.scale)}});
        }
      } catch (...) { owner->_mailbox.listen(false); }
    });
  } catch (...) { _mailbox.listen(false); }
}
- (void)dealloc { _mailbox.listen(false); }
@end

static auto ProducerReadyCallback(RhwpProducerReadyEvents* events, uint64_t texture) {
  __weak RhwpProducerReadyEvents* weakEvents = events;
  return [weakEvents, texture](uint64_t serial, const auto& stamp) {
    RhwpProducerReadyEvents* current = weakEvents;
    if (current) [current offer:rhwp::ProducerReadyFrame{texture, stamp.device, serial,
        stamp.document, stamp.revision, stamp.font, stamp.view, stamp.scale}];
  };
}
