#ifndef RHWP_WINDOWS_FRAME_LIFETIME_H_
#define RHWP_WINDOWS_FRAME_LIFETIME_H_

#include <array>
#include <cmath>
#include <cstdint>
#include <limits>
#include <memory>
#include <mutex>

namespace rhwp::d3d {
enum class PublicationMode { LegacyImmutable, ImmutableOwnershipTransfer };
inline const char* publication_mode_name(PublicationMode mode) {
  return mode == PublicationMode::ImmutableOwnershipTransfer ? "immutableOwnershipTransfer" : "legacyImmutable";
}
struct Stamp {
  uint64_t document = 0, revision = 0, font = 0, view = 0, device = 0;
  double scale = 0;
  bool valid() const { return document && view && device && std::isfinite(scale) && scale > 0 && scale <= 16; }
  bool operator==(const Stamp& other) const {
    return document == other.document && revision == other.revision && font == other.font &&
        view == other.view && device == other.device && scale == other.scale;
  }
};
class GenerationGate {
 public:
  explicit GenerationGate(uint64_t device) : device_(device) {}
  uint64_t accept(const Stamp& stamp) {
    if (closed_ || !stamp.valid() || stamp.device != device_ || stamp.view <= latest_.view ||
        serial_ == std::numeric_limits<uint64_t>::max()) return 0;
    latest_ = stamp; return ++serial_;
  }
  bool current(uint64_t serial, const Stamp& stamp) const {
    return !closed_ && serial && serial == serial_ && stamp == latest_;
  }
  void close() { closed_ = true; }
 private:
  uint64_t device_, serial_ = 0;
  Stamp latest_;
  bool closed_ = false;
};

// Charge two surfaces: shared D3D12 input and legacy D3D11 destination. Round
// each to a power-of-two row-pitch/64-KiB allowance, then verify actual D3D12
// allocation info before allocation. Driver bookkeeping is outside this budget.
inline size_t frame_charge(uint64_t width, uint64_t height) {
  if (!width || !height || width > 4096 || height > 4096) return 0;
  uint64_t row = (width * 4 + 255) & ~uint64_t(255);
  uint64_t required = row * height;
  uint64_t surface = 64 * 1024;
  while (surface < required && surface < 8 * 1024 * 1024) surface *= 2;
  return surface >= required ? size_t(surface * 2) : 0;
}

// Both modes share the same process-wide four-slot/64-MiB admission limit.
// Legacy publication retains process-lifetime resources. Explicit ownership
// transfer releases only creator references to fresh immutable D3D11 resources
// after opening and CPU descriptor quiescence; the consumer's references and
// driver deferred allocations are outside this producer-ownership accounting.
// No transition permits rewriting a published target or proves GPU completion.
class ImmutableFrames {
  enum class Phase { Free, Reserved, Producing, Complete, Immutable, TransferPublished,
                     TransferPending, TransferOpened, Quarantined };
  struct Slot {
    size_t bytes = 0;
    Phase phase = Phase::Free;
    PublicationMode mode = PublicationMode::LegacyImmutable;
    std::shared_ptr<void> resources;
  };
  struct State {
    std::mutex mutex;
    std::array<Slot, 4> slots;
    size_t bytes = 0;
    uint64_t handed_off = 0, unexposed_discarded = 0;
  };
 public:
  struct Stats {
    size_t outstanding = 0, immutable = 0, quarantined = 0, bytes = 0;
    size_t producer_owned = 0, producer_owned_bytes = 0, import_pending = 0, import_pending_bytes = 0;
    size_t legacy_retained = 0, permanent = 0;
    uint64_t handed_off = 0, unexposed_discarded = 0;
  };
  class Reservation {
   public:
    ~Reservation() {
      if (!charged_) return;  // A shared_ptr control-block allocation can fail before admission.
      std::shared_ptr<void> discarded;
      {
        std::lock_guard<std::mutex> lock(state_->mutex);
        auto& slot = state_->slots[index_];
        if (slot.phase == Phase::Reserved || slot.phase == Phase::Complete ||
            slot.phase == Phase::TransferPublished || slot.phase == Phase::TransferOpened) {
          if (slot.phase == Phase::TransferOpened && state_->handed_off < uint64_t(std::numeric_limits<int64_t>::max()))
            ++state_->handed_off;
          else if (slot.phase == Phase::TransferPublished &&
              state_->unexposed_discarded < uint64_t(std::numeric_limits<int64_t>::max())) ++state_->unexposed_discarded;
          state_->bytes -= slot.bytes; discarded = std::move(slot.resources); slot = {};
        } else if (slot.phase == Phase::Producing || slot.phase == Phase::TransferPending) {
          slot.phase = Phase::Quarantined;
        }
      }
    }
    size_t bytes() const { std::lock_guard<std::mutex> lock(state_->mutex); return state_->slots[index_].bytes; }
    // Resource bundle is attached before the first producer GPU operation, so
    // exceptions/close cannot drop potentially in-flight D3D12/D3D11 resources.
    bool begin_gpu(std::shared_ptr<void> resources) {
      std::lock_guard<std::mutex> lock(state_->mutex);
      auto& slot = state_->slots[index_];
      if (!resources || slot.phase != Phase::Reserved) return false;
      slot.resources = std::move(resources); slot.phase = Phase::Producing; return true;
    }
    // Internal producer proof only: checked D3D12 completion and, if submitted,
    // checked D3D11 copy completion. Never called from Flutter release_callback.
    bool producer_complete() {
      std::lock_guard<std::mutex> lock(state_->mutex);
      auto& slot = state_->slots[index_];
      if (slot.phase != Phase::Producing) return false;
      slot.phase = Phase::Complete; return true;
    }
    bool publish_immutable() {
      std::lock_guard<std::mutex> lock(state_->mutex);
      auto& slot = state_->slots[index_];
      if (slot.phase != Phase::Complete || !slot.resources) return false;
      slot.phase = slot.mode == PublicationMode::ImmutableOwnershipTransfer ?
          Phase::TransferPublished : Phase::Immutable;
      return true;
    }
    // Only the bounded import-ticket layer calls these methods. A strong ticket
    // must pin both descriptor storage and this reservation before exposure.
    bool offer_to_consumer() {
      std::lock_guard<std::mutex> lock(state_->mutex);
      auto& slot = state_->slots[index_];
      if (slot.mode != PublicationMode::ImmutableOwnershipTransfer) return false;
      if (slot.phase == Phase::TransferPublished) slot.phase = Phase::TransferPending;
      return slot.phase == Phase::TransferPending || slot.phase == Phase::TransferOpened;
    }
    bool consumer_opened() {
      std::lock_guard<std::mutex> lock(state_->mutex);
      auto& slot = state_->slots[index_];
      if (slot.mode != PublicationMode::ImmutableOwnershipTransfer) return false;
      if (slot.phase == Phase::TransferPending) slot.phase = Phase::TransferOpened;
      return slot.phase == Phase::TransferOpened;
    }
    Reservation(const Reservation&) = delete;
    Reservation& operator=(const Reservation&) = delete;
   private:
    friend class ImmutableFrames;
    Reservation(std::shared_ptr<State> state, size_t index) : state_(std::move(state)), index_(index) {}
    std::shared_ptr<State> state_;
    size_t index_;
    bool charged_ = false;
  };
  ImmutableFrames() : state_(std::make_shared<State>()) {}
  std::shared_ptr<Reservation> reserve(size_t bytes,
      PublicationMode mode = PublicationMode::LegacyImmutable) {
    std::lock_guard<std::mutex> lock(state_->mutex);
    if (!bytes || bytes > 16 * 1024 * 1024 || bytes > 64 * 1024 * 1024 - state_->bytes) return {};
    for (size_t index = 0; index < state_->slots.size(); ++index) {
      auto& slot = state_->slots[index];
      if (slot.phase != Phase::Free) continue;
      auto result = std::shared_ptr<Reservation>(new Reservation(state_, index));
      slot.bytes = bytes; slot.phase = Phase::Reserved; slot.mode = mode; state_->bytes += bytes;
      result->charged_ = true; return result;
    }
    return {};
  }
  Stats stats() const {
    std::lock_guard<std::mutex> lock(state_->mutex);
    Stats result; result.bytes = state_->bytes;
    result.handed_off = state_->handed_off; result.unexposed_discarded = state_->unexposed_discarded;
    for (const auto& slot : state_->slots) {
      if (slot.phase != Phase::Free) ++result.outstanding;
      if (slot.phase == Phase::Immutable) ++result.immutable;
      if (slot.phase == Phase::Quarantined || slot.phase == Phase::Producing) ++result.quarantined;
      if (slot.phase == Phase::Immutable || slot.phase == Phase::Quarantined) ++result.permanent;
      if (slot.mode == PublicationMode::LegacyImmutable &&
          (slot.phase == Phase::Immutable || slot.phase == Phase::Quarantined)) ++result.legacy_retained;
      if (slot.mode == PublicationMode::ImmutableOwnershipTransfer && slot.phase != Phase::Free) {
        ++result.producer_owned; result.producer_owned_bytes += slot.bytes;
      }
      if (slot.phase == Phase::TransferPending) {
        ++result.import_pending; result.import_pending_bytes += slot.bytes;
      }
    }
    return result;
  }
 private:
  std::shared_ptr<State> state_;
};
}  // namespace rhwp::d3d
#endif
