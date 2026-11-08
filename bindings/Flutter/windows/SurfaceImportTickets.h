#ifndef RHWP_WINDOWS_SURFACE_IMPORT_TICKETS_H_
#define RHWP_WINDOWS_SURFACE_IMPORT_TICKETS_H_

#include "FrameLifetime.h"
#include <map>
#include <stdexcept>
#include <thread>
#include <vector>

namespace rhwp::d3d {

// Opening is a creator-reference handoff, not a GPU fence. The pinned Windows
// engine reads descriptor dimensions *after* calling release_callback. Keep its
// immutable descriptor and frame pin until a later callback on the same raster
// OS thread, or confirmed registration removal, proves those CPU reads ended.
// The engine retains its last HANDLE value even after a null callback destroys
// its EGL surface. Separately keep the last exported creator allocation/credit
// alive until a distinct replacement is offered, or confirmed unregister. CPU
// descriptor retirement alone must not permit that cached HANDLE to be recycled.
// Opaque never-reused integer cookies permit arbitrary late/duplicate opening
// notifications without dereferencing a freed userdata allocation.
template <typename Descriptor>
class SurfaceImportTickets {
 public:
  static constexpr size_t kMaxTickets = 8;
  static constexpr size_t kMaxOwners = 2;
  using Notification = void (*)(void*) noexcept;
  struct Stats {
    size_t tickets = 0, cpu_pending = 0, awaiting_opened = 0;
    uint64_t opened = 0, ignored = 0;
  };
  class Callback {
   public:
    Callback() = default;
    Callback(const Callback&) = delete;
    Callback& operator=(const Callback&) = delete;
    Callback(Callback&& other) noexcept : owner_(other.owner_) { other.owner_ = 0; }
    ~Callback() { if (owner_) finish_callback(owner_); }
    explicit operator bool() const { return owner_ != 0; }
   private:
    friend class SurfaceImportTickets;
    explicit Callback(uint64_t owner) : owner_(owner) {}
    uint64_t owner_ = 0;
  };

  static void register_owner(uint64_t owner) {
    if (!owner) throw std::invalid_argument("zero texture import owner");
    auto& data = storage();
    std::lock_guard<std::mutex> lock(data.mutex);
    if (data.owners.count(owner) || data.owners.size() >= kMaxOwners)
      throw std::runtime_error("texture import owner capacity exceeded");
    data.owners.emplace(owner, Owner{});
  }

  static Callback begin_callback(uint64_t owner) {
    std::vector<std::shared_ptr<Ticket>> discarded;
    auto& data = storage();
    {
      std::lock_guard<std::mutex> lock(data.mutex);
      auto found = data.owners.find(owner);
      if (found == data.owners.end() || found->second.closed) return {};
      auto& state = found->second;
      const auto current = std::this_thread::get_id();
      if (state.inside || (state.thread != std::thread::id{} && state.thread != current))
        throw std::runtime_error("texture import callback changed raster thread or reentered");
      state.thread = current;
      retire_cpu_locked(data, owner, discarded);
      state.inside = true;
    }
    return Callback(owner);
  }

  static const Descriptor* offer(uint64_t owner, const Descriptor& description,
      std::shared_ptr<void> frame, std::shared_ptr<ImmutableFrames::Reservation> credit,
      Notification notify = nullptr, void* notify_context = nullptr) {
    if (!frame || !credit || !description.handle) return nullptr;
    auto ticket = std::make_shared<Ticket>();
    ticket->owner = owner; ticket->descriptor = description;
    ticket->frame = std::move(frame); ticket->credit = std::move(credit);
    ticket->notify = notify; ticket->notify_context = notify_context;
    std::shared_ptr<void> retired_frame;
    std::shared_ptr<ImmutableFrames::Reservation> retired_credit;
    auto& data = storage();
    std::lock_guard<std::mutex> lock(data.mutex);
    auto found = data.owners.find(owner);
    if (found == data.owners.end() || found->second.closed || !found->second.inside ||
        found->second.thread != std::this_thread::get_id() || data.tickets.size() >= kMaxTickets ||
        !data.next_cookie) return nullptr;
    auto& state = found->second;
    if (state.last_frame && state.last_handle == description.handle && state.last_frame != ticket->frame)
      return nullptr;  // A different allocation cannot inherit the engine's cached HANDLE.
    const uintptr_t cookie = data.next_cookie;
    data.next_cookie = cookie == std::numeric_limits<uintptr_t>::max() ? 0 : cookie + 1;
    ticket->descriptor.release_context = reinterpret_cast<void*>(cookie);
    ticket->descriptor.release_callback = &opened_callback;
    data.tickets.emplace(cookie, ticket);
    if (!ticket->credit->offer_to_consumer()) {
      data.tickets.erase(cookie);
      return nullptr;
    }
    retired_frame = std::move(state.last_frame);
    retired_credit = std::move(state.last_credit);
    state.last_frame = ticket->frame;
    state.last_credit = ticket->credit;
    state.last_handle = description.handle;
    // The old creator references drop after unlocking. Both pins consume their
    // existing process-wide frame credits; this does not add a resource pool.
    return &ticket->descriptor;
  }

