#include "texture_state.h"
#include <algorithm>
#include <chrono>
#include <condition_variable>
#include <functional>
#include <future>
#include <iostream>
#include <map>
#include <limits>
#include <set>

using namespace rhwp_flutter;
namespace {
int checks = 0;
#define CHECK(condition) do { ++checks; if (!(condition)) throw std::runtime_error("check failed: " #condition); } while (0)

template <class Function> void rejected(const char* code, Function function) {
  try { function(); } catch (const Failure& error) { CHECK(error.code == code); return; }
  throw std::runtime_error("expected Failure");
}

struct Device {
  uint64_t current = 1;
  uint32_t next = 10;
  int deleted = 0, current_callbacks = 0;
  bool reclaim_on_abandon = true;  // Fake target names have no driver allocation.
};
class FakeContext : public Context {
 public:
  std::shared_ptr<Device> device;
  uint64_t token;
  std::thread::id thread = std::this_thread::get_id();
  FakeContext(std::shared_ptr<Device> device, uint64_t token) : device(std::move(device)), token(token) {}
  bool same_current() const noexcept override { ++device->current_callbacks; return device->current == token && thread == std::this_thread::get_id(); }
  bool same_context(const Context& other) const noexcept override {
    auto* context = dynamic_cast<const FakeContext*>(&other);
    return context && context->token == token && context->thread == thread;
  }
  const void* procedure(const char*) noexcept override { return reinterpret_cast<void*>(1); }
  Target allocate(uint32_t width, uint32_t height) override { return {device->next++, device->next++, width, height}; }
  void destroy(Target& target) noexcept override { if (target.texture) ++device->deleted; target = {}; }
  bool abandon_reclaims_resources() const noexcept override { return device->reclaim_on_abandon; }
};
class FakeBridge : public Bridge {
 public:
  std::mutex mutex;
  struct Owner { std::thread::id thread; Callbacks callbacks; uint64_t identity; };
  std::map<uint64_t, Owner> owners;
  std::map<uint64_t, int> releases;
  SnapshotInfo info{7, 3, 2, true, true};
  uint64_t next = 1, readbacks = 0;
  int renders = 0, released = 0, abandoned = 0;
  std::string render_error;
  bool cleanup_error = false;
  std::function<void()> during_render;
  SnapshotInfo inspect(uint64_t) override { return info; }
  void release_snapshot(uint64_t id) noexcept override { std::lock_guard<std::mutex> lock(mutex); ++releases[id]; }
  uint64_t create(uint64_t identity, uint64_t, Callbacks callbacks) override {
    CHECK(callbacks.current(callbacks.user, identity) == 1);
    CHECK(callbacks.load(callbacks.user, "glDummy") != nullptr);
    auto id = next++;
    owners.emplace(id, Owner{std::this_thread::get_id(), callbacks, identity});
    return id;
  }
  uint64_t render(uint64_t presenter, uint64_t, const Description&, const Target&) override {
    auto& owner = owners.at(presenter);
    CHECK(owner.thread == std::this_thread::get_id());
    CHECK(owner.callbacks.current(owner.callbacks.user, owner.identity) == 1);
    ++renders;
    if (during_render) { auto hook = std::move(during_render); hook(); }
    if (!render_error.empty()) throw Failure(render_error, "synthetic replay failure");
    return readbacks;
  }
  void release(uint64_t presenter, uint64_t identity, uint64_t) override {
    if (cleanup_error) throw Failure("cleanup", "synthetic cleanup failure");
    auto& owner = owners.at(presenter);
    CHECK(owner.thread == std::this_thread::get_id());
    CHECK(owner.callbacks.current(owner.callbacks.user, identity) == 1);
    ++released; owners.erase(presenter);
  }
  void abandon(uint64_t presenter) override {
    if (cleanup_error) throw Failure("cleanup", "synthetic cleanup failure");
    CHECK(owners.at(presenter).thread == std::this_thread::get_id());
    ++abandoned; owners.erase(presenter);  // Must never call current or GL.
  }
};

Description frame(uint64_t view = 1, uint64_t snapshot = 19) {
  Description result;
  result.stamp = {7, 3, 2, view, 1, 1.5};
  result.width = 800; result.height = 600;
  result.pages = {{snapshot, 1.875, 8, -12, 0xffffff}};
  return result;
}

void ownership_and_validation() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  auto description = frame(); description.width = 4097;
  rejected("invalidArgument", [&] { state->submit(description); });
  CHECK(bridge->releases.empty());
  description = frame(); description.pages.resize(kMaxPages + 1, description.pages[0]);
  rejected("invalidArgument", [&] { state->submit(description); });
  description = frame(); description.pages[0].scale = std::numeric_limits<double>::quiet_NaN();
  rejected("invalidArgument", [&] { state->submit(description); });
  bridge->info.current = false;
  rejected("staleFrame", [&] { state->submit(frame()); });
  bridge->info.current = true;
  CHECK(state->submit(frame()) == 1);
  CHECK(bridge->releases.empty());
  rejected("staleFrame", [&] { state->submit(frame()); });
  description = frame(2); description.pages.push_back(description.pages[0]);
  CHECK(state->submit(description) == 2);  // Two placements, only one owner.
  state->close(); state->close();
  CHECK(bridge->releases[19] == 1);
  CHECK(state->status().cleanup_complete);
  rejected("closing", [&] { state->submit(frame(3, 20)); });
  CHECK(bridge->releases[20] == 0);
}

