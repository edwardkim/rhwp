// Actual Apple SDK producer probe. This is not a Flutter consumer fence.
#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

#include "rhwp_native_ffi.h"
#include <cstring>
#include <cstdlib>
#include <dlfcn.h>
#include <memory>
#include <stdexcept>
#include <vector>

namespace {
template <typename Function>
Function symbol(void* library, const char* name) {
  auto* address = dlsym(library, name);
  if (!address) throw std::runtime_error("Native Metal symbol is unavailable");
  Function output;
  static_assert(sizeof(output) == sizeof(address), "unsupported loader ABI");
  std::memcpy(&output, &address, sizeof(output));
  return output;
}

char* json(NSDictionary* value) {
  NSData* bytes = [NSJSONSerialization dataWithJSONObject:value options:0 error:nil];
  if (!bytes) return nullptr;
  auto* output = static_cast<char*>(std::malloc(bytes.length + 1));
  if (!output) return nullptr;
  std::memcpy(output, bytes.bytes, bytes.length);
  output[bytes.length] = 0;
  return output;
}

struct Api {
  explicit Api(const char* path) {
    // Process pinning is deliberate: the Python owner and its Rust TLS use this
    // exact module. A test never unloads a library underneath either owner.
    library = dlopen(path, RTLD_NOW | RTLD_LOCAL);
    if (!library) throw std::runtime_error("Native dylib could not be opened");
    caps = symbol<decltype(&rhwp_metal_capabilities)>(library, "rhwp_metal_capabilities");
    create = symbol<decltype(&rhwp_metal_presenter_create)>(library, "rhwp_metal_presenter_create");
    render = symbol<decltype(&rhwp_metal_presenter_render)>(library, "rhwp_metal_presenter_render");
    request = symbol<decltype(&rhwp_metal_presenter_request)>(library, "rhwp_metal_presenter_request");
    free = symbol<decltype(&rhwp_string_free)>(library, "rhwp_string_free");
  }
  NSDictionary* read(char* raw) const {
    if (!raw) throw std::runtime_error("Native returned null JSON");
    std::unique_ptr<char, decltype(free)> owned(raw, free);
    const size_t length = strnlen(raw, 64 * 1024 + 1);
    if (length > 64 * 1024) throw std::runtime_error("Native reply exceeds the probe bound");
    NSData* data = [NSData dataWithBytes:raw length:length];
    id value = [NSJSONSerialization JSONObjectWithData:data options:0 error:nil];
    if (![value isKindOfClass:[NSDictionary class]]) throw std::runtime_error("Native reply is not JSON object");
    return value;
  }
  void* library = nullptr;
  decltype(&rhwp_metal_capabilities) caps = nullptr;
  decltype(&rhwp_metal_presenter_create) create = nullptr;
  decltype(&rhwp_metal_presenter_render) render = nullptr;
  decltype(&rhwp_metal_presenter_request) request = nullptr;
  decltype(&rhwp_string_free) free = nullptr;
};

// On an unproven GPU failure a disposable test process holds its one possible
// in-flight target and Native owner until exit. It never guesses safe release.
NSMutableArray* quarantine() {
  static NSMutableArray* retained = [[NSMutableArray alloc] init];
  return retained;
}
}

