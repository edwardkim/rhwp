#ifndef RHWP_WINDOWS_PRIVATE_TARGET_POOL_H_
#define RHWP_WINDOWS_PRIVATE_TARGET_POOL_H_
#include "FrameLifetime.h"
#include <memory>
#include <utility>

namespace rhwp::d3d {
struct PrivateGeometry {
  uint64_t device = 0;
  uint32_t width = 0, height = 0;
  bool operator==(const PrivateGeometry& other) const {
    return device == other.device && width == other.width && height == other.height;
  }
};

// Only internal producer surfaces use this state machine. A public descriptor
// or its opening callback never advances private GPU completion.
template<class Resource> struct PrivateTarget {
  enum class Phase { Reserved, NativePending, NativeComplete, CopyPending, Complete };
  PrivateTarget(PrivateGeometry geometry, std::shared_ptr<ImmutableFrames::PrivateReservation> credit,
                std::shared_ptr<Resource> resource)
      : geometry(geometry), credit(std::move(credit)), resource(std::move(resource)) {}
  PrivateGeometry geometry;
  // Resource dies before the independent byte reservation is returned.
  std::shared_ptr<ImmutableFrames::PrivateReservation> credit;
  std::shared_ptr<Resource> resource;
  bool begin_native() { return transition(Phase::Reserved, Phase::NativePending); }
  bool native_complete() { return transition(Phase::NativePending, Phase::NativeComplete); }
  bool begin_copy() { return transition(Phase::NativeComplete, Phase::CopyPending); }
  bool copy_complete() { return transition(Phase::CopyPending, Phase::Complete); }
  bool skip_copy() { return transition(Phase::NativeComplete, Phase::Complete); }
  bool complete() const { return phase_ == Phase::Complete; }
 private:
  template<class> friend class PrivateTargetPool;
  bool transition(Phase expected, Phase next) {
    if (phase_ != expected) return false;
    phase_ = next; return true;
  }
  Phase phase_ = Phase::Reserved;
};

// One fully completed idle target per producer; device/geometry changes discard
// only that idle resource. Active/unknown work stays in its frame quarantine.
template<class Resource> class PrivateTargetPool {
 public:
  using Target = PrivateTarget<Resource>;
  std::shared_ptr<Target> take(PrivateGeometry geometry,
                             const std::shared_ptr<ImmutableFrames::Reservation>& frame) {
    if (!idle_) return {};
    if (!(idle_->geometry == geometry)) { idle_.reset(); return {}; }
    if (idle_.use_count() != 1 || !idle_->complete() || !frame || !frame->adopt_private(idle_->credit)) return {};
    idle_->phase_ = Target::Phase::Reserved;
    return std::move(idle_);
  }
  bool recycle(std::shared_ptr<Target>& target) {
    // Leave the caller's reference intact on rejection: dropping it could
    // destroy resources whose copy completion was never established.
    if (!target || target.use_count() != 1 || !target->complete() || !target->resource || !target->credit) return false;
    idle_.reset();
    if (!target->credit->set_idle(true)) return false;
    idle_ = std::move(target); return true;
  }
 private:
  std::shared_ptr<Target> idle_;
};
}  // namespace rhwp::d3d
#endif
