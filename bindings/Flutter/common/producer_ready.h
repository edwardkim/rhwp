#ifndef RHWP_PRODUCER_READY_H_
#define RHWP_PRODUCER_READY_H_
#include <cstdint>
#include <map>
#include <mutex>
#include <utility>
#include <vector>

namespace rhwp {
// A wake-up hint, never a consumer fence or permission to expose a frame.
struct ProducerReadyFrame {
  uint64_t texture = 0, device = 0, serial = 0;
  uint64_t document = 0, revision = 0, font = 0, view = 0;
  double scale = 1;
};

// Two native registrations, at most one queued platform-thread drain. Losing a
// hint is safe: hosts must retain status polling and full publication checks.
class ProducerReadyMailbox {
 public:
  void listen(bool enabled) {
    std::lock_guard<std::mutex> lock(mutex_);
    enabled_ = enabled;
    pending_.clear(); seen_.clear(); scheduled_ = false;
  }
  bool offer(const ProducerReadyFrame& frame) {
    std::lock_guard<std::mutex> lock(mutex_);
    if (!enabled_ || !frame.texture || !frame.device || !frame.serial) return false;
    const auto identity = std::make_pair(frame.device, frame.serial);
    const auto previous = seen_.find(frame.texture);
    if (previous != seen_.end() && identity <= previous->second) return false;
    if (!pending_.count(frame.texture) && pending_.size() == 2) return false;
    if (!seen_.count(frame.texture) && seen_.size() == 2) seen_.erase(seen_.begin());
    seen_[frame.texture] = identity;
    pending_[frame.texture] = frame;
    const bool schedule = !scheduled_;
    scheduled_ = true;
    return schedule;
  }
  std::vector<ProducerReadyFrame> take() {
    std::lock_guard<std::mutex> lock(mutex_);
    std::vector<ProducerReadyFrame> result;
    if (enabled_) for (const auto& item : pending_) result.push_back(item.second);
    pending_.clear(); scheduled_ = false;
    return result;
  }
 private:
  std::mutex mutex_;
  bool enabled_ = false, scheduled_ = false;
  std::map<uint64_t, ProducerReadyFrame> pending_;
  std::map<uint64_t, std::pair<uint64_t, uint64_t>> seen_;
};
}  // namespace rhwp
#endif
