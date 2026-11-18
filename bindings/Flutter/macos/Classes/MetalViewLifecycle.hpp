#ifndef RHWP_METAL_VIEW_LIFECYCLE_HPP
#define RHWP_METAL_VIEW_LIFECYCLE_HPP

#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <map>
#include <memory>
#include <optional>

namespace rhwp::metal {

// Platform-thread registry: unfinished and unacknowledged closes retain their
// exact handle. Only an explicit acknowledgment after producer completion can
// release a record; a bounded cache holds resource-free completion receipts.
template <typename Value, typename Receipt, size_t Capacity = 2>
class RetirementRegistry {
 public:
  static_assert(Capacity > 0, "retirement registry requires a positive bound");
  struct Record { std::shared_ptr<Value> value; bool closing = false; };
  bool insert(uint64_t handle, std::shared_ptr<Value> value) {
    if (!handle || !value || records_.size() >= Capacity || records_.count(handle) || receipts_.count(handle)) return false;
    return records_.emplace(handle, Record{std::move(value), false}).second;
  }
  std::shared_ptr<Value> lookup(uint64_t handle) const {
    auto found = records_.find(handle);
    return found == records_.end() ? nullptr : found->second.value;
  }
  bool registered(uint64_t handle) const {
    auto found = records_.find(handle);
    return found != records_.end() && !found->second.closing;
  }
  bool close(uint64_t handle) {
    auto found = records_.find(handle);
    if (found == records_.end()) return false;
    found->second.closing = true;
    return true;
  }
  bool acknowledge(uint64_t handle, bool completionConfirmed, Receipt receipt) {
    if (receipts_.count(handle)) return true;
    auto found = records_.find(handle);
    if (!completionConfirmed || found == records_.end() || !found->second.closing) return false;
    // Allocate before removing the only observable owner-completion record.
    receipts_.emplace(handle, std::move(receipt));
    records_.erase(found);
    if (receipts_.size() > Capacity) {
      receipts_.erase(receipt_order_[0]);
      for (size_t i = 1; i < Capacity; ++i) receipt_order_[i-1] = receipt_order_[i];
      receipt_order_[Capacity-1] = handle;
    } else receipt_order_[receipts_.size()-1] = handle;
    return true;
  }
  const Receipt* receipt(uint64_t handle) const {
    auto found = receipts_.find(handle);
    return found == receipts_.end() ? nullptr : &found->second;
  }
  void erase_unstarted(uint64_t handle) { records_.erase(handle); }
  size_t size() const { return records_.size(); }
  size_t receipt_count() const { return receipts_.size(); }
  const std::map<uint64_t, Record>& records() const { return records_; }
  const std::map<uint64_t, Receipt>& receipts() const { return receipts_; }
 private:
  std::map<uint64_t, Record> records_;
  std::map<uint64_t, Receipt> receipts_;
  std::array<uint64_t, Capacity> receipt_order_{};
};

// Matches the Dart controller's ceil(logical viewport * DPR) admission. A
// resize never clamps an old drawable into the new requested geometry.
struct DrawableSize {
  uint32_t width, height;
  static std::optional<DrawableSize> FromBacking(double width, double height) {
    if (!std::isfinite(width) || !std::isfinite(height) || width <= 0 || height <= 0) return {};
    width = std::ceil(width);
    height = std::ceil(height);
    if (width > 4096 || height > 4096 || width * height > 4194304) return {};
    return DrawableSize{static_cast<uint32_t>(width), static_cast<uint32_t>(height)};
  }
  bool matches(uint64_t otherWidth, uint64_t otherHeight) const {
    return width == otherWidth && height == otherHeight;
  }
};

// Visibility requests are ordered independently of producer generations. A
// delayed uncover can never undo a newer cover, even when the Native source has
// not applied a queued edit yet. The caller supplies a full-stamp check result.
class VisibilityGate {
 public:
  bool apply(uint64_t serial, bool covered, bool currentFrameMatches) {
    if (closed_ || !serial || serial <= serial_) return false;
    serial_ = serial;
    covered_ = covered || !currentFrameMatches;
    return covered || currentFrameMatches;
  }
  void invalidate() { covered_ = true; }
  void close() { closed_ = true; covered_ = true; }
  uint64_t serial() const { return serial_; }
  bool covered() const { return covered_; }
 private:
  uint64_t serial_ = 0;
  bool covered_ = true, closed_ = false;
};

// One worker borrows at most one drawable. Handoff transfers presentation to
// CAMetalLayer's bounded pool; it is not a consumer-GPU completion receipt.
// Access is serialized by the associated producer state mutex.
class DrawableLifecycle {
 public:
  enum class Phase { idle, producing, producerComplete, quarantined };

  bool acquire(uint64_t ticket) {
    if (closed_ || phase_ != Phase::idle || !ticket || ticket <= last_ticket_) return false;
    last_ticket_ = ticket;
    phase_ = Phase::producing;
    return true;
  }
  bool producer_complete(uint64_t ticket) {
    if (ticket != last_ticket_ || phase_ != Phase::producing) return false;
    phase_ = Phase::producerComplete;
    return true;
  }
  bool discard_unsubmitted(uint64_t ticket) {
    if (ticket != last_ticket_ || phase_ != Phase::producing) return false;
    phase_ = Phase::idle;
    ++discarded_;
    return true;
  }
  bool discard_completed(uint64_t ticket) {
    if (ticket != last_ticket_ || phase_ != Phase::producerComplete) return false;
    phase_ = Phase::idle;
    ++discarded_;
    return true;
  }
  bool handoff(uint64_t ticket) {
    if (closed_ || ticket != last_ticket_ || phase_ != Phase::producerComplete) return false;
    phase_ = Phase::idle;
    ++scheduled_;
    return true;
  }
  void unknown_failure(uint64_t ticket) {
    if (ticket == last_ticket_ && phase_ != Phase::idle) {
      phase_ = Phase::quarantined;
      closed_ = true;
    }
  }
  void close() { closed_ = true; }
  bool finish_owner() {
    if (!closed_ || phase_ != Phase::idle) return false;
    finished_ = true;
    return true;
  }
  bool closed() const { return closed_; }
  bool finished() const { return finished_; }
  bool quarantined() const { return phase_ == Phase::quarantined; }
  bool busy() const { return phase_ != Phase::idle; }
  uint64_t scheduled() const { return scheduled_; }
  uint64_t discarded() const { return discarded_; }
  Phase phase() const { return phase_; }

 private:
  Phase phase_ = Phase::idle;
  uint64_t last_ticket_ = 0, scheduled_ = 0, discarded_ = 0;
  bool closed_ = false, finished_ = false;
};

}  // namespace rhwp::metal
#endif
