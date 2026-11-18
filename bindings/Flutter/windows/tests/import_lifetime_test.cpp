#include "SurfaceImportTickets.h"
#include <atomic>
#include <functional>
#include <iostream>
#include <set>
#include <thread>
#include <vector>

using namespace rhwp::d3d;
namespace {
int checks = 0;
#define CHECK(value) do { ++checks; if (!(value)) throw std::runtime_error("check failed: " #value); } while (0)
struct Descriptor {
  void* handle = nullptr;
  size_t width = 0, height = 0, visible_width = 0, visible_height = 0;
  void (*release_callback)(void*) = nullptr;
  void* release_context = nullptr;
};
using Imports = SurfaceImportTickets<Descriptor>;
struct Frame {
  std::shared_ptr<ImmutableFrames::Reservation> credit;
  std::shared_ptr<int> pixels;
};
std::shared_ptr<Frame> publish(ImmutableFrames& pool, int value, size_t bytes = 128 * 1024) {
  auto frame = std::make_shared<Frame>();
  frame->credit = pool.reserve(bytes, PublicationMode::ImmutableOwnershipTransfer);
  CHECK(frame->credit);
  frame->pixels = std::make_shared<int>(value);
  CHECK(frame->credit->begin_gpu(frame->pixels));
  CHECK(frame->credit->producer_complete() && frame->credit->publish_immutable());
  return frame;
}
Descriptor description(int value) {
  Descriptor result;
  result.handle = reinterpret_cast<void*>(uintptr_t(value + 1));
  result.width = result.visible_width = size_t(96 + value);
  result.height = result.visible_height = 64;
  return result;
}
const Descriptor* offer(uint64_t owner, const std::shared_ptr<Frame>& frame, int value) {
  return Imports::offer(owner, description(value), frame, frame->credit);
}
void engine_reads_dimensions_after_opening_callback() {
  constexpr uint64_t owner = 100;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  auto frame = publish(pool, 7);
  std::weak_ptr<Frame> weak_frame = frame;
  auto consumer_pixels = frame->pixels;  // Independent consumer COM reference.
  const Descriptor* descriptor;
  {
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    descriptor = offer(owner, frame, 7); CHECK(descriptor);
  }  // Plugin returned. The engine has not read all descriptor fields yet.
  const auto notification = descriptor->release_callback;
  void* cookie = descriptor->release_context;
  notification(cookie);
  frame.reset();  // Concurrent producer supersedes the frame after opening.
  CHECK(!weak_frame.expired() && pool.stats().producer_owned == 1);
  CHECK(descriptor->visible_width == 103 && descriptor->visible_height == 64);
  CHECK(*consumer_pixels == 7);
  {
    auto next_callback = Imports::begin_callback(owner); CHECK(next_callback);
    CHECK(!weak_frame.expired() && pool.stats().outstanding == 1);
    CHECK(Imports::stats().tickets == 0);  // CPU descriptor retired, cached HANDLE still pinned.
  }
  CHECK(pool.stats().handed_off == 0 && *consumer_pixels == 7);
  notification(cookie);  // Late duplicate never dereferences the old descriptor.
  CHECK(pool.stats().outstanding == 1);
  Imports::unregistered(owner);
  CHECK(weak_frame.expired() && pool.stats().handed_off == 1 && pool.stats().outstanding == 0);
}
void continuous_128_fresh_frames_remain_producer_bounded() {
  constexpr uint64_t owner = 101;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  std::shared_ptr<Frame> current;
  std::shared_ptr<int> consumer_pixels;
  std::vector<std::pair<void (*)(void*), void*>> old_notifications;
  std::set<void*> cookies;
  for (int index = 0; index < 128; ++index) {
    current = publish(pool, index);
    CHECK(pool.stats().outstanding <= 2 && pool.stats().producer_owned_bytes <= 2 * 128 * 1024);
    const Descriptor* descriptor;
    {
      auto callback = Imports::begin_callback(owner); CHECK(callback);
      descriptor = offer(owner, current, index); CHECK(descriptor);
    }
    CHECK(cookies.insert(descriptor->release_context).second);
    consumer_pixels = current->pixels;
    old_notifications.emplace_back(descriptor->release_callback, descriptor->release_context);
    descriptor->release_callback(descriptor->release_context);
    CHECK(descriptor->visible_width == size_t(96 + index));
    CHECK(*consumer_pixels == index && Imports::stats().tickets <= 1);
    CHECK(pool.stats().import_pending == 0 && pool.stats().immutable == 0);
  }
  Imports::unregistered(owner); current.reset();
  CHECK(pool.stats().outstanding == 0 && pool.stats().bytes == 0 && pool.stats().handed_off == 128);
  CHECK(*consumer_pixels == 127 && Imports::stats().tickets == 0);
  std::vector<std::thread> late;
  for (int thread = 0; thread < 4; ++thread) late.emplace_back([&, thread] {
    for (size_t index = size_t(thread); index < old_notifications.size(); index += 4)
      old_notifications[index].first(old_notifications[index].second);
  });
  for (auto& thread : late) thread.join();
  CHECK(pool.stats().outstanding == 0 && Imports::stats().tickets == 0);
}
void four_unopened_frames_apply_backpressure_until_real_opening() {
  constexpr uint64_t owner = 102;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  std::vector<std::pair<void (*)(void*), void*>> notifications;
  for (int index = 0; index < 4; ++index) {
    auto frame = publish(pool, index, 16 * 1024 * 1024);
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    const auto* descriptor = offer(owner, frame, index); CHECK(descriptor);
    notifications.emplace_back(descriptor->release_callback, descriptor->release_context);
  }
  CHECK(pool.stats().import_pending == 4 && pool.stats().import_pending_bytes == 64 * 1024 * 1024);
  CHECK(!pool.reserve(1, PublicationMode::ImmutableOwnershipTransfer));
  Imports::unregistered(owner);  // CPU quiescence is not an opening notification.
  CHECK(pool.stats().outstanding == 4 && Imports::stats().tickets == 4);
  std::thread opened([&] { notifications[0].first(notifications[0].second); }); opened.join();
  CHECK(pool.stats().outstanding == 3 && pool.stats().import_pending == 3);
  auto admitted = pool.reserve(1, PublicationMode::ImmutableOwnershipTransfer); CHECK(admitted); admitted.reset();
  for (size_t index = 1; index < notifications.size(); ++index)
    notifications[index].first(notifications[index].second);
  CHECK(pool.stats().outstanding == 0 && Imports::stats().tickets == 0);
}
void producer_uncertainty_cannot_be_opened_or_refunded() {
  constexpr uint64_t owner = 103;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  auto frame = std::make_shared<Frame>();
  frame->credit = pool.reserve(128 * 1024, PublicationMode::ImmutableOwnershipTransfer);
  frame->pixels = std::make_shared<int>(9); std::weak_ptr<int> weak = frame->pixels;
  CHECK(frame->credit->begin_gpu(frame->pixels));
  {
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    CHECK(!offer(owner, frame, 9));
  }
  CHECK(Imports::stats().tickets == 0 && !frame->credit->consumer_opened());
  frame.reset(); Imports::unregistered(owner);
  CHECK(!weak.expired() && pool.stats().quarantined == 1 && pool.stats().permanent == 1);
  CHECK(pool.stats().handed_off == 0 && pool.stats().producer_owned == 1);
}
void published_but_never_offered_is_safe_to_discard() {
  ImmutableFrames pool;
  auto frame = publish(pool, 10); std::weak_ptr<int> weak = frame->pixels;
  frame.reset();
  CHECK(weak.expired() && pool.stats().outstanding == 0 && pool.stats().unexposed_discarded == 1);
  CHECK(pool.stats().handed_off == 0);
}
void legacy_and_transfer_share_one_admission_limit() {
  constexpr uint64_t owner = 104;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  auto legacy = pool.reserve(128 * 1024);
  CHECK(legacy->begin_gpu(std::make_shared<int>(1)) && legacy->producer_complete() && legacy->publish_immutable());
  CHECK(!legacy->offer_to_consumer() && !legacy->consumer_opened()); legacy.reset();
  std::vector<std::pair<void (*)(void*), void*>> notifications;
  for (int index = 0; index < 3; ++index) {
    auto frame = publish(pool, index);
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    const auto* descriptor = offer(owner, frame, index); CHECK(descriptor);
    notifications.emplace_back(descriptor->release_callback, descriptor->release_context);
  }
  CHECK(!pool.reserve(1, PublicationMode::ImmutableOwnershipTransfer));
  for (const auto& notification : notifications) notification.first(notification.second);
  Imports::unregistered(owner);
  CHECK(pool.stats().outstanding == 1 && pool.stats().immutable == 1 && pool.stats().legacy_retained == 1);
  CHECK(pool.stats().producer_owned == 0 && pool.stats().handed_off == 3);
}
void descriptor_registry_is_bounded_independently_of_frame_count() {
  constexpr uint64_t owner = 105;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  auto frame = publish(pool, 6);
  std::vector<const Descriptor*> descriptors;
  {
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    for (size_t index = 0; index < Imports::kMaxTickets; ++index) {
      const auto* descriptor = offer(owner, frame, 6); CHECK(descriptor); descriptors.push_back(descriptor);
    }
    CHECK(!offer(owner, frame, 6));
  }
  CHECK(Imports::stats().tickets == Imports::kMaxTickets && pool.stats().outstanding == 1);
  for (const auto* descriptor : descriptors) {
    descriptor->release_callback(descriptor->release_context);
    CHECK(descriptor->visible_width == 102);
  }
  Imports::unregistered(owner); frame.reset();
  CHECK(Imports::stats().tickets == 0 && pool.stats().handed_off == 1);
}
void close_races_with_opening_but_not_descriptor_cpu_retirement() {
  constexpr uint64_t owner = 106;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  auto frame = publish(pool, 17); std::weak_ptr<Frame> weak = frame;
  const Descriptor* descriptor;
  {
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    descriptor = offer(owner, frame, 17); CHECK(descriptor);
  }
  const auto notify = descriptor->release_callback; void* cookie = descriptor->release_context;
  std::atomic<bool> start{false};
  std::thread closing([owned = std::move(frame), &start]() mutable {
    while (!start.load(std::memory_order_acquire)) std::this_thread::yield();
    owned.reset();  // close() drops current; it cannot retire a CPU descriptor.
  });
  std::thread opening([&] {
    while (!start.load(std::memory_order_acquire)) std::this_thread::yield();
    notify(cookie); notify(cookie);
  });
  start.store(true, std::memory_order_release); closing.join(); opening.join();
  CHECK(!weak.expired() && descriptor->visible_width == 113);
  CHECK(Imports::stats().cpu_pending == 1 && pool.stats().outstanding == 1);
  Imports::unregistered(owner);
  CHECK(weak.expired() && pool.stats().handed_off == 1 && Imports::stats().tickets == 0);
  notify(cookie); CHECK(pool.stats().outstanding == 0);
}
void thread_change_and_reentry_do_not_prove_cpu_quiescence() {
  constexpr uint64_t owner = 107;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  auto frame = publish(pool, 5); std::weak_ptr<Frame> weak = frame;
  const Descriptor* descriptor;
  {
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    bool refused = false;
    try { auto nested = Imports::begin_callback(owner); (void)nested; } catch (const std::exception&) { refused = true; }
    CHECK(refused);
    descriptor = offer(owner, frame, 5); CHECK(descriptor);
  }
  descriptor->release_callback(descriptor->release_context); frame.reset();
  std::atomic<bool> refused{false};
  std::thread moved([&] {
    try { auto callback = Imports::begin_callback(owner); (void)callback; }
    catch (const std::exception&) { refused = true; }
  }); moved.join();
  CHECK(refused && !weak.expired() && descriptor->visible_width == 101);
  Imports::unregistered(owner); CHECK(weak.expired() && pool.stats().outstanding == 0);
}
void premature_unregistration_stays_pinned() {
  constexpr uint64_t owner = 108;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  auto frame = publish(pool, 3); std::weak_ptr<Frame> weak = frame;
  const Descriptor* descriptor;
  {
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    descriptor = offer(owner, frame, 3); CHECK(descriptor);
    Imports::unregistered(owner);  // Contradicts an in-progress CPU callback.
  }
  descriptor->release_callback(descriptor->release_context); frame.reset();
  CHECK(!weak.expired() && descriptor->visible_width == 99 && Imports::stats().tickets == 1);
  { auto callback = Imports::begin_callback(owner); CHECK(!callback); }
  Imports::unregistered(owner); CHECK(weak.expired() && pool.stats().outstanding == 0);
}
struct RecycledAllocation {
  static std::set<uintptr_t> live;
  uintptr_t handle = 1;
  int value;
  explicit RecycledAllocation(int value) : value(value) {
    while (live.count(handle)) ++handle;
    live.insert(handle);
  }
  ~RecycledAllocation() { live.erase(handle); }
};
std::set<uintptr_t> RecycledAllocation::live;
struct RecycledFrame {
  std::shared_ptr<ImmutableFrames::Reservation> credit;
  std::shared_ptr<RecycledAllocation> pixels;
};
std::shared_ptr<RecycledFrame> recycled_frame(ImmutableFrames& pool, int value) {
  auto frame = std::make_shared<RecycledFrame>();
  frame->credit = pool.reserve(128 * 1024, PublicationMode::ImmutableOwnershipTransfer); CHECK(frame->credit);
  frame->pixels = std::make_shared<RecycledAllocation>(value);
  CHECK(frame->credit->begin_gpu(frame->pixels));
  CHECK(frame->credit->producer_complete() && frame->credit->publish_immutable());
  return frame;
}
struct PinnedEngine {
  // Pinned external_texture_d3d.cc:54-68 keeps the cached HANDLE when a null
  // callback destroys its EGL surface; :84 imports only a different HANDLE.
  void* cached = nullptr;
  std::shared_ptr<RecycledAllocation> surface;
  int imports = 0;
  bool populate(const Descriptor* descriptor, std::shared_ptr<RecycledAllocation> allocation = {}) {
    if (!descriptor || !descriptor->handle) { surface.reset(); return false; }
    if (descriptor->handle != cached) { surface = std::move(allocation); cached = descriptor->handle; ++imports; }
    descriptor->release_callback(descriptor->release_context);
    return bool(surface);
  }
};
void empty_callbacks_do_not_recycle_the_engines_cached_handle() {
  constexpr uint64_t owner = 109;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  PinnedEngine engine;
  std::shared_ptr<RecycledFrame> current;
  const auto opened_before = Imports::stats().opened;
  for (int index = 0; index < 128; ++index) {
    if (current) {
      current.reset();  // State::accept clears current before fresh allocation.
      for (int empty = 0; empty < 3; ++empty) {
        { auto callback = Imports::begin_callback(owner); CHECK(callback); }
        CHECK(!engine.populate(nullptr));
        CHECK(pool.stats().outstanding == 1 && pool.stats().producer_owned == 1);
        CHECK(Imports::stats().tickets == 0 && RecycledAllocation::live.size() == 1);
      }
    }
    current = recycled_frame(pool, index);
    CHECK(reinterpret_cast<void*>(current->pixels->handle) != engine.cached);
    CHECK(pool.stats().outstanding <= 2 && pool.stats().bytes <= 2 * 128 * 1024);
    const Descriptor* descriptor;
    {
      auto callback = Imports::begin_callback(owner); CHECK(callback);
      auto value = description(index); value.handle = reinterpret_cast<void*>(current->pixels->handle);
      descriptor = Imports::offer(owner, value, current, current->credit); CHECK(descriptor);
    }
    CHECK(engine.populate(descriptor, current->pixels));
    CHECK(engine.surface->value == index && engine.imports == index + 1);
    CHECK(pool.stats().outstanding == 1 && Imports::stats().tickets == 1);
  }
  CHECK(Imports::stats().opened - opened_before == 128 && pool.stats().handed_off == 127);
  current.reset();
  Imports::unregistered(owner); engine.populate(nullptr);
  CHECK(pool.stats().outstanding == 0 && pool.stats().handed_off == 128);
  CHECK(RecycledAllocation::live.empty() && Imports::stats().tickets == 0);
}
void invalid_replacements_do_not_release_the_last_exported_frame() {
  constexpr uint64_t owner = 110;
  Imports::register_owner(owner);
  ImmutableFrames pool;
  auto frame = publish(pool, 7); std::weak_ptr<Frame> weak = frame;
  const Descriptor* descriptor;
  {
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    descriptor = offer(owner, frame, 7); CHECK(descriptor);
  }
  descriptor->release_callback(descriptor->release_context); frame.reset();
  {
    auto callback = Imports::begin_callback(owner); CHECK(callback);
    auto replacement = publish(pool, 9);
    auto invalid = description(9); invalid.handle = nullptr;
    CHECK(!Imports::offer(owner, invalid, replacement, replacement->credit));
    CHECK(!offer(owner, replacement, 7));  // Different allocation aliases cached HANDLE.
    CHECK(!weak.expired() && Imports::stats().tickets == 0);
  }
  CHECK(pool.stats().outstanding == 1 && !weak.expired() && pool.stats().handed_off == 0);
  Imports::unregistered(owner);
  CHECK(weak.expired() && pool.stats().outstanding == 0 && pool.stats().handed_off == 1);
}
}  // namespace
int main() {
  try {
    engine_reads_dimensions_after_opening_callback(); continuous_128_fresh_frames_remain_producer_bounded();
    four_unopened_frames_apply_backpressure_until_real_opening(); producer_uncertainty_cannot_be_opened_or_refunded();
    published_but_never_offered_is_safe_to_discard(); legacy_and_transfer_share_one_admission_limit();
    descriptor_registry_is_bounded_independently_of_frame_count(); close_races_with_opening_but_not_descriptor_cpu_retirement();
    thread_change_and_reentry_do_not_prove_cpu_quiescence(); premature_unregistration_stays_pinned();
    empty_callbacks_do_not_recycle_the_engines_cached_handle(); invalid_replacements_do_not_release_the_last_exported_frame();
    CHECK(Imports::stats().tickets == 0);
    std::cout << "Windows immutable ownership transfer: 12 cases, " << checks << " checks passed\n";
    return 0;
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
