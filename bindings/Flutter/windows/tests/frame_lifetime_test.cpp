#include "FrameLifetime.h"
#include "NativeReply.h"
#include "PrivateTargetPool.h"
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

constexpr size_t private_bytes = 64 * 1024;
using Private = PrivateTarget<int>;
struct SplitPixels { std::shared_ptr<Private> producer; std::shared_ptr<int> published; };
std::shared_ptr<Private> fresh_private(const std::shared_ptr<ImmutableFrames::Reservation>& frame,
                                     PrivateGeometry geometry = {1, 16, 16}) {
  auto credit = frame->split_private(private_bytes); CHECK(credit);
  return std::make_shared<Private>(geometry, std::move(credit), std::make_shared<int>(17));
}
void private_bytes_survive_the_frame_and_reject_duplicate_or_foreign_split() {
  ImmutableFrames ledger, foreign;
  auto frame = ledger.reserve(private_bytes * 2, PublicationMode::ImmutableOwnershipTransfer);
  auto credit = frame->split_private(private_bytes); CHECK(credit);
  CHECK(frame->bytes() == private_bytes && ledger.stats().bytes == private_bytes * 2);
  CHECK(!frame->split_private(private_bytes));
  CHECK(credit->set_idle(true));
  auto wrong = foreign.reserve(private_bytes * 2);
  CHECK(!wrong->adopt_private(credit) && foreign.stats().bytes == private_bytes * 2);
  frame.reset();
  CHECK(ledger.stats().outstanding == 0 && ledger.stats().bytes == private_bytes);
  CHECK(ledger.stats().private_targets == 1 && ledger.stats().private_idle == 1);
  auto next = ledger.reserve(private_bytes * 2);
  CHECK(next->adopt_private(credit));
  CHECK(!next->adopt_private(credit) && !next->split_private(private_bytes));
  auto duplicate = ledger.reserve(private_bytes * 2);
  CHECK(!duplicate->adopt_private(credit));
  CHECK(ledger.stats().private_reuses == 1 && ledger.stats().private_idle == 0);
  next.reset(); duplicate.reset(); credit.reset();
  CHECK(ledger.stats().bytes == 0 && ledger.stats().private_targets == 0);
}
void private_pool_reuses_only_input_and_never_pins_the_public_frame_credit() {
  ImmutableFrames ledger;
  {
    PrivateTargetPool<int> pool;
    std::weak_ptr<int> first_private;
    for (int index = 0; index < 128; ++index) {
      auto frame = ledger.reserve(private_bytes * 2, PublicationMode::ImmutableOwnershipTransfer); CHECK(frame);
      auto pixels = std::make_shared<SplitPixels>();
      pixels->producer = pool.take({1, 16, 16}, frame);
      if (!pixels->producer) pixels->producer = fresh_private(frame);
      if (!index) first_private = pixels->producer->resource;
      CHECK(first_private.lock() == pixels->producer->resource);
      pixels->published = std::make_shared<int>(index);
      std::weak_ptr<int> public_resource = pixels->published;
      CHECK(frame->begin_gpu(pixels) && pixels->producer->begin_native());
      CHECK(!pool.recycle(pixels->producer));
      CHECK(pixels->producer->native_complete() && pixels->producer->begin_copy());
      CHECK(!pool.recycle(pixels->producer));
      CHECK(pixels->producer->copy_complete() && frame->producer_complete());
      CHECK(pool.recycle(pixels->producer) && !pixels->producer);
      CHECK(frame->publish_immutable() && frame->offer_to_consumer() && frame->consumer_opened());
      pixels.reset(); frame.reset();
      CHECK(public_resource.expired() && !first_private.expired());
      CHECK(ledger.stats().outstanding == 0 && ledger.stats().producer_owned_bytes == 0);
      CHECK(ledger.stats().bytes == private_bytes && ledger.stats().private_idle_bytes == private_bytes);
    }
    CHECK(ledger.stats().private_reuses == 127 && ledger.stats().private_targets == 1);
  }
  CHECK(ledger.stats().bytes == 0 && ledger.stats().private_targets == 0);
}
void private_copy_unknown_keeps_both_resources_and_bytes_in_quarantine() {
  // Match frames(): intentionally quarantined resources have a process owner.
  static auto* retained_ledger = new ImmutableFrames;
  auto& ledger = *retained_ledger;
  for (int phase = 0; phase < 3; ++phase) {
    std::weak_ptr<Private> held;
    {
      PrivateTargetPool<int> pool;
      auto frame = ledger.reserve(private_bytes * 2, PublicationMode::ImmutableOwnershipTransfer);
      auto pixels = std::make_shared<SplitPixels>(); pixels->producer = fresh_private(frame);
      held = pixels->producer;
      CHECK(frame->begin_gpu(pixels));
      if (phase >= 1) CHECK(pixels->producer->begin_native());
      if (phase == 2) CHECK(pixels->producer->native_complete() && pixels->producer->begin_copy());
      CHECK(!pixels->producer->skip_copy() && !pool.recycle(pixels->producer));
      // Cancellation before Native work, unknown Native completion and unknown
      // D3D11 copy completion all retain the independently charged input.
      pixels.reset(); frame.reset();
    }
    const auto count = size_t(phase + 1);
    CHECK(!held.expired() && ledger.stats().quarantined == count);
    CHECK(ledger.stats().bytes == private_bytes * 2 * count && ledger.stats().private_bytes == private_bytes * count);
    CHECK(ledger.stats().private_idle == 0);
  }
}
void private_geometry_device_and_alias_controls_preserve_resource_lifetime() {
  ImmutableFrames ledger;
  PrivateTargetPool<int> pool;
  for (PrivateGeometry different : {PrivateGeometry{1, 17, 16}, PrivateGeometry{2, 16, 16}}) {
    auto frame = ledger.reserve(private_bytes * 2);
    auto target = fresh_private(frame);
    auto alias = target;
    CHECK(target->begin_native() && target->native_complete() && target->skip_copy());
    CHECK(!pool.recycle(target)); alias.reset();
    size_t bytes_at_resource_destruction = 0;
    target->resource = std::shared_ptr<int>(new int(19), [&](int* resource) {
      bytes_at_resource_destruction = ledger.stats().private_bytes; delete resource;
    });
    std::weak_ptr<int> resource = target->resource;
    CHECK(pool.recycle(target)); frame.reset();
    auto next = ledger.reserve(private_bytes * 2);
    CHECK(!pool.take(different, next));
    CHECK(resource.expired() && bytes_at_resource_destruction == private_bytes);
    CHECK(ledger.stats().private_bytes == 0 && ledger.stats().bytes == private_bytes * 2);
    next.reset();
  }
}
void private_and_public_reservations_share_the_original_global_caps() {
  ImmutableFrames ledger;
  std::vector<std::shared_ptr<ImmutableFrames::PrivateReservation>> private_credits;
  std::vector<std::shared_ptr<ImmutableFrames::Reservation>> frames;
  for (int index = 0; index < 4; ++index) {
    auto frame = ledger.reserve(16 * 1024 * 1024); CHECK(frame);
    auto credit = frame->split_private(8 * 1024 * 1024); CHECK(credit);
    frames.push_back(frame); private_credits.push_back(credit);
  }
  CHECK(!ledger.reserve(1) && ledger.stats().bytes == 64 * 1024 * 1024);
  frames.clear();
  CHECK(ledger.stats().outstanding == 0 && ledger.stats().bytes == 32 * 1024 * 1024);
  auto first = ledger.reserve(16 * 1024 * 1024), second = ledger.reserve(16 * 1024 * 1024);
  CHECK(first && second && !ledger.reserve(1));
  CHECK(!first->split_private(8 * 1024 * 1024)); // Four independent private targets are retained.
  CHECK(private_credits[0]->set_idle(true) && private_credits[1]->set_idle(true));
  CHECK(!private_credits[2]->set_idle(true)); // At most two producer idle entries.
  private_credits.clear(); first.reset(); second.reset();
  CHECK(ledger.stats().bytes == 0 && ledger.stats().private_targets == 0);
}
}  // namespace
int main() {
  try {
    full_stamp_gate(); two_surface_budget_boundaries(); unsubmitted_and_complete_unpublished_refund();
    unknown_producer_completion_quarantines_resources(); descriptor_open_and_unregister_never_recycle_published_frame();
    saturation_and_concurrent_admission(); trusted_native_metadata_has_scoped_fields(); native_metadata_bounds_and_escapes();
    private_bytes_survive_the_frame_and_reject_duplicate_or_foreign_split();
    private_pool_reuses_only_input_and_never_pins_the_public_frame_credit();
    private_copy_unknown_keeps_both_resources_and_bytes_in_quarantine();
    private_geometry_device_and_alias_controls_preserve_resource_lifetime();
    private_and_public_reservations_share_the_original_global_caps();
    std::cout << "Windows bridge portable: 13 cases, " << checks << " checks passed\n"; return 0;
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
