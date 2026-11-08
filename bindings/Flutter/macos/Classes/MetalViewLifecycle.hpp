#ifndef RHWP_METAL_VIEW_LIFECYCLE_HPP
#define RHWP_METAL_VIEW_LIFECYCLE_HPP

#include <cmath>
#include <cstdint>
#include <optional>

namespace rhwp::metal {

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
