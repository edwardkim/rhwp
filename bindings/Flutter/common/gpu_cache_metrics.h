#ifndef RHWP_GPU_CACHE_METRICS_H_
#define RHWP_GPU_CACHE_METRICS_H_

#include <array>
#include <cstddef>
#include <cstdint>
#include <optional>

namespace rhwp::metrics {
constexpr uint64_t kMaxExactCounter = 9007199254740991ULL;
inline constexpr const char* kScope = "publishedFramePostRenderSnapshot";
struct Counter {
  const char* name;
  const char* native_key;
  bool target;
};
inline constexpr std::array<Counter, 10> kCounters{{
    {"cacheHits", "hits", false}, {"cacheMisses", "misses", false},
    {"cacheEvictions", "evictions", false}, {"cachedTiles", "cachedTiles", false},
    {"cachedBytes", "cachedBytes", false}, {"managedTargets", "targets", true},
    {"managedBytes", "estimatedBytes", true}, {"canonicalReplays", "canonicalReplays", false},
    {"viewportFallbacks", "viewportFallbacks", false},
    {"targetAdmissionFallbacks", "targetAdmissionFallbacks", false},
}};
struct CacheMetrics {
  std::array<std::optional<uint64_t>, kCounters.size()> values{};
  bool empty() const noexcept {
    for (const auto& value : values) if (value) return false;
    return true;
  }
};

// The platform adapter reads actual Native info. Missing, malformed or unsafe
// integers stay unknown instead of turning into a fabricated zero measurement.
// Diagnostics must never throw into frame publication or GPU resource cleanup.
template <typename Read> CacheMetrics capture(Read read) noexcept {
  CacheMetrics result;
  for (std::size_t index = 0; index < kCounters.size(); ++index) {
    try {
      auto value = read(kCounters[index].target, kCounters[index].native_key);
      if (value && *value <= kMaxExactCounter) result.values[index] = *value;
    } catch (...) {}
  }
  return result;
}
}  // namespace rhwp::metrics
#endif
