#include "../Classes/MetalFrameLifetime.hpp"

#include <atomic>
#include <iostream>
#include <stdexcept>
#include <thread>

static void require(bool value) {
  if (!value) throw std::runtime_error("Metal frame lifetime invariant failed");
}
struct Pixels {
  explicit Pixels(std::atomic<int>& counter) : destroyed(counter) {}
  ~Pixels() { ++destroyed; }
  std::atomic<int>& destroyed;
};

int main() {
  using namespace rhwp::metal;
  {
    ImmutableFrameCredits credits(3, 256, 128);
    require(!credits.reserve(0));
    require(!credits.reserve(std::numeric_limits<size_t>::max()));
    auto first = credits.reserve(128), second = credits.reserve(128);
    require(first && second && !credits.reserve(1));
    first.reset();  // Never exposed: producer can return this reservation.
    require(credits.stats().outstanding_frames == 1);
    require(credits.reserve(128) != nullptr);
    second.reset();
    require(credits.stats().reserved_bytes == 0);
  }
  {
    std::atomic<int> destroyed{0};
    ImmutableFrameCredits credits(2, 128, 64);
    for (int i = 0; i < 2; ++i) {
      auto reservation = credits.reserve(64);
      auto pixels = std::make_shared<Pixels>(destroyed);
      reservation->retain_until_process_exit(pixels);
      reservation->retain_until_process_exit(pixels);  // Idempotent.
      auto consumer_copy = pixels;
      pixels.reset();
      consumer_copy.reset();  // copyPixelBuffer return is not GPU completion.
      reservation.reset();   // Producer close/unregister is not completion.
    }
    require(destroyed == 0);
    require(!credits.reserve(1));
    const auto stats = credits.stats();
    require(stats.outstanding_frames == 2 && stats.retained_pending_retire == 2 && stats.reserved_bytes == 128);
  }
  {
    ImmutableFrameCredits credits(4, 256, 64);
    std::vector<std::thread> threads;
    for (int i = 0; i < 8; ++i) threads.emplace_back([&] {
      for (int j = 0; j < 1000; ++j) {
        auto reservation = credits.reserve(64);
        auto stats = credits.stats();
        require(stats.outstanding_frames <= 4 && stats.reserved_bytes <= 256);
      }
    });
    for (auto& thread : threads) thread.join();
    require(credits.stats().outstanding_frames == 0);
  }
  {
    GenerationGate gate(3);
    Stamp stamp{1, 0, 0, 1, 3, 2};
    const auto old = gate.accept(stamp);
    require(old && gate.current(old, stamp));
    auto newer = stamp;
    newer.revision = 1;
    const auto current = gate.accept(newer);
    require(current && !gate.current(old, stamp) && gate.current(current, newer));
    auto invalid = newer;
    invalid.device = 4;
    require(!gate.accept(invalid) && gate.current(current, newer));
    invalid = newer;
    invalid.device_scale = std::numeric_limits<double>::infinity();
    require(!gate.accept(invalid));
    newer.view = 2;
    const auto latest = gate.accept(newer);
    require(!gate.accept(stamp) && gate.current(latest, newer));
    gate.close();
    require(!gate.current(latest, newer) && !gate.accept(newer));
  }
  std::cout << "Metal portable lifetime: limits, immutable retention, concurrent credit, source/device generations PASS\n";
}
