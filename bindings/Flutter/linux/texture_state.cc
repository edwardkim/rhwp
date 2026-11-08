#include "texture_state.h"

#include <algorithm>
#include <cmath>
#include <limits>
#include <utility>

namespace rhwp_flutter {
namespace {
struct Credit { uint64_t id; std::weak_ptr<Shared> state; std::shared_ptr<Shared> orphaned; };
struct Credits { std::mutex mutex; std::vector<Credit> entries; uint64_t next = 1; };
Credits& credit_state() {
  // Raster TLS destructors can run after unrelated application static teardown.
  // Four weak records/mutex remain resident with the plugin until process exit.
  static auto* state = new Credits;
  return *state;
}
std::atomic<uint64_t> next_context{1};

void validate(const Description& frame) {
  const auto& stamp = frame.stamp;
  if (!stamp.document || !stamp.view || !stamp.device ||
      !std::isfinite(stamp.dpr) || stamp.dpr <= 0 || stamp.dpr > 16 ||
      !frame.width || !frame.height || frame.width > 4096 || frame.height > 4096 ||
      uint64_t(frame.width) * frame.height > kMaxPixels ||
      frame.pages.empty() || frame.pages.size() > kMaxPages) {
    throw Failure("invalidArgument", "invalid/beyond-budget texture frame");
  }
  for (const auto& page : frame.pages) {
    if (!page.snapshot || !std::isfinite(page.scale) || page.scale <= 0 || page.scale > 64 ||
        !std::isfinite(page.x) || !std::isfinite(page.y) ||
        std::abs(page.x) > 16777216 || std::abs(page.y) > 16777216) {
      throw Failure("invalidArgument", "invalid texture placement");
    }
  }
}

uint64_t give_credit(const std::shared_ptr<Shared>& state) {
  auto& credits = credit_state();
  std::lock_guard<std::mutex> lock(credits.mutex);
  if (credits.entries.size() >= kMaxRasterSlots) {
    throw Failure("retireLimit", "four raster slots are live or awaiting owner-thread retire");
  }
  const auto id = credits.next++;
  credits.entries.push_back(Credit{id, state, nullptr});
  return id;
}

void take_credit(uint64_t id, bool orphaned = false) {
  auto& credits = credit_state();
  std::lock_guard<std::mutex> lock(credits.mutex);
  auto found = std::find_if(credits.entries.begin(), credits.entries.end(), [&](const auto& entry) {
    return entry.id == id;
  });
  if (found != credits.entries.end()) {
    if (orphaned) found->orphaned = found->state.lock();
    else credits.entries.erase(found);
  }
}
}  // namespace

uint64_t Shared::submit(Description description) {
  validate(description);
  // Inspect before consuming any new ID. SnapshotInfo is a cross-thread
  // registry read, never a legacy worker/core/layout request.
  for (const auto& page : description.pages) {
    const auto info = bridge->inspect(page.snapshot);
    if (!info.current || !info.screen || info.document != description.stamp.document ||
        info.revision != description.stamp.revision || info.font != description.stamp.font) {
      throw Failure("staleFrame", "snapshot does not match current frame source");
    }
  }
  std::shared_ptr<const Frame> previous;
  uint64_t serial;
  {
    std::lock_guard<std::mutex> lock(mutex_);
    if (status_.closing) throw Failure("closing", "texture is closing");
    if (description.stamp.device != status_.device) {
      throw Failure("contextChanged", "query status and configure the new device generation");
    }
    if (latest_ && description.stamp.view <= latest_->description.stamp.view) {
      throw Failure("staleFrame", "view generation must strictly advance");
    }
    if (status_.accepted == std::numeric_limits<uint64_t>::max()) {
      throw Failure("limitExceeded", "texture serial exhausted");
    }
    auto frame = std::make_shared<Frame>();
    frame->description = std::move(description);
    frame->serial = status_.accepted + 1;
    std::vector<std::weak_ptr<SnapshotLease>> next;
    next.reserve(2 * kMaxPages);
    for (const auto& weak : leases_) if (!weak.expired()) next.push_back(weak);
    for (const auto& page : frame->description.pages) {
      std::shared_ptr<SnapshotLease> lease;
      for (const auto& weak : next) {
        auto existing = weak.lock();
        if (existing && existing->id == page.snapshot) { lease = existing; break; }
      }
      if (!lease) {
        if (next.size() >= 2 * kMaxPages) {
          throw Failure("limitExceeded", "snapshot slots are full while raster work is in flight");
        }
        lease = std::make_shared<SnapshotLease>();
        lease->bridge = bridge;
        lease->id = page.snapshot;
        next.push_back(lease);
      }
      frame->leases.push_back(std::move(lease));
    }
    // No throwing operations remain. Failure above leaves all new IDs owned by
    // the caller; accepted frames keep exactly one lease per distinct ID.
    for (const auto& lease : frame->leases) lease->owned = true;
    previous = std::move(latest_);
    latest_ = std::move(frame);
    leases_ = std::move(next);
    serial = status_.accepted = latest_->serial;
    status_.accepted_stamp = latest_->description.stamp;
    status_.code.clear();
    status_.message.clear();
  }
  // Registry release never happens while holding the GUI/raster metadata lock.
  previous.reset();
  return serial;
}

void Shared::close() {
  std::shared_ptr<const Frame> previous;
  {
    std::lock_guard<std::mutex> lock(mutex_);
    status_.closing = true;
    previous = std::move(latest_);
    if (!status_.raster_owner) status_.cleanup_complete = true;
  }
  previous.reset();
}

Status Shared::status() const {
  std::lock_guard<std::mutex> lock(mutex_);
  return status_;
}

void Shared::error(const std::string& code, const std::string& message) {
  std::lock_guard<std::mutex> lock(mutex_);
  status_.code = code.substr(0, 48);
  status_.message = message.substr(0, 512);
}

struct RasterRegistry::Slot {
  std::shared_ptr<Shared> state;
  std::shared_ptr<Context> context;
  std::thread::id owner = std::this_thread::get_id();
  uint64_t identity = 0, generation = 0, presenter = 0, credit = 0;
  Target front, back;

