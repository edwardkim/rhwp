#include "cache_trim.h"
#include <atomic>
#include <iostream>
#include <stdexcept>
#include <thread>
using rhwp::cache::TrimMailbox;
int checks = 0;
#define CHECK(x) do { ++checks; if (!(x)) throw std::runtime_error("failed: " #x); } while (0)
template<class Callback> void rejected(Callback callback) {
  bool failed = false; try {callback();} catch (const std::invalid_argument&) {failed = true;} CHECK(failed);
}
int main() {
  try {
    TrimMailbox queue;
    rejected([&] {queue.enqueue(0,0);}); rejected([&] {queue.enqueue(1, rhwp::cache::kMaxTrimBytes+1);});
    auto first = queue.enqueue(1,0); CHECK(first.pending && first.busy && queue.queued());
    CHECK(queue.enqueue(1,0).pending); rejected([&] {queue.enqueue(1,1);}); rejected([&] {queue.enqueue(2,0);});
    auto work = queue.take(); CHECK(work && work->request == 1 && !queue.queued() && queue.poll(1).pending);
    CHECK(!queue.take()); CHECK(!queue.finish(2,64,0,64,false));
    rejected([&] {queue.finish(1,64,32,64,false);}); rejected([&] {queue.finish(1,64,32,32,false);});
    rejected([&] {queue.finish(1,64,0,32,false);}); // Fails only the independent byte delta invariant.
    CHECK(queue.finish(1,64,0,64,false)); auto done = queue.poll(1);
    CHECK(!done.pending && !done.busy && done.trimmed == 64 && done.before == 64 && done.after == 0);
    CHECK(!queue.enqueue(1,0).pending); rejected([&] {queue.poll(2);});
    queue.enqueue(2,16); CHECK(queue.take()); CHECK(queue.finish(2,64,32,32,true));
    CHECK(!queue.poll(2).pending && queue.poll(2).busy && queue.poll(2).after == 32);
    queue.enqueue(3,0); CHECK(queue.take()); queue.cancel();
    CHECK(!queue.finish(3,64,0,64,false) && !queue.poll(3).pending && queue.poll(3).busy);
    rejected([&] {queue.enqueue(2,16);}); queue.enqueue(4,0); queue.fail(4);
    CHECK(!queue.queued() && !queue.poll(4).pending && queue.poll(4).busy);
    queue.enqueue(5,0); std::atomic<bool> finished{false};
    std::thread producer([&] {auto next = queue.take(); if(next) queue.finish(next->request,32,0,32,false);finished=true;});
    while (!finished) (void)queue.poll(5);
    producer.join();
    CHECK(!queue.poll(5).pending && !queue.poll(5).busy && queue.poll(5).trimmed == 32);
    std::cout << "cache trim mailbox: " << checks << " checks passed\n"; return 0;
  } catch(const std::exception& error) {std::cerr << error.what() << '\n';return 1;}
}
