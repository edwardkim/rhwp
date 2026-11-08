#include "../Classes/MetalViewLifecycle.hpp"
#include "../Classes/MetalFrameLifetime.hpp"
#include <limits>

#include <iostream>
#include <stdexcept>

static void require(bool condition) {
  if (!condition) throw std::runtime_error("CAMetalLayer owner lifecycle invariant failed");
}
int main() {
  using rhwp::metal::DrawableLifecycle;
  // More than the legacy four immutable CV frames; one borrowed drawable at a
  // time and no callback that fabricates Core Animation consumer completion.
  {
    DrawableLifecycle life;
    for (uint64_t serial = 1; serial <= 400; ++serial) {
      require(life.acquire(serial));
      require(!life.acquire(serial + 1));
      require(!life.handoff(serial));
      require(life.producer_complete(serial));
      require(!life.producer_complete(serial));
      require(life.handoff(serial));
      require(!life.handoff(serial));
      require(!life.busy());
    }
    require(life.scheduled() == 400 && !life.finished());
    life.close();
    require(life.finish_owner());
    require(!life.acquire(401));
  }
  {
    DrawableLifecycle life;
    require(life.acquire(10));
    life.close();
    require(!life.finish_owner());  // Closing while Native owns GPU work waits.
    require(!life.handoff(10));
    require(life.producer_complete(10));
    require(!life.handoff(10));     // Never present a frame after close.
    require(life.discard_completed(10));
    require(life.finish_owner());
  }
  {
    DrawableLifecycle life;
    require(life.acquire(1));
    life.unknown_failure(1);
    require(life.quarantined());
    require(!life.producer_complete(1));
    require(!life.discard_unsubmitted(1));
    require(!life.discard_completed(1));
    require(!life.finish_owner());
    require(!life.acquire(2));
    life.close();
    require(!life.finish_owner());  // Close/unregister is not GPU evidence.
  }
  {
    DrawableLifecycle life;
    require(!life.acquire(0));
    require(life.acquire(4));
    require(!life.producer_complete(3));
    require(!life.discard_unsubmitted(3));
    require(life.discard_unsubmitted(4));
    require(!life.acquire(4));       // Late ticket cannot bind to newer work.
    require(life.acquire(5));
    require(life.producer_complete(5));
    require(!life.discard_unsubmitted(5));
    require(life.discard_completed(5));
    require(life.discarded() == 2 && life.scheduled() == 0);
    life.close();
    require(life.finish_owner());
  }
  {
    using rhwp::metal::DrawableSize;
    const auto scaled = DrawableSize::FromBacking(327.25, 239.01);
    require(scaled && scaled->width == 328 && scaled->height == 240);
    require(scaled->matches(328, 240) && !scaled->matches(327, 240));
    require(!DrawableSize::FromBacking(4096.01, 1));
    require(!DrawableSize::FromBacking(4096, 1025));
    require(DrawableSize::FromBacking(4096, 1024).has_value());
    require(!DrawableSize::FromBacking(0, 24));
    require(!DrawableSize::FromBacking(-1, 24));
    require(!DrawableSize::FromBacking(std::numeric_limits<double>::infinity(), 24));
    require(!DrawableSize::FromBacking(32, std::numeric_limits<double>::quiet_NaN()));
  }
  {
    // An accepted resize/new source supersedes in-flight pixels. Their Native
    // producer can complete, but they must be discarded before presentation.
    rhwp::metal::GenerationGate gate(7);
    rhwp::metal::Stamp before{1, 3, 0, 1, 7, 1};
    DrawableLifecycle life;
    const auto old = gate.accept(before);
    require(life.acquire(old));
    auto latest = before;
    for (uint64_t i = 2; i <= 30; ++i) {
      latest.view = i;
      require(gate.accept(latest) != 0);
    }
    require(life.producer_complete(old));
    require(!gate.current(old, before));
    require(life.discard_completed(old));
    const auto next = gate.accept(latest);
    require(life.acquire(next) && life.producer_complete(next));
    require(gate.current(next, latest));
    require(life.handoff(next) && life.scheduled() == 1);
    gate.close();
    life.close();
    require(!gate.current(next, latest) && life.finish_owner());
  }
  {
    rhwp::metal::VisibilityGate visibility;
    require(visibility.covered());
    require(visibility.apply(1, true, false));
    require(visibility.apply(2, false, true) && !visibility.covered());
    visibility.invalidate();  // New accepted work/stale source covers immediately.
    require(visibility.covered() && visibility.serial() == 2);
    require(!visibility.apply(2, false, true) && visibility.covered());
    require(visibility.apply(3, false, true) && !visibility.covered());
    require(visibility.apply(4, true, false) && visibility.covered());
    require(!visibility.apply(5, false, false) && visibility.covered());
    require(visibility.apply(6, false, true) && !visibility.covered());
    require(!visibility.apply(3, true, false) && !visibility.covered());
    visibility.close();
    require(visibility.covered() && !visibility.apply(7, false, true));
  }
  {
    // Two legitimate concurrent closes must remain independently observable.
    struct Owner { bool finished = false; };
    using Registry = rhwp::metal::RetirementRegistry<Owner, uint64_t>;
    Registry registry;
    auto first = std::make_shared<Owner>(), second = std::make_shared<Owner>();
    std::weak_ptr<Owner> firstWeak = first;
    require(registry.insert(1, first) && registry.insert(2, second));
    require(registry.close(1) && registry.close(2));
    require(!registry.registered(1) && !registry.registered(2));
    require(registry.lookup(1) == first && registry.lookup(2) == second);
    require(!registry.acknowledge(1, first->finished, 1));
    first->finished = true;
    require(!registry.insert(3, std::make_shared<Owner>())); // Finished alone is no acknowledgment.
    require(registry.lookup(1)->finished && !registry.lookup(2)->finished);
    require(registry.acknowledge(1, first->finished, 1));
    first.reset();
    require(firstWeak.expired() && registry.receipt(1) && *registry.receipt(1) == 1);
    require(registry.insert(3, std::make_shared<Owner>()));
    require(registry.lookup(2) == second && !registry.acknowledge(2, false, 2));
    require(registry.acknowledge(1, false, 99)); // Repeated acknowledgment replays its terminal proof.
    require(*registry.receipt(1) == 1);
    require(!registry.acknowledge(3, true, 3)); // An active view cannot be retired.
  }
  {
    // Completion order differs from handle order. Preserve the two most recent
    // acknowledgments, even when a much older producer finally exits.
    rhwp::metal::RetirementRegistry<int, uint64_t> registry;
    require(registry.insert(1, std::make_shared<int>(1)));
    require(registry.insert(2, std::make_shared<int>(2)));
    require(registry.close(1) && registry.close(2));
    require(registry.acknowledge(2, true, 2));
    require(registry.insert(3, std::make_shared<int>(3)) && registry.close(3));
    require(registry.acknowledge(3, true, 3));
    require(registry.lookup(1) != nullptr && registry.receipt_count() == 2);
    require(registry.acknowledge(1, true, 1));
    require(registry.receipt(1) && registry.receipt(3) && !registry.receipt(2));
    require(!registry.lookup(1) && registry.size() == 0 && registry.receipt_count() == 2);
    require(!registry.acknowledge(2, false, 2)); // Expired acknowledged receipt is explicit.
  }
  {
    // Quarantine cannot be acknowledged, replaced, or hidden by a newer close.
    rhwp::metal::RetirementRegistry<DrawableLifecycle, uint64_t> registry;
    auto first = std::make_shared<DrawableLifecycle>(), second = std::make_shared<DrawableLifecycle>();
    require(first->acquire(1) && second->acquire(1));
    first->unknown_failure(1);
    require(registry.insert(1, first) && registry.insert(2, second));
    require(registry.close(1) && registry.close(2));
    require(!registry.acknowledge(1, first->finish_owner(), 1));
    require(!registry.acknowledge(2, second->finish_owner(), 2));
    require(!registry.insert(3, std::make_shared<DrawableLifecycle>()));
    require(registry.lookup(1)->quarantined() && registry.lookup(2)->busy());
  }
  {
    rhwp::metal::RetirementRegistry<int, uint64_t> registry;
    for (uint64_t handle = 1; handle <= 400; ++handle) {
      auto owner = std::make_shared<int>(1);
      std::weak_ptr<int> weak = owner;
      require(registry.insert(handle, owner) && registry.close(handle));
      require(registry.acknowledge(handle, true, handle));
      owner.reset();
      require(weak.expired() && registry.size() == 0 && registry.receipt_count() <= 2);
    }
    require(!registry.receipt(398) && registry.receipt(399) && registry.receipt(400));
  }
  std::cout << "CAMetalLayer lifecycle: 11 scenarios passed\n";
}