  // Call only from a confirmed registrar-unregister callback, never close().
  // A contradictory receipt during our callback is kept closed and pinned; it
  // cannot establish completion of the engine's subsequent descriptor reads.
  static void unregistered(uint64_t owner) {
    std::vector<std::shared_ptr<Ticket>> discarded;
    std::shared_ptr<void> retired_frame;
    std::shared_ptr<ImmutableFrames::Reservation> retired_credit;
    auto& data = storage();
    {
      std::lock_guard<std::mutex> lock(data.mutex);
      auto found = data.owners.find(owner);
      if (found == data.owners.end()) return;
      if (found->second.inside) { found->second.closed = true; return; }
      retire_cpu_locked(data, owner, discarded);
      retired_frame = std::move(found->second.last_frame);
      retired_credit = std::move(found->second.last_credit);
      data.owners.erase(found);
    }
  }

  static Stats stats() {
    auto& data = storage();
    std::lock_guard<std::mutex> lock(data.mutex);
    Stats result; result.tickets = data.tickets.size();
    result.opened = data.opened; result.ignored = data.ignored;
    for (const auto& entry : data.tickets) {
      if (!entry.second->cpu_retired) ++result.cpu_pending;
      if (!entry.second->opened) ++result.awaiting_opened;
    }
    return result;
  }

 private:
  struct Ticket {
    Descriptor descriptor{};
    uint64_t owner = 0;
    std::shared_ptr<void> frame;
    std::shared_ptr<ImmutableFrames::Reservation> credit;
    Notification notify = nullptr;
    void* notify_context = nullptr;
    bool opened = false, cpu_retired = false;
  };
  struct Owner {
    std::thread::id thread;
    std::shared_ptr<void> last_frame;
    std::shared_ptr<ImmutableFrames::Reservation> last_credit;
    void* last_handle = nullptr;
    bool inside = false, closed = false;
  };
  struct Storage {
    std::mutex mutex;
    std::map<uint64_t, Owner> owners;
    std::map<uintptr_t, std::shared_ptr<Ticket>> tickets;
    uintptr_t next_cookie = 1;
    uint64_t opened = 0, ignored = 0;
  };
  static Storage& storage() { static auto* value = new Storage; return *value; }
  static void increment(uint64_t& value) {
    if (value < uint64_t(std::numeric_limits<int64_t>::max())) ++value;
  }
  static void finish_callback(uint64_t owner) noexcept {
    try {
      auto& data = storage();
      std::lock_guard<std::mutex> lock(data.mutex);
      auto found = data.owners.find(owner);
      if (found != data.owners.end()) found->second.inside = false;
    } catch (...) { /* Keep ownership/descriptor records pinned on failure. */ }
  }
  static void retire_cpu_locked(Storage& data, uint64_t owner,
      std::vector<std::shared_ptr<Ticket>>& discarded) {
    // Allocate before mutating any record: allocation failure leaves it pinned.
    discarded.reserve(kMaxTickets);
    for (auto it = data.tickets.begin(); it != data.tickets.end();) {
      if (it->second->owner != owner) { ++it; continue; }
      it->second->cpu_retired = true;
      if (it->second->opened) {
        discarded.push_back(std::move(it->second)); it = data.tickets.erase(it);
      } else ++it;
    }
  }
  static void opened_callback(void* context) noexcept {
    std::shared_ptr<Ticket> ticket;
    try {
      auto& data = storage();
      {
        std::lock_guard<std::mutex> lock(data.mutex);
        const auto cookie = reinterpret_cast<uintptr_t>(context);
        auto found = data.tickets.find(cookie);
        if (found == data.tickets.end() || found->second->opened) {
          increment(data.ignored); return;
        }
        ticket = found->second;
        if (!ticket->credit->consumer_opened()) return;
        ticket->opened = true; increment(data.opened);
        if (ticket->cpu_retired) data.tickets.erase(found);
      }
      if (ticket->notify) ticket->notify(ticket->notify_context);
    } catch (...) { /* Unproven opening must not release a creator reference. */ }
  }
};
}  // namespace rhwp::d3d
#endif
