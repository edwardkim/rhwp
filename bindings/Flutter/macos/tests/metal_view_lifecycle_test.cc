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
    require(visibility.apply(3, true, false) && visibility.covered());
    require(!visibility.apply(2, false, true) && visibility.covered());
    require(!visibility.apply(4, false, false) && visibility.covered());
    require(visibility.apply(5, false, true) && !visibility.covered());
    require(!visibility.apply(3, true, false) && !visibility.covered());
    visibility.close();
    require(visibility.covered() && !visibility.apply(6, false, true));
  }
  std::cout << "CAMetalLayer lifecycle: 7 scenarios passed\n";
}