void strict_failure_preserves_front_and_releases_on_own_populate() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  auto device = std::make_shared<Device>();
  auto context = std::make_shared<FakeContext>(device, 1);
  RasterRegistry registry;
  state->submit(frame());
  auto first = registry.populate(state, context);
  CHECK(first.texture && state->status().rendered == 1);
  CHECK(registry.populate(state, context).texture == first.texture && bridge->renders == 1);
  bridge->render_error = "documentError";
  state->submit(frame(2, 20));
  CHECK(bridge->releases[19] == 1);
  CHECK(registry.populate(state, context).texture == first.texture);
  CHECK(state->status().rendered == 1 && state->status().code == "documentError");
  state->close();
  CHECK(!state->status().cleanup_complete && RasterRegistry::pending_retire() == 1);
  rejected("closing", [&] { registry.populate(state, context); });
  CHECK(state->status().cleanup_complete && bridge->released == 1);
  CHECK(device->deleted == 2 && RasterRegistry::live_slots() == 0);
  CHECK(bridge->releases[20] == 1);
}

void changed_inflight_frame_is_never_committed() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  auto context = std::make_shared<FakeContext>(std::make_shared<Device>(), 1);
  RasterRegistry registry;
  state->submit(frame());
  auto first = registry.populate(state, context);
  state->submit(frame(2, 20));
  bridge->during_render = [&] { state->submit(frame(3, 21)); };
  CHECK(registry.populate(state, context).texture == first.texture);
  CHECK(state->status().rendered == 1 && state->status().accepted == 3);
  CHECK(bridge->releases[20] == 1);
  CHECK(registry.populate(state, context).texture != first.texture);
  CHECK(state->status().rendered == 3);
  state->close(); rejected("closing", [&] { registry.populate(state, context); });
}

void readback_contract_failure_is_not_a_gpu_success() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  auto context = std::make_shared<FakeContext>(std::make_shared<Device>(), 1);
  RasterRegistry registry;
  state->submit(frame()); bridge->readbacks = 1;
  rejected("contract", [&] { registry.populate(state, context); });
  CHECK(state->status().frames == 0);
  CHECK(state->status().whole_frame_readbacks == 1);
  state->close(); rejected("closing", [&] { registry.populate(state, context); });
}

void context_switch_requires_new_generation_and_abandons_without_callback() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  auto device = std::make_shared<Device>();
  auto context = std::make_shared<FakeContext>(device, 1);
  RasterRegistry registry;
  state->submit(frame()); registry.populate(state, context);
  device->current = 2;
  auto replacement = std::make_shared<FakeContext>(device, 2);
  rejected("contextChanged", [&] { registry.populate(state, replacement); });
  CHECK(bridge->abandoned == 1 && bridge->released == 0 && device->deleted == 0);
  CHECK(state->status().device == 2 && state->status().abandoned_targets == 1);
  rejected("contextChanged", [&] { state->submit(frame(2)); });
  auto description = frame(2); description.stamp.device = 2;
  state->submit(description); registry.populate(state, replacement);
  state->close(); rejected("closing", [&] { registry.populate(state, replacement); });
  CHECK(bridge->releases[19] == 1);
}

