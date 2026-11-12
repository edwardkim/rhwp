#ifndef RHWP_METAL_TILE_GROUP_HPP
#define RHWP_METAL_TILE_GROUP_HPP

#include "MetalFrameLifetime.hpp"
#include "MetalViewLifecycle.hpp"
#include "../../common/bounded_tile_group.h"

namespace rhwp::metal {

// Both members must belong to the same current batch. A producer-ready tile is
// not independently publishable, including when the other member is stale.
inline bool TileFramesCurrent(bool firstCurrent, const Stamp& first,
                              bool secondCurrent, const Stamp& second) {
  return firstCurrent && secondCurrent && first.valid() && first == second;
}

struct TileCoverResult { bool accepted, changed; };
inline TileCoverResult ApplyTileCover(VisibilityGate& first, VisibilityGate& second,
                                     uint64_t serial, bool covered,
                                     bool bothCurrent, bool closed) {
  if (closed || !serial || serial <= first.serial() || serial <= second.serial()) return {false, false};
  // A newer invalid uncover consumes its serial and keeps both members hidden.
  // An old request above changes neither current visibility nor serial.
  const bool a = first.apply(serial, covered, bothCurrent);
  const bool b = second.apply(serial, covered, bothCurrent);
  return {a && b, true};
}

template <class Registry, class Receipt>
bool AcknowledgeMetalTileGroup(Registry& registry, const rhwp::tiles::Identity& identity,
                               bool bothComplete, std::array<Receipt, 2> receipts) {
  if (!identity.valid() || !bothComplete) return false;
  for (const auto id : identity.textures) {
    if (!registry.lookup(id) || registry.registered(id)) return false;
  }
  auto staged = registry;
  for (size_t i = 0; i < 2; ++i) {
    if (!staged.acknowledge(identity.textures[i], true, std::move(receipts[i]))) return false;
  }
  registry = std::move(staged);
  return true;
}

// The real State/Job use this admission routine. Keeping the transaction here
// also lets its allocation/ownership boundary execute without an Apple SDK.
// A factory returns an untransferred lease; no destructor releases the caller's
// snapshot until all allocation, gate checks and receipt preparation succeed.
template <class State, class Job, class Factory>
bool EnqueueMetalTileGroup(const std::shared_ptr<State>& first,
                          const std::shared_ptr<State>& second,
                          const std::shared_ptr<Job>& a,
                          const std::shared_ptr<Job>& b, Factory makeLease) {
  if (!first || !second || first == second || !a || !b || !(a->stamp == b->stamp)) return false;
  const auto ids = rhwp::tiles::unique_snapshots({a->ids, b->ids});
  std::shared_ptr<Job> oldFirst, oldSecond;
  {
    std::scoped_lock lock(first->mutex, second->mutex);
    if (first->closed || second->closed || !first->ready || !second->ready ||
        !first->api || first->api != second->api || first->generation != second->generation) return false;
    auto firstGate = first->gate, secondGate = second->gate;
    const auto firstSerial = firstGate.accept(a->stamp), secondSerial = secondGate.accept(b->stamp);
    if (!firstSerial || !secondSerial || firstSerial != secondSerial) return false;
    auto firstLeases = first->leases, secondLeases = second->leases;
    for (auto* leases : {&firstLeases, &secondLeases}) {
      for (auto it = leases->begin(); it != leases->end();) {
        if (it->second.expired()) it = leases->erase(it); else ++it;
      }
    }
    // Four snapshots per accepted batch, with bounded pending/in-flight and
    // displayed jobs. This accounting counts distinct IDs, never weak entries
    // repeatedly appended by a borrowed-snapshot redraw.
    for (const auto id : ids) {
      const auto left = firstLeases.find(id), right = secondLeases.find(id);
      const auto l = left == firstLeases.end() ? nullptr : left->second.lock();
      const auto r = right == secondLeases.end() ? nullptr : right->second.lock();
      if (l && r && l != r) throw std::invalid_argument("two destructive tile leases");
      auto lease = l ? l : r;
      if (!lease) lease = makeLease(id, first->api);
      firstLeases[id] = lease;
      secondLeases[id] = lease;
      a->snapshots.push_back(lease);
      b->snapshots.push_back(std::move(lease));
    }
    if (firstLeases.size() > 32 || secondLeases.size() > 32) return false;
    auto firstReceipt = ids, secondReceipt = ids;
    a->sequence = firstSerial;
    b->sequence = secondSerial;
    // Everything below is allocation-free and performed while both owners are
    // locked. Status cannot observe a stamp without the complete ID receipt.
    for (const auto& lease : a->snapshots) lease->transferred = true;
    first->gate = firstGate;
    second->gate = secondGate;
    first->acceptedStamp = a->stamp;
    second->acceptedStamp = b->stamp;
    first->acceptedSnapshots = std::move(firstReceipt);
    second->acceptedSnapshots = std::move(secondReceipt);
    first->leases = std::move(firstLeases);
    second->leases = std::move(secondLeases);
    oldFirst = std::move(first->pending);
    oldSecond = std::move(second->pending);
    first->pending = a;
    second->pending = b;
    first->error.clear();
    second->error.clear();
  }
  first->changed.notify_one();
  second->changed.notify_one();
  return true;
}

}  // namespace rhwp::metal
#endif
