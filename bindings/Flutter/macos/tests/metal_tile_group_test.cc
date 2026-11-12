// Portable production admission/visibility/retirement contracts, not pixels.
#include "../Classes/MetalTileGroup.hpp"
#include <condition_variable>
#include <iostream>
#include <stdexcept>
#include <string>

namespace {
size_t checks = 0;
void require(bool value) {
  ++checks;
  if (!value) throw std::runtime_error("Metal tile group check " + std::to_string(checks));
}
struct Lease {
  uint64_t id;
  bool transferred = false;
  std::map<uint64_t, int>* releases;
  ~Lease() { if (transferred) ++(*releases)[id]; }
};
struct Job {
  uint64_t sequence = 0;
  rhwp::metal::Stamp stamp;
  std::vector<uint64_t> ids;
  std::vector<std::shared_ptr<Lease>> snapshots;
};
struct Owner {
  explicit Owner(std::shared_ptr<int> api) : api(std::move(api)), gate(7) {}
  std::mutex mutex;
  std::condition_variable changed;
  std::shared_ptr<int> api;
  rhwp::metal::GenerationGate gate;
  uint64_t generation = 7;
  bool closed = false, ready = true;
  std::map<uint64_t, std::weak_ptr<Lease>> leases;
  rhwp::metal::Stamp acceptedStamp;
  std::vector<uint64_t> acceptedSnapshots;
  std::shared_ptr<Job> pending;
  std::string error;
};
struct Fixture {
  std::map<uint64_t, int> releases;
  std::shared_ptr<int> api = std::make_shared<int>(1);
  std::shared_ptr<Owner> a = std::make_shared<Owner>(api), b = std::make_shared<Owner>(api);
  rhwp::metal::Stamp stamp{1, 0, 2, 1, 7, 1.25};
  int failAt = 0, allocations = 0;
  std::shared_ptr<Job> job(std::vector<uint64_t> ids) {
    auto value = std::make_shared<Job>();value->ids = std::move(ids);value->stamp = stamp;return value;
  }
  bool enqueue(const std::shared_ptr<Job>& x, const std::shared_ptr<Job>& y) {
    return rhwp::metal::EnqueueMetalTileGroup(a, b, x, y, [this](uint64_t id, const auto& passedApi) {
      require(passedApi == api);
      if (++allocations == failAt) throw std::bad_alloc();
      return std::make_shared<Lease>(Lease{id, false, &releases});
    });
  }
};
struct Receipt {
  static int movesBeforeFailure;
  uint64_t id;
  explicit Receipt(uint64_t id) : id(id) {}
  Receipt(const Receipt&) = default;
  Receipt(Receipt&& other) : id(other.id) {
    if (movesBeforeFailure == 0) throw std::bad_alloc();
    if (movesBeforeFailure > 0) --movesBeforeFailure;
  }
};
int Receipt::movesBeforeFailure = -1;
}  // namespace

