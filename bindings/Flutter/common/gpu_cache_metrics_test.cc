#include "gpu_cache_metrics.h"
#include <cassert>
#include <map>
#include <stdexcept>
#include <string>

int main() {
  using namespace rhwp::metrics;
  size_t reads = 0;
  auto absent = capture([&](bool, const char*) -> std::optional<uint64_t> { ++reads; return {}; });
  assert(absent.empty() && reads == 10);
  auto zero = capture([](bool, const char*) -> std::optional<uint64_t> { return 0; });
  assert(!zero.empty());
  for (const auto& value : zero.values) assert(value && *value == 0);
  auto bounded = capture([](bool, const char*) -> std::optional<uint64_t> { return kMaxExactCounter; });
  for (const auto& value : bounded.values) assert(value && *value == kMaxExactCounter);
  auto oversized = capture([](bool, const char*) -> std::optional<uint64_t> { return kMaxExactCounter + 1; });
  assert(oversized.empty());
  auto throwing = capture([](bool, const char*) -> std::optional<uint64_t> { throw std::runtime_error("missing"); });
  assert(throwing.empty());
  std::map<std::string, uint64_t> tiles{{"hits", 7}, {"misses", 4}, {"cachedBytes", 128}};
  std::map<std::string, uint64_t> targets{{"targets", 3}, {"estimatedBytes", 1024}};
  auto sample = capture([&](bool target, const char* key) -> std::optional<uint64_t> {
    const auto& values = target ? targets : tiles;
    auto found = values.find(key); return found == values.end() ? std::nullopt : std::optional(found->second);
  });
  assert(sample.values[0] == 7 && sample.values[1] == 4);
  assert(!sample.values[2] && sample.values[4] == 128);
  assert(sample.values[5] == 3 && sample.values[6] == 1024);
  assert(std::string(kCounters[0].name) == "cacheHits");
  assert(std::string(kCounters[6].name) == "managedBytes");
}
