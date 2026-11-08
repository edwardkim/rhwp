#include "CloseRecordRegistry.h"
#include <iostream>
#include <memory>
#include <stdexcept>

using namespace rhwp::d3d;
namespace {
int checks = 0;
#define CHECK(value) do { ++checks; if (!(value)) throw std::runtime_error("check failed: " #value); } while (0)
struct Record {
  uint64_t generation;
  CloseCompletion completion;
  bool negotiated = true;
  std::shared_ptr<int> resource;
};
struct Receipt { uint64_t generation; bool acknowledged; };
using Registry = CloseRecordRegistry<Record, Receipt>;
Record finished(uint64_t generation, bool negotiated = true) {
  return {generation, {true, true, true, false}, negotiated, std::make_shared<int>(1)};
}
bool ready(const Record& record) { return record.completion.ready(); }
Receipt receipt(const Record& record) { return {record.generation, record.negotiated}; }
bool acknowledge(Registry& registry, int64_t id, uint64_t generation) {
  const auto* record = registry.find(id);
  if (!record) {
    const auto* cached = registry.completed(id);
    return cached && cached->acknowledged && close_generation_matches(cached->generation, generation, true, true);
  }
  if (!close_generation_matches(record->generation, generation, true, record->negotiated)) return false;
  return registry.retire(id, ready, receipt);
}
void old_create_prune_loses_the_first_receipt() {
  // Exact pre-fix create->prune routing policy: every completed record is
  // removed and only the last shared State remains addressable by status.
  std::map<int64_t, Record> old;
  old.emplace(1, finished(11)); old.emplace(2, finished(12));
  int64_t last_closed = 0;
  for (auto it = old.begin(); it != old.end();) {
    if (ready(it->second)) { last_closed = it->first; it = old.erase(it); }
    else ++it;
  }
  old.emplace(3, finished(13));  // A new create runs prune before allocation.
  CHECK(old.count(1) == 0 && last_closed == 2); // status(1) can no longer confirm completion.
}
void two_unobserved_closes_survive_new_owner_attempts_and_400_other_acks() {
  Registry registry;
  CHECK(registry.insert(1, finished(11)) && registry.insert(2, finished(12)));
  auto weak_first = std::weak_ptr<int>(registry.find(1)->resource);
  auto weak_second = std::weak_ptr<int>(registry.find(2)->resource);
  CHECK(!registry.can_create() && !registry.insert(3, finished(13)) && !registry.insert(4, finished(14)));
  CHECK(registry.find(1) && registry.find(2) && registry.receipt_count() == 0);
  CHECK(acknowledge(registry, 1, 11));
  CHECK(weak_first.expired() && !weak_second.expired());
  CHECK(acknowledge(registry, 1, 11)); // An acked receipt has no producer resources.
  for (int64_t id = 100; id < 500; ++id) {
    CHECK(registry.can_create() && registry.insert(id, finished(uint64_t(id))));
    CHECK(!registry.can_create() && registry.find(2));
    CHECK(acknowledge(registry, id, uint64_t(id)));
    CHECK(registry.size() == 1 && registry.receipt_count() <= 2);
  }
  CHECK(!registry.completed(1)); // Only acknowledged metadata can expire.
  CHECK(registry.find(2) && !weak_second.expired());
  CHECK(acknowledge(registry, 2, 12) && weak_second.expired());
  CHECK(registry.size() == 0 && registry.receipt_count() == 2);
}
void incomplete_or_quiescence_only_records_are_not_acknowledged() {
  for (int bits = 0; bits < 16; ++bits) {
    Registry registry;
    auto record = finished(1);
    record.completion = {bool(bits & 1), bool(bits & 2), bool(bits & 4), bool(bits & 8)};
    CHECK(registry.insert(1, std::move(record)));
    const bool expected = bits == 7;
    CHECK(acknowledge(registry, 1, 1) == expected);
    CHECK(registry.size() == (expected ? 0u : 1u));
  }
}
void reused_texture_id_requires_the_current_generation() {
  Registry registry;
  CHECK(registry.insert(9, finished(11)));
  CHECK(!registry.insert(9, finished(12)) && registry.find(9)->generation == 11);
  CHECK(!acknowledge(registry, 9, 12) && registry.find(9));
  CHECK(acknowledge(registry, 9, 11));
  CHECK(registry.insert(9, finished(12)) && !registry.completed(9));
  CHECK(!acknowledge(registry, 9, 11) && registry.find(9)->generation == 12);
  CHECK(acknowledge(registry, 9, 12));
  CHECK(!acknowledge(registry, 9, 11) && acknowledge(registry, 9, 12));
  CHECK(!close_generation_matches(12, 0, false, true));
  CHECK(close_generation_matches(12, 0, false, false)); // Old non-negotiated clients.
  CHECK(!close_generation_matches(12, 11, true, false));
}
void metadata_allocation_failure_preserves_live_records_and_credit() {
  Registry registry;
  CHECK(registry.insert(1, finished(1)) && registry.insert(2, finished(2)));
  auto weak = std::weak_ptr<int>(registry.find(1)->resource);
  bool failed = false;
  try {
    registry.retire(1, ready, [](const Record&) -> Receipt { throw std::bad_alloc(); });
  } catch (const std::bad_alloc&) { failed = true; }
  CHECK(failed && registry.size() == 2 && !registry.can_create());
  CHECK(!weak.expired() && registry.receipt_count() == 0 && registry.find(1));
  CHECK(acknowledge(registry, 1, 1) && weak.expired());
  CHECK(registry.insert(3, finished(3)));
  registry.erase_failed_creation(3);
  CHECK(registry.can_create() && registry.size() == 1);
}
void only_legacy_records_allow_automatic_retirement() {
  Registry registry;
  CHECK(!registry.insert(0, finished(1)));
  CHECK(registry.insert(1, finished(1, false)) && registry.insert(2, finished(2)));
  for (auto it = registry.records().begin(); it != registry.records().end();) {
    const auto id = (it++)->first;
    if (!registry.find(id)->negotiated) CHECK(registry.retire(id, ready, receipt));
  }
  CHECK(!registry.find(1) && registry.find(2) && registry.can_create());
  CHECK(registry.completed(1) && !registry.completed(1)->acknowledged);
  CHECK(registry.insert(3, finished(3)));
  CHECK(!registry.can_create() && !registry.insert(4, finished(4)));
}
}  // namespace
int main() {
  try {
    old_create_prune_loses_the_first_receipt();
    two_unobserved_closes_survive_new_owner_attempts_and_400_other_acks();
    incomplete_or_quiescence_only_records_are_not_acknowledged();
    reused_texture_id_requires_the_current_generation();
    metadata_allocation_failure_preserves_live_records_and_credit();
    only_legacy_records_allow_automatic_retirement();
    std::cout << "Windows close registry: 6 cases, " << checks << " checks passed\n";
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