extern "C" char* rhwp_probe_metal_sdk(const char* path, const uint8_t* request, size_t length) {
  @autoreleasepool {
    try {
      if (!path || !request || !length || length > 64 * 1024 || quarantine().count >= 1) {
        throw std::runtime_error("invalid probe input or previous unsafe GPU failure");
      }
      NSData* input = [NSData dataWithBytes:request length:length];
      NSDictionary* value = [NSJSONSerialization JSONObjectWithData:input options:0 error:nil];
      if (![value isKindOfClass:[NSDictionary class]]) throw std::runtime_error("render request is not an object");
      NSDictionary* target = value[@"target"];
      if (![target isKindOfClass:[NSDictionary class]]) throw std::runtime_error("missing target description");
      const auto width = [target[@"width"] unsignedLongLongValue];
      const auto height = [target[@"height"] unsignedLongLongValue];
      const auto generation = [target[@"deviceGeneration"] unsignedLongLongValue];
      if (!width || !height || width > 256 || height > 256 || !generation) {
        throw std::runtime_error("invalid probe target dimensions/generation");
      }
      Api api(path);
      NSDictionary* capabilities = api.read(api.caps());
      if (![capabilities[@"ok"] boolValue] || ![capabilities[@"available"] boolValue]) {
        throw std::runtime_error("explicit native-skia-metal build is required");
      }
      id<MTLDevice> device = MTLCreateSystemDefaultDevice();
      id<MTLCommandQueue> queue = [device newCommandQueue];
      if (!device || !queue) throw std::runtime_error("no actual Metal device/queue");
      MTLTextureDescriptor* descriptor = [MTLTextureDescriptor
          texture2DDescriptorWithPixelFormat:MTLPixelFormatBGRA8Unorm
          width:width height:height mipmapped:NO];
      descriptor.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
      descriptor.storageMode = device.hasUnifiedMemory ? MTLStorageModeShared : MTLStorageModeManaged;
      id<MTLTexture> texture = [device newTextureWithDescriptor:descriptor];
      if (!texture) throw std::runtime_error("Metal target allocation failed");
      const size_t bytes = size_t(width * height * 4);
      std::vector<uint8_t> pixels(bytes, 0x5a);
      [texture replaceRegion:MTLRegionMake2D(0, 0, width, height) mipmapLevel:0
                   withBytes:pixels.data() bytesPerRow:width * 4];
      NSDictionary* created = api.read(api.create((__bridge const void*)device,
          (__bridge const void*)queue, generation, nullptr, 0));
      if (![created[@"ok"] boolValue]) throw std::runtime_error("Native Metal context creation failed");
      const uint64_t presenter = [created[@"presenter"] unsignedLongLongValue];
      NSDictionary* rendered = nil;
      try {
        rendered = api.read(api.render(presenter, (__bridge const void*)texture, request, length));
      } catch (...) {
        [quarantine() addObject:@{@"device":device, @"queue":queue, @"texture":texture}];
        throw;  // An unreadable reply is not proof that the GPU target is idle.
      }
      const bool success = [rendered[@"ok"] boolValue];
      const bool safe_stale = [rendered[@"code"] isEqualToString:@"staleFrame"] &&
          [rendered[@"unpublishedTargetSafeToDiscard"] boolValue];
      if (!success && !safe_stale) {
        [quarantine() addObject:@{@"device":device, @"queue":queue, @"texture":texture}];
        return json(@{@"ok":@NO,@"native":rendered,@"unsafeFailureQuarantined":@YES});
      }
      if (success && ![rendered[@"frame"][@"producerGpuComplete"] boolValue]) {
        [quarantine() addObject:@{@"device":device, @"queue":queue, @"texture":texture}];
        return json(@{@"ok":@NO,@"error":@"Native did not prove producer completion",@"unsafeFailureQuarantined":@YES});
      }
      if (descriptor.storageMode == MTLStorageModeManaged) {
        id<MTLCommandBuffer> command = [queue commandBuffer];
        id<MTLBlitCommandEncoder> blit = [command blitCommandEncoder];
        [blit synchronizeResource:texture]; [blit endEncoding];
        [command commit]; [command waitUntilCompleted];
        if (command.status != MTLCommandBufferStatusCompleted) {
          [quarantine() addObject:@{@"device":device, @"queue":queue, @"texture":texture}];
          return json(@{@"ok":@NO,@"error":@"probe readback synchronization failed",@"unsafeFailureQuarantined":@YES});
        }
      }
      [texture getBytes:pixels.data() bytesPerRow:width * 4
              fromRegion:MTLRegionMake2D(0, 0, width, height) mipmapLevel:0];
      static const uint8_t release[] = "{\"op\":\"release\"}";
      NSDictionary* released = api.read(api.request(presenter, release, sizeof(release) - 1));
      if (![released[@"ok"] boolValue]) throw std::runtime_error("Native presenter release failed");
      NSData* data = [NSData dataWithBytes:pixels.data() length:pixels.size()];
      return json(@{@"ok":@YES,@"native":rendered,@"deviceName":device.name ?: @"",
        @"lowPower":@(device.lowPower),@"width":@(width),@"height":@(height),
        @"hostVerificationReadbacks":@1,@"consumerGpuCompletionVerified":@NO,
        @"bgraBase64":[data base64EncodedStringWithOptions:0]});
    } catch (const std::exception& error) {
      return json(@{@"ok":@NO,@"error":[NSString stringWithUTF8String:error.what()] ?: @"probe error"});
    }
  }
}

extern "C" void rhwp_probe_metal_free(char* output) { std::free(output); }
