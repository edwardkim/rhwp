#ifndef RHWP_WINDOWS_CLOSE_RECORD_REGISTRY_H_
#define RHWP_WINDOWS_CLOSE_RECORD_REGISTRY_H_

#include <array>
#include <cstddef>
#include <cstdint>
#include <map>
#include <utility>

namespace rhwp::d3d {

struct CloseCompletion {
  bool closed, owner_finished, unregistered, in_flight;
  bool ready() const { return closed && owner_finished && unregistered && !in_flight; }
};
inline bool close_generation_matches(uint64_t generation, uint64_t supplied, bool present, bool required) {
  return present ? supplied == generation : !required;
}

// Platform-thread registry: completed producer/unregister work is not a client
// receipt. Negotiated closing records consume one of the two live slots until
// explicit acknowledgement. Only acknowledged, resource-free metadata may be
// evicted from the bounded terminal cache. The caller supplies readiness and
// identity checks; this helper infers no producer or consumer GPU completion.
template <typename Record, typename Receipt>
class CloseRecordRegistry {
 public:
  static constexpr size_t kMaxRecords = 2, kMaxReceipts = 2;
  bool can_create() const { return records_.size() < kMaxRecords; }
  bool insert(int64_t id, Record record) {
    if (id <= 0 || !can_create()) return false;
    const auto inserted = records_.emplace(id, std::move(record));
    if (!inserted.second) return false;
    erase_receipt(id); // Flutter may reuse an acknowledged texture object address.
    return true;
  }
  Record* find(int64_t id) {
    const auto found = records_.find(id);
    return found == records_.end() ? nullptr : &found->second;
  }
  const Receipt* completed(int64_t id) const {
    const auto found = receipts_.find(id);
    return found == receipts_.end() ? nullptr : &found->second;
  }
  template <typename Ready, typename MakeReceipt>
  bool retire(int64_t id, Ready ready, MakeReceipt make_receipt) {
    const auto found = records_.find(id);
    if (found == records_.end() || !ready(found->second)) return false;
    // Allocate/copy all terminal storage before releasing any live record.
    Receipt value = make_receipt(found->second);
    const auto inserted = receipts_.emplace(id, std::move(value));
    if (!inserted.second) return false;
    if (receipt_count_ == kMaxReceipts) erase_receipt(order_[0]);
    order_[receipt_count_++] = id;
    records_.erase(found);
    return true;
  }
  void erase_failed_creation(int64_t id) { records_.erase(id); }
  auto& records() { return records_; }
  size_t size() const { return records_.size(); }
  size_t receipt_count() const { return receipt_count_; }
 private:
  void erase_receipt(int64_t id) {
    if (!receipts_.erase(id)) return;
    for (size_t index = 0; index < receipt_count_; ++index) {
      if (order_[index] != id) continue;
      for (size_t next = index + 1; next < receipt_count_; ++next) order_[next - 1] = order_[next];
      --receipt_count_;
      break;
    }
  }
  std::map<int64_t, Record> records_;
  std::map<int64_t, Receipt> receipts_;
  std::array<int64_t, kMaxReceipts> order_{};
  size_t receipt_count_ = 0;
};
}  // namespace rhwp::d3d
#endif