void close_does_not_wait_or_call_foreign_thread_native_handles() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  state->submit(frame());
  std::promise<void> entered, resume;
  auto waiting = resume.get_future();
  bridge->during_render = [&] { entered.set_value(); waiting.wait(); };
  auto raster = std::async(std::launch::async, [&] {
    auto context = std::make_shared<FakeContext>(std::make_shared<Device>(), 1);
    RasterRegistry registry;
    rejected("notReady", [&] { registry.populate(state, context); });
    rejected("closing", [&] { registry.populate(state, context); });
  });
  entered.get_future().wait();
  auto start = std::chrono::steady_clock::now();
  state->close();
  CHECK(std::chrono::steady_clock::now() - start < std::chrono::milliseconds(100));
  CHECK(bridge->released == 0 && bridge->abandoned == 0);
  CHECK(!state->status().cleanup_complete && bridge->releases[19] == 0);
  resume.set_value(); raster.get();
  CHECK(state->status().cleanup_complete && bridge->releases[19] == 1);
  CHECK(bridge->released == 1 && state->status().frames == 0);
}

void no_other_populate_is_a_fence_and_retire_slots_are_bounded() {
  auto bridge = std::make_shared<FakeBridge>();
  auto device = std::make_shared<Device>();
  auto context = std::make_shared<FakeContext>(device, 1);
  std::vector<std::shared_ptr<Shared>> states;
  {
    RasterRegistry registry;
    for (size_t index = 0; index < kMaxRasterSlots; ++index) {
      auto state = std::make_shared<Shared>(bridge);
      state->submit(frame(1, 19 + index)); registry.populate(state, context);
      state->close(); states.push_back(state);
    }
    CHECK(RasterRegistry::pending_retire() == kMaxRasterSlots);
    auto extra = std::make_shared<Shared>(bridge); extra->submit(frame(1, 24));
    rejected("retireLimit", [&] { registry.populate(extra, context); });
    CHECK(bridge->released == 0 && device->deleted == 0);
    rejected("closing", [&] { registry.populate(states[0], context); });
    CHECK(bridge->released == 1 && RasterRegistry::pending_retire() == 3);
    registry.populate(extra, context);
    CHECK(RasterRegistry::live_slots() == kMaxRasterSlots);
    extra->close();
    const auto callbacks = device->current_callbacks;
    device->current = 0;
    // Remaining owners survive until this same OS thread's registry teardown.
    CHECK(!states[1]->status().cleanup_complete);
    CHECK(callbacks == device->current_callbacks);
  }
  CHECK(RasterRegistry::live_slots() == 0 && bridge->abandoned == 4);
  CHECK(states[1]->status().cleanup_complete);
  CHECK(states[1]->status().abandoned_targets == 1);
  CHECK(bridge->owners.empty());
}

void registry_must_not_migrate_to_another_os_thread() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  RasterRegistry registry;
  auto other = std::async(std::launch::async, [&] {
    rejected("wrongThread", [&] { registry.drain_retired(state); });
  });
  other.get(); CHECK(bridge->owners.empty());
}

void cleanup_failure_keeps_userdata_and_credit_until_owner_retry() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  auto context = std::make_shared<FakeContext>(std::make_shared<Device>(), 1);
  RasterRegistry registry;
  state->submit(frame()); registry.populate(state, context);
  state->close(); bridge->cleanup_error = true;
  rejected("closing", [&] { registry.populate(state, context); });
  CHECK(!state->status().cleanup_complete && RasterRegistry::pending_retire() == 1);
  CHECK(!bridge->owners.empty());
  auto& owner = bridge->owners.begin()->second;
  CHECK(owner.callbacks.current(owner.callbacks.user, owner.identity) == 1);
  bridge->cleanup_error = false;
  rejected("closing", [&] { registry.populate(state, context); });
  CHECK(state->status().cleanup_complete && RasterRegistry::live_slots() == 0);
}

void a_live_texture_cannot_gain_a_second_raster_thread_owner() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  auto context = std::make_shared<FakeContext>(std::make_shared<Device>(), 1);
  RasterRegistry registry;
  state->submit(frame()); registry.populate(state, context);
  auto other = std::async(std::launch::async, [&] {
    RasterRegistry foreign;
    auto context = std::make_shared<FakeContext>(std::make_shared<Device>(), 1);
    rejected("wrongThread", [&] { foreign.populate(state, context); });
  });
  other.get();
  CHECK(state->status().raster_owner && RasterRegistry::live_slots() == 1);
  CHECK(registry.populate(state, context).texture != 0);
  state->close(); rejected("closing", [&] { registry.populate(state, context); });
}

