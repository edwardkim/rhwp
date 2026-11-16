#ifndef RHWP_GPU_CACHE_TRIM_H_
#define RHWP_GPU_CACHE_TRIM_H_
#include <cstdint>
#include <limits>
#include <mutex>
#include <optional>
#include <stdexcept>

namespace rhwp::cache {
constexpr uint64_t kMaxTrimBytes = 192 * 1024 * 1024;
struct TrimReceipt {
  uint64_t request = 0, target = 0, before = 0, after = 0, trimmed = 0;
  bool pending = false, busy = true;
  const char* reason = "notRequested";
};
// One owner request and one receipt. UI cancellation or duplicate polling never
// establishes GPU completion; only the actual producer supplies before/after.
class TrimMailbox {
 public:
  TrimReceipt enqueue(uint64_t id, uint64_t target) {
    if (!id || id > uint64_t(std::numeric_limits<int64_t>::max()) || target > kMaxTrimBytes)
      throw std::invalid_argument("invalid bounded cache trim request");
    std::lock_guard<std::mutex> lock(mutex_);
    if (id == receipt_.request) {
      if (target != receipt_.target) throw std::invalid_argument("cache trim ID changed its target");
      return receipt_;
    }
    if (receipt_.pending) throw std::invalid_argument("one cache trim request is already pending");
    if (id < receipt_.request) throw std::invalid_argument("stale cache trim request ID");
    receipt_ = {id, target, 0, 0, 0, true, true, "queued"};
    queued_ = true; return receipt_;
  }
  TrimReceipt poll(uint64_t id) const {
    std::lock_guard<std::mutex> lock(mutex_);
    if (!id || id != receipt_.request) throw std::invalid_argument("unknown cache trim request ID");
    return receipt_;
  }
  bool queued() const { std::lock_guard<std::mutex> lock(mutex_); return queued_; }
  std::optional<TrimReceipt> take() {
    std::lock_guard<std::mutex> lock(mutex_);
    if (!queued_) return {};
    queued_ = false; return receipt_;
  }
  bool finish(uint64_t id, uint64_t before, uint64_t after, uint64_t trimmed, bool busy) {
    std::lock_guard<std::mutex> lock(mutex_);
    if (id != receipt_.request || !receipt_.pending || queued_) return false;
    if (before > kMaxTrimBytes || after > before || trimmed != before - after ||
        (!busy && after > receipt_.target)) throw std::invalid_argument("contradictory Native cache trim receipt");
    receipt_.before = before; receipt_.after = after; receipt_.trimmed = trimmed;
    receipt_.pending = false; receipt_.busy = busy;
    receipt_.reason = busy ? "pinnedOrUnknownCompletion" : "completed"; return true;
  }
  void fail(uint64_t id, const char* reason = "nativeError") {
    std::lock_guard<std::mutex> lock(mutex_);
    if (id != receipt_.request || !receipt_.pending) return;
    queued_ = false; receipt_.pending = false; receipt_.busy = true; receipt_.reason = reason;
  }
  void cancel() {
    std::lock_guard<std::mutex> lock(mutex_);
    queued_ = false;
    if (receipt_.pending) { receipt_.pending = false; receipt_.busy = true; receipt_.reason = "closed"; }
  }
 private:
  mutable std::mutex mutex_;
  TrimReceipt receipt_;
  bool queued_ = false;
};
}  // namespace rhwp::cache
#endif
