// Synthetic queue/lifetime contracts, not GPU or compositor evidence.
#include "producer_ready.h"
#include <cassert>
#include <thread>
#include <iostream>
int main() {
  rhwp::ProducerReadyMailbox queue;
  rhwp::ProducerReadyFrame frame{1, 7, 1, 10, 2, 3, 4, 1.25};
  assert(!queue.offer(frame));
  queue.listen(true);
  assert(queue.offer(frame));
  assert(!queue.offer(frame));
  frame.serial = 3;
  assert(!queue.offer(frame));
  frame.serial = 2;
  assert(!queue.offer(frame));
  auto second = frame; second.texture = 2;
  assert(!queue.offer(second));
  auto third = frame; third.texture = 3;
  assert(!queue.offer(third));
  auto values = queue.take();
  assert(values.size() == 2 && values[0].serial == 3 && values[1].texture == 2);
  assert(values[0].document == 10 && values[0].scale == 1.25);
  assert(!queue.offer(frame));
  frame.device = 8; frame.serial = 1;
  assert(queue.offer(frame));
  queue.listen(false);
  assert(queue.take().empty() && !queue.offer(frame));
  queue.listen(true);
  assert(queue.offer(frame));
  queue.take();
  auto produce = [&](uint64_t id) {
    auto next = frame; next.texture = id;
    for (uint64_t serial = 2; serial <= 1000; ++serial) {
      next.serial = serial; queue.offer(next);
    }
  };
  std::thread a(produce, 1), b(produce, 2); a.join(); b.join();
  values = queue.take();
  assert(values.size() == 2 && values[0].serial == 1000 && values[1].serial == 1000);
  std::cout << "producer-ready bounded queue contracts passed\n";
}