int main() {
  using namespace rhwp::metal;
  {
    Fixture f;
    auto a = f.job({10, 11}), b = f.job({11, 12});
    require(f.enqueue(a, b));
    require(a->sequence == 1 && b->sequence == 1);
    require(f.a->acceptedStamp == f.stamp && f.b->acceptedStamp == f.stamp);
    const std::vector<uint64_t> unionIds{10, 11, 12};
    require(f.a->acceptedSnapshots == unionIds && f.b->acceptedSnapshots == unionIds);
    require(a->snapshots == b->snapshots && a->snapshots.size() == 3);
    require(f.allocations == 3 && f.releases.empty());
    f.a->pending.reset();a.reset();
    require(f.releases.empty());
    f.b->pending.reset();b.reset();
    require(f.releases == std::map<uint64_t, int>({{10, 1}, {11, 1}, {12, 1}}));
  }
  {
    Fixture f;
    // The same snapshot can cross the seam and be borrowed for many redraws;
    // weak-map bookkeeping must not create false saturation at redraw five.
    for (uint64_t i = 1; i <= 400; ++i) {
      f.stamp.view = i;
      auto a = f.job({10, 11, 12, 13}), b = f.job({10, 11, 12, 13});
      require(f.enqueue(a, b));
      require(a->sequence == i && b->sequence == i);
      require(f.a->leases.size() == 4 && f.b->leases.size() == 4);
      require(f.releases.empty());
    }
    require(f.allocations == 4);
    f.a->pending.reset();f.b->pending.reset();
    require(f.releases == std::map<uint64_t, int>({{10, 1}, {11, 1}, {12, 1}, {13, 1}}));
  }
  {
    Fixture f;
    auto a = f.job({10}), b = f.job({11});
    f.b->ready = false;
    require(!f.enqueue(a, b));
    require(!f.a->pending && !f.b->pending && !a->sequence && !b->sequence && !f.allocations);
    f.b->ready = true;f.b->closed = true;
    require(!f.enqueue(a, b));
    f.b->closed = false;f.b->api = std::make_shared<int>(2);
    require(!f.enqueue(a, b));
    f.b->api = f.api;b->stamp.revision++;
    require(!f.enqueue(a, b));
    require(f.releases.empty() && f.a->acceptedSnapshots.empty() && f.b->acceptedSnapshots.empty());
  }
  {
    Fixture f;
    require(f.enqueue(f.job({10}), f.job({10})));
    auto previousA = f.a->pending, previousB = f.b->pending;
    f.stamp.view = 0;
    require(!f.enqueue(f.job({11}), f.job({11})));
    require(f.a->pending == previousA && f.b->pending == previousB);
    f.stamp.view = 2;
    f.failAt = f.allocations + 2;
    bool threw = false;
    try { f.enqueue(f.job({11}), f.job({12})); } catch (const std::bad_alloc&) { threw = true; }
    require(threw && f.releases.empty());
    require(f.a->pending == previousA && f.b->pending == previousB);
    require(f.a->acceptedSnapshots == std::vector<uint64_t>{10} && f.b->acceptedStamp.view == 1);
    f.failAt = 0;
    auto a = f.job({11}), b = f.job({12});
    require(f.enqueue(a, b) && a->sequence == 2 && b->sequence == 2);
  }
  {
    Fixture f;
    bool threw = false;
    try {f.enqueue(f.job({10, 11, 12}), f.job({12, 13, 14}));}
    catch (const std::invalid_argument&) {threw = true;}
    require(threw && !f.allocations && f.releases.empty());
    require(!f.a->pending && !f.b->pending);
    require(f.enqueue(f.job({10, 11}), f.job({12, 13})));
  }
  {
    Stamp stamp{1, 0, 0, 1, 7, 1};
    require(TileFramesCurrent(true, stamp, true, stamp));
    require(!TileFramesCurrent(true, stamp, false, stamp));
    for (int field = 0; field < 6; ++field) {
      auto changed = stamp;
      switch (field) {
        case 0: ++changed.document;break;
        case 1: ++changed.revision;break;
        case 2: ++changed.font;break;
        case 3: ++changed.view;break;
        case 4: ++changed.device;break;
        default: changed.device_scale = 2;break;
      }
      require(!TileFramesCurrent(true, stamp, true, changed));
    }
    VisibilityGate a, b;
    require(!ApplyTileCover(a, b, 1, false, false, false).accepted);
    require(a.covered() && b.covered() && a.serial() == 1 && b.serial() == 1);
    require(ApplyTileCover(a, b, 2, false, true, false).accepted);
    require(!a.covered() && !b.covered());
    require(!ApplyTileCover(a, b, 1, true, false, false).changed);
    require(!a.covered() && !b.covered() && a.serial() == 2 && b.serial() == 2);
    require(ApplyTileCover(a, b, 3, true, false, false).accepted);
    require(a.covered() && b.covered());
    require(!ApplyTileCover(a, b, 4, false, true, true).accepted);
    require(a.serial() == 3 && b.serial() == 3);
  }
  {
    RetirementRegistry<int, Receipt> registry;
    require(registry.insert(40, std::make_shared<int>(1)) && registry.insert(41, std::make_shared<int>(2)));
    const rhwp::tiles::Identity identity{7, {40, 41}};
    auto acknowledge = [&](bool complete) {
      return AcknowledgeMetalTileGroup(registry, identity, complete, std::array<Receipt, 2>{Receipt(40), Receipt(41)});
    };
    require(!acknowledge(true)); // Registration is not retirement evidence.
    registry.close(40);registry.close(41);
    require(!acknowledge(false) && registry.size() == 2);
    // A receipt allocation/move failure after staging the first member cannot
    // remove either actual owner. Every throw location leaves both observable.
    for (int failure = 0; failure < 4; ++failure) {
      Receipt::movesBeforeFailure = failure;
      bool threw = false;
      try {acknowledge(true);} catch (const std::bad_alloc&) {threw = true;}
      require(threw && registry.size() == 2 && registry.receipt_count() == 0);
    }
    Receipt::movesBeforeFailure = -1;
    require(acknowledge(true) && registry.size() == 0 && registry.receipt_count() == 2);
    require(registry.receipt(40)->id == 40 && registry.receipt(41)->id == 41);
  }
  std::cout << "Metal tile group production policy: " << checks << " checks passed\n";
}
