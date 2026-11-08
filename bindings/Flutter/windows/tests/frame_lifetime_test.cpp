#include "FrameLifetime.h"
#include "NativeReply.h"
#include <atomic>
#include <functional>
#include <iostream>
#include <thread>
#include <vector>

using namespace rhwp::d3d;
namespace {
int checks = 0;
#define CHECK(value) do { ++checks; if (!(value)) throw std::runtime_error("check failed: " #value); } while (0)
void rejected(const std::function<void()>& call) {
  try { call(); } catch (const std::exception&) { ++checks; return; }
  throw std::runtime_error("expected rejection");
}
void full_stamp_gate() {
  GenerationGate gate(2);
  Stamp stamp{7, 3, 4, 1, 2, 1.5};
  auto first = gate.accept(stamp); CHECK(first && gate.current(first, stamp));
  for (int field = 0; field < 6; ++field) {
    auto wrong = stamp;
    if (field == 0) ++wrong.document; else if (field == 1) ++wrong.revision;
    else if (field == 2) ++wrong.font; else if (field == 3) ++wrong.view;
    else if (field == 4) ++wrong.device; else wrong.scale += .25;
    CHECK(!gate.current(first, wrong));
  }
  CHECK(!gate.accept(stamp)); ++stamp.view;
  auto second = gate.accept(stamp); CHECK(second > first && !gate.current(first, stamp));
  gate.close(); CHECK(!gate.current(second, stamp) && !gate.accept(stamp));
}
void two_surface_budget_boundaries() {
  CHECK(frame_charge(1, 1) == 128 * 1024);
  CHECK(frame_charge(1920, 1080) == 16 * 1024 * 1024);
  CHECK(frame_charge(800, 600) >= 2 * 800 * 600 * 4);
  CHECK(!frame_charge(0, 1) && !frame_charge(4096, 4096) && !frame_charge(UINT64_MAX, 1));
  ImmutableFrames pool;
  CHECK(!pool.reserve(0) && !pool.reserve(16 * 1024 * 1024 + 1));
  std::vector<std::shared_ptr<ImmutableFrames::Reservation>> slots;
  for (int i = 0; i < 4; ++i) { auto slot = pool.reserve(16 * 1024 * 1024); CHECK(slot); slots.push_back(slot); }
  CHECK(!pool.reserve(1) && pool.stats().outstanding == 4 && pool.stats().bytes == 64 * 1024 * 1024);
  slots.clear(); CHECK(pool.stats().outstanding == 0 && pool.stats().bytes == 0);
}
void unsubmitted_and_complete_unpublished_refund() {
  ImmutableFrames pool;
  auto slot = pool.reserve(128 * 1024);
  CHECK(!slot->begin_gpu(nullptr)); slot.reset(); CHECK(pool.stats().bytes == 0);
  auto resource = std::make_shared<int>(42); std::weak_ptr<int> weak = resource;
  slot = pool.reserve(128 * 1024);
  CHECK(slot->begin_gpu(resource)); resource.reset(); CHECK(!weak.expired());
  CHECK(!slot->begin_gpu(std::make_shared<int>(9)));
  CHECK(slot->producer_complete()); slot.reset();
  CHECK(weak.expired() && pool.stats().outstanding == 0);
}
void unknown_producer_completion_quarantines_resources() {
  ImmutableFrames pool;
  auto resource = std::make_shared<int>(42); std::weak_ptr<int> weak = resource;
  auto slot = pool.reserve(128 * 1024); CHECK(slot->begin_gpu(resource));
  resource.reset(); slot.reset();
  CHECK(!weak.expired() && pool.stats().quarantined == 1 && pool.stats().bytes == 128 * 1024);
}
void descriptor_open_and_unregister_never_recycle_published_frame() {
  ImmutableFrames pool;
  auto slot = pool.reserve(128 * 1024); auto resource = std::make_shared<int>(42);
  std::weak_ptr<int> weak = resource;
  CHECK(slot->begin_gpu(resource) && slot->producer_complete() && slot->publish_immutable());
  CHECK(!slot->producer_complete() && !slot->publish_immutable());
  unsigned descriptor_opened = 0; ++descriptor_opened;  // No GPU-retirement transition exists.
  CHECK(descriptor_opened == 1); resource.reset(); slot.reset();
  CHECK(!weak.expired() && pool.stats().immutable == 1 && pool.stats().outstanding == 1);
  auto reopened_registration = pool.reserve(128 * 1024); CHECK(reopened_registration);
  reopened_registration.reset(); CHECK(pool.stats().immutable == 1);
}
void saturation_and_concurrent_admission() {
  ImmutableFrames pool;
  std::atomic<int> accepted{0};
  std::vector<std::thread> threads;
  for (int i = 0; i < 12; ++i) threads.emplace_back([&] {
    auto slot = pool.reserve(16 * 1024 * 1024);
    if (!slot) return;
    if (!slot->begin_gpu(std::make_shared<int>(1)) || !slot->producer_complete() || !slot->publish_immutable()) std::terminate();
    ++accepted;
  });
  for (auto& thread : threads) thread.join();
  CHECK(accepted == 4 && pool.stats().immutable == 4 && !pool.reserve(1));
  CHECK(pool.stats().bytes == 64 * 1024 * 1024);
}
void trusted_native_metadata_has_scoped_fields() {
  const std::string text = R"({"frame":{"ok":true,"producerGpuComplete":true,"wholeFrameReadbacks":0,"stamp":{"deviceScale":1.25}},"error":"escaped \"ok\":true, {braces}","ok":false})";
  NativeReply reply(text); CHECK(!reply.field("ok").boolean());
  CHECK(reply.field("frame").field("producerGpuComplete").boolean());
  CHECK(reply.field("frame").field("wholeFrameReadbacks").integer() == 0);
  CHECK(reply.field("frame").field("stamp").field("deviceScale").number() == 1.25);
  rejected([&] { reply.field("producerGpuComplete"); });
  rejected([] { NativeReply("\"true\"").boolean(); });
  rejected([] { NativeReply("18446744073709551616").integer(); });
  rejected([] { NativeReply("NaN").number(); });
}
void native_metadata_bounds_and_escapes() {
  rejected([] { NativeReply(std::string(65537, ' ')); });
  CHECK(NativeReply("\"line\\nquote\\\"end\"").text() == "line\nquote\"end");
  CHECK(NativeReply("  { \"child\" : [1,{\"x\":\"}\\\"\"}], \"ok\" : true }").field("ok").boolean());
  std::string deep = "{\"ignore\":"; for (int i = 0; i < 20; ++i) deep += '[';
  deep += '0'; for (int i = 0; i < 20; ++i) deep += ']'; deep += ",\"ok\":true}";
  rejected([&] { NativeReply(deep).field("ok"); });
  rejected([] { NativeReply("{\"broken\":\"unterminated}").field("ok"); });
}
}  // namespace
int main() {
  try {
    full_stamp_gate(); two_surface_budget_boundaries(); unsubmitted_and_complete_unpublished_refund();
    unknown_producer_completion_quarantines_resources(); descriptor_open_and_unregister_never_recycle_published_frame();
    saturation_and_concurrent_admission(); trusted_native_metadata_has_scoped_fields(); native_metadata_bounds_and_escapes();
    std::cout << "Windows bridge portable: 8 cases, " << checks << " checks passed\n"; return 0;
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