  static const void* load(void* user, const char* name) noexcept {
    auto* slot = static_cast<Slot*>(user);
    return slot->owner == std::this_thread::get_id() ? slot->context->procedure(name) : nullptr;
  }
  static int32_t current(void* user, uint64_t identity) noexcept {
    auto* slot = static_cast<Slot*>(user);
    return identity == slot->identity && slot->owner == std::this_thread::get_id() &&
           slot->context->same_current() ? 1 : 0;
  }
};

RasterRegistry::RasterRegistry() = default;

size_t RasterRegistry::live_slots() {
  auto& credits = credit_state();
  std::lock_guard<std::mutex> lock(credits.mutex);
  return credits.entries.size();
}

size_t RasterRegistry::pending_retire() {
  auto& credits = credit_state();
  std::lock_guard<std::mutex> lock(credits.mutex);
  return std::count_if(credits.entries.begin(), credits.entries.end(), [](const auto& entry) {
    auto state = entry.state.lock();
    return entry.orphaned || (state && state->status().closing);
  });
}

size_t RasterRegistry::orphaned_slots() {
  auto& credits = credit_state();
  std::lock_guard<std::mutex> lock(credits.mutex);
  return std::count_if(credits.entries.begin(), credits.entries.end(), [](const auto& entry) {
    return bool(entry.orphaned);
  });
}

void RasterRegistry::retire(size_t index, bool thread_exit) {
  auto& slot = *slots_[index];
  const bool current = !thread_exit && slot.context->same_current();
  bool native_clean = true;
  bool abandoned_owner = !current && slot.presenter;
  try {
    if (slot.presenter) {
      if (current) slot.state->bridge->release(slot.presenter, slot.identity, slot.generation);
      else slot.state->bridge->abandon(slot.presenter);
    }
  } catch (const std::exception& error) {
    // No GL after a release failure. The owner-thread ABI/TLS destructor always
    // drops an abandoned owner without calling its callback userdata.
    abandoned_owner = bool(slot.presenter);
    try { if (slot.presenter) slot.state->bridge->abandon(slot.presenter); }
    catch (...) { native_clean = false; }
    slot.state->error("cleanup", error.what());
  }
  if (!native_clean && !thread_exit) {
    // A failed ABI cleanup cannot invalidate retained callback userdata. Keep
    // this credit/owner until a later callback retry or native TLS teardown.
    if (slot.state->notify) slot.state->notify(slot.state);
    return;
  }
  const bool destroy_targets = current && !abandoned_owner;
  const bool orphaned = (abandoned_owner || (!destroy_targets && (slot.front.texture || slot.back.texture))) &&
      !slot.context->abandon_reclaims_resources();
  if (destroy_targets) {
    slot.context->destroy(slot.front);
    slot.context->destroy(slot.back);
  }
  {
    std::lock_guard<std::mutex> lock(slot.state->mutex_);
    if (!destroy_targets) slot.state->status_.abandoned_targets +=
        unsigned(slot.front.texture != 0) + unsigned(slot.back.texture != 0);
    if (orphaned) ++slot.state->status_.orphaned_owners;
    slot.state->status_.raster_owner = false;
    slot.state->status_.cleanup_complete = slot.state->status_.closing;
    slot.state->status_.owner_thread_exit = thread_exit;
    if (thread_exit && !slot.state->status_.closing &&
        slot.state->status_.device != std::numeric_limits<uint64_t>::max()) {
      ++slot.state->status_.device;
      slot.state->status_.code = "contextChanged";
      slot.state->status_.message = "raster owner exited; configure the next device generation";
    }
  }
  auto state = slot.state;
  const auto credit = slot.credit;
  slots_.erase(slots_.begin() + index);  // Callbacks remain alive through ABI cleanup.
  // Abandon cannot prove GL share-group allocations were deleted. Keep its
  // process credit rather than permitting unbounded allocate/abandon cycles.
  take_credit(credit, orphaned);
  if (state->notify) state->notify(state);
}

RasterRegistry::~RasterRegistry() {
  while (!slots_.empty()) retire(slots_.size() - 1, true);
}

void RasterRegistry::drain_retired(const std::shared_ptr<Shared>& sampled) {
  if (owner_ != std::this_thread::get_id()) {
    throw Failure("wrongThread", "raster registry cannot move between OS threads");
  }
  for (size_t index = slots_.size(); index-- > 0;) {
    // Only its own populate guarantees Flutter has invalidated this texture's
    // cached image. Another texture's callback is not a release/fence signal.
    if (slots_[index]->state == sampled && sampled->status().closing) retire(index, false);
  }
}

Target RasterRegistry::populate(const std::shared_ptr<Shared>& state, std::shared_ptr<Context> context) {
  drain_retired(state);
  std::shared_ptr<const Frame> frame;
  {
    std::lock_guard<std::mutex> lock(state->mutex_);
    if (state->status_.closing) throw Failure("closing", "texture is retiring");
    frame = state->latest_;
  }
  if (!frame) throw Failure("notReady", "submit a frame before sampling its texture");
  auto found = std::find_if(slots_.begin(), slots_.end(), [&](const auto& item) { return item->state == state; });
  if (found != slots_.end() && !(*found)->context->same_context(*context)) {
    auto index = size_t(found - slots_.begin());
    const auto previous_size = slots_.size();
    retire(index, false);
    if (slots_.size() == previous_size) throw Failure("cleanup", "previous context retire is still pending");
    {
      std::lock_guard<std::mutex> lock(state->mutex_);
      if (state->status_.device == std::numeric_limits<uint64_t>::max()) {
        throw Failure("limitExceeded", "device generation exhausted");
      }
      ++state->status_.device;
    }
    throw Failure("contextChanged", "configure and submit with the new device generation");
  }
  if (found == slots_.end()) {
    const auto credit = give_credit(state);
    bool claimed = false;
    try {
      auto slot = std::make_unique<Slot>();
      slot->state = state;
      slot->credit = credit;
      slot->context = std::move(context);
      slot->identity = next_context.fetch_add(1);
      if (!slot->identity) throw Failure("limitExceeded", "context identity exhausted");
      const auto driver = slot->context->driver_info();
      {
        std::lock_guard<std::mutex> lock(state->mutex_);
        if (state->status_.closing) throw Failure("closing", "texture is retiring");
        if (state->status_.raster_owner) {
          throw Failure("wrongThread", "texture is still owned by another raster OS thread");
        }
        if (frame->description.stamp.device != state->status_.device) {
          throw Failure("contextChanged", "frame belongs to an earlier device generation");
        }
        slot->generation = state->status_.device;
        state->status_.driver = {driver.vendor.substr(0, 256), driver.renderer.substr(0, 256),
                                driver.version.substr(0, 256)};
        state->status_.raster_owner = true;
        state->status_.owner_thread_exit = false;
        claimed = true;
      }
      slots_.push_back(std::move(slot));
      found = slots_.end() - 1;
    } catch (...) {
      if (claimed) {
        std::lock_guard<std::mutex> lock(state->mutex_);
        state->status_.raster_owner = false;
        state->status_.cleanup_complete = state->status_.closing;
      }
      take_credit(credit);
      throw;
    }
  }
  Slot& slot = **found;
  try {
    if (!slot.presenter) {
      slot.presenter = state->bridge->create(slot.identity, slot.generation,
          Callbacks{Slot::load, Slot::current, &slot});
      if (!slot.presenter) throw Failure("contract", "native returned a zero presenter handle");
    }
    if (state->status().rendered == frame->serial && slot.front.texture) return slot.front;
    const auto& description = frame->description;
    if (slot.back.width != description.width || slot.back.height != description.height) {
      slot.context->destroy(slot.back);
      slot.back = slot.context->allocate(description.width, description.height);
    }
    const auto readbacks = state->bridge->render(slot.presenter, slot.identity, description, slot.back);
    if (readbacks) {
      {
        std::lock_guard<std::mutex> lock(state->mutex_);
        state->status_.whole_frame_readbacks = readbacks;
      }
      throw Failure("contract", "GPU bridge reported a whole-frame CPU readback");
    }
    bool committed = false;
    {
      std::lock_guard<std::mutex> lock(state->mutex_);
      if (!state->status_.closing && state->latest_ == frame &&
          state->status_.device == description.stamp.device) {
        std::swap(slot.front, slot.back);
        state->status_.rendered = frame->serial;
        state->status_.rendered_stamp = description.stamp;
        ++state->status_.frames;
        state->status_.whole_frame_readbacks = readbacks;
        state->status_.code.clear();
        state->status_.message.clear();
        committed = true;
      }
    }
    if (!committed && state->notify) state->notify(state);
  } catch (const Failure& error) {
    state->error(error.code, error.what());
    if (!slot.front.texture) throw;
  }
  if (!slot.front.texture) throw Failure("notReady", "latest frame changed during GPU replay");
  return slot.front;
}

}  // namespace rhwp_flutter
