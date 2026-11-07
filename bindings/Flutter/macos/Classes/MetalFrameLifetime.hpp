#ifndef RHWP_METAL_FRAME_LIFETIME_HPP
#define RHWP_METAL_FRAME_LIFETIME_HPP

#include <cmath>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <memory>
#include <mutex>
#include <utility>
#include <vector>

namespace rhwp::metal {

struct Stamp {
  uint64_t document = 0, revision = 0, font = 0, view = 0, device = 0;
  double device_scale = 0;
  bool valid() const {
    return document && view && device && std::isfinite(device_scale) &&
           device_scale > 0 && device_scale <= 16;
  }
  bool operator==(const Stamp& other) const {
    return document == other.document && revision == other.revision &&
           font == other.font && view == other.view && device == other.device &&
           device_scale == other.device_scale;
  }
};

// The plugin serializes this gate with its pending/current mailbox mutex.
// Identity includes the whole source/device stamp, never just a view integer.
class GenerationGate {
 public:
  explicit GenerationGate(uint64_t device) : device_(device) {}
  uint64_t accept(const Stamp& stamp) {
    if (closed_ || !stamp.valid() || stamp.device != device_ ||
        stamp.view < latest_.view || serial_ == std::numeric_limits<uint64_t>::max()) {
      return 0;
    }
    latest_ = stamp;
    return ++serial_;
  }
  bool current(uint64_t serial, const Stamp& stamp) const {
    return !closed_ && serial != 0 && serial == serial_ && stamp == latest_;
  }
  void close() { closed_ = true; }

 private:
  uint64_t device_, serial_ = 0;
  Stamp latest_;
  bool closed_ = false;
};

// No consumer-retire operation exists. A returned CVPixelBuffer, an engine
// unregister notification, and producer GPU completion prove no such event.
// The production instance is process-lifetime and holds every exposed frame.
class ImmutableFrameCredits {
 public:
  struct Stats {
    size_t outstanding_frames, retained_pending_retire, reserved_bytes;
  };

 private:
  struct State {
    State(size_t frames, size_t bytes, size_t single)
        : max_frames(frames), max_bytes(bytes), max_single(single) {
      // Publication must never allocate after a consumer might see the frame.
      immutable.reserve(frames);
    }
    std::mutex mutex;
    size_t max_frames, max_bytes, max_single;
    size_t outstanding = 0, bytes = 0;
    std::vector<std::shared_ptr<void>> immutable;
  };

 public:
  class Reservation {
   public:
    ~Reservation() {
      if (!charged_ || retained_) return;
      std::lock_guard<std::mutex> lock(state_->mutex);
      --state_->outstanding;
      state_->bytes -= bytes_;
    }
    Reservation(const Reservation&) = delete;
    Reservation& operator=(const Reservation&) = delete;
    size_t bytes() const { return bytes_; }

    // Keep pixels plus CVMetalTexture strongly held. This also quarantines a
    // producer failure when GPU completion is unknown. It never refunds credit.
    void retain_until_process_exit(std::shared_ptr<void> pixels) {
      if (!pixels) return;
      std::lock_guard<std::mutex> lock(state_->mutex);
      if (retained_) return;
      state_->immutable.push_back(std::move(pixels));
      retained_ = true;
    }

   private:
    friend class ImmutableFrameCredits;
    Reservation(std::shared_ptr<State> state, size_t bytes)
        : state_(std::move(state)), bytes_(bytes) {}
    std::shared_ptr<State> state_;
    size_t bytes_;
    bool charged_ = false, retained_ = false;
  };

  ImmutableFrameCredits(size_t frames, size_t bytes, size_t single)
      : state_(std::make_shared<State>(frames, bytes, single)) {}
  std::shared_ptr<Reservation> reserve(size_t bytes) {
    std::lock_guard<std::mutex> lock(state_->mutex);
    if (!bytes || bytes > state_->max_single || state_->outstanding >= state_->max_frames ||
        bytes > state_->max_bytes - state_->bytes) {
      return {};
    }
    auto result = std::shared_ptr<Reservation>(new Reservation(state_, bytes));
    ++state_->outstanding;
    state_->bytes += bytes;
    result->charged_ = true;
    return result;
  }
  Stats stats() const {
    std::lock_guard<std::mutex> lock(state_->mutex);
    return {state_->outstanding, state_->immutable.size(), state_->bytes};
  }

 private:
  std::shared_ptr<State> state_;
};

}  // namespace rhwp::metal
#endif