void owner_slot_pins_bridge_and_callback_userdata_after_ui_drops_references() {
  std::weak_ptr<FakeBridge> weak_bridge;
  std::weak_ptr<Shared> weak_state;
  {
    RasterRegistry registry;
    auto bridge = std::make_shared<FakeBridge>();
    auto state = std::make_shared<Shared>(bridge);
    auto context = std::make_shared<FakeContext>(std::make_shared<Device>(), 1);
    state->submit(frame()); registry.populate(state, context); state->close();
    weak_bridge = bridge; weak_state = state;
    bridge.reset(); state.reset(); context.reset();
    CHECK(!weak_bridge.expired() && !weak_state.expired());
    CHECK(RasterRegistry::pending_retire() == 1);
    auto retained = weak_bridge.lock();
    auto& owner = retained->owners.begin()->second;
    CHECK(owner.callbacks.current(owner.callbacks.user, owner.identity) == 1);
  }
  CHECK(weak_bridge.expired() && weak_state.expired());
  CHECK(RasterRegistry::live_slots() == 0);
}

void in_flight_snapshot_credit_rejection_preserves_caller_ownership() {
  auto bridge = std::make_shared<FakeBridge>();
  auto state = std::make_shared<Shared>(bridge);
  auto context = std::make_shared<FakeContext>(std::make_shared<Device>(), 1);
  RasterRegistry registry;
  auto pages = [](uint64_t view, uint64_t first) {
    auto result = frame(view, first);
    result.pages.clear();
    for (size_t page = 0; page < kMaxPages; ++page) result.pages.push_back({first + page, 1, 0, 0, 0xffffff});
    return result;
  };
  state->submit(pages(1, 100));
  bridge->during_render = [&] {
    state->submit(pages(2, 200));
    rejected("limitExceeded", [&] { state->submit(pages(3, 300)); });
    for (size_t page = 0; page < kMaxPages; ++page) CHECK(bridge->releases[300 + page] == 0);
  };
  rejected("notReady", [&] { registry.populate(state, context); });
  CHECK(state->status().accepted == 2 && state->status().rendered == 0);
  state->submit(pages(3, 300));
  state->close(); rejected("closing", [&] { registry.populate(state, context); });
  for (size_t page = 0; page < kMaxPages; ++page) {
    CHECK(bridge->releases[100 + page] == 1);
    CHECK(bridge->releases[200 + page] == 1);
    CHECK(bridge->releases[300 + page] == 1);
  }
}

void unproven_driver_orphans_permanently_consume_the_process_budget() {
  auto bridge = std::make_shared<FakeBridge>();
  auto device = std::make_shared<Device>(); device->reclaim_on_abandon = false;
  auto context = std::make_shared<FakeContext>(device, 1);
  for (size_t index = 0; index < kMaxRasterSlots; ++index) {
    RasterRegistry registry;
    auto state = std::make_shared<Shared>(bridge);
    state->submit(frame(1, 100 + index)); registry.populate(state, context); state->close();
    // Simulated owner TLS teardown abandons, rather than claiming a GL delete.
  }
  CHECK(RasterRegistry::live_slots() == kMaxRasterSlots);
  CHECK(RasterRegistry::orphaned_slots() == kMaxRasterSlots);
  CHECK(RasterRegistry::pending_retire() == kMaxRasterSlots);
  RasterRegistry registry;
  auto state = std::make_shared<Shared>(bridge); state->submit(frame());
  rejected("retireLimit", [&] { registry.populate(state, context); });
  CHECK(bridge->owners.empty() && device->deleted == 0);
}
}  // namespace

int main() {
  try {
    ownership_and_validation();
    strict_failure_preserves_front_and_releases_on_own_populate();
    changed_inflight_frame_is_never_committed();
    readback_contract_failure_is_not_a_gpu_success();
    context_switch_requires_new_generation_and_abandons_without_callback();
    close_does_not_wait_or_call_foreign_thread_native_handles();
    no_other_populate_is_a_fence_and_retire_slots_are_bounded();
    registry_must_not_migrate_to_another_os_thread();
    cleanup_failure_keeps_userdata_and_credit_until_owner_retry();
    a_live_texture_cannot_gain_a_second_raster_thread_owner();
    owner_slot_pins_bridge_and_callback_userdata_after_ui_drops_references();
    in_flight_snapshot_credit_rejection_preserves_caller_ownership();
    unproven_driver_orphans_permanently_consume_the_process_budget();
    std::cout << "texture state: 13 cases, " << checks << " checks passed\n";
    return 0;
  } catch (const std::exception& error) {
    std::cerr << error.what() << '\n'; return 1;
  }
}
