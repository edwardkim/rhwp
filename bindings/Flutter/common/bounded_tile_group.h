#ifndef RHWP_BOUNDED_TILE_GROUP_H_
#define RHWP_BOUNDED_TILE_GROUP_H_
#include <algorithm>
#include <array>
#include <cstdint>
#include <memory>
#include <stdexcept>
#include <vector>

namespace rhwp::tiles {
constexpr size_t kMaxTiles = 2, kMaxSnapshots = 4;
// A batch owns each opaque Native snapshot exactly once. The two jobs retain
// copies of the same shared_ptrs, including a page which crosses the tile seam.
// Callers allocate every job/receipt before marking these leases transferred.
inline std::vector<uint64_t> unique_snapshots(const std::array<std::vector<uint64_t>, 2>& pages) {
  std::vector<uint64_t> result;
  for (const auto& tile : pages) for (const auto id : tile) {
    if (!id) throw std::invalid_argument("zero tile snapshot");
    if (std::find(result.begin(), result.end(), id) == result.end()) {
      if (result.size() == kMaxSnapshots) throw std::invalid_argument("a tile group retains at most four unique snapshots");
      result.push_back(id);
    }
  }
  if (result.empty()) throw std::invalid_argument("empty tile group");
  std::sort(result.begin(), result.end());
  return result;
}
template <class Lease, class Factory>
std::vector<std::shared_ptr<Lease>> prepare_leases(const std::vector<uint64_t>& ids, Factory factory) {
  std::vector<std::shared_ptr<Lease>> result;
  result.reserve(ids.size());
  for (const auto id : ids) result.push_back(factory(id));
  return result;
}
struct Identity {
  uint64_t generation = 0;
  std::array<uint64_t, 2> textures{};
  uint64_t id() const { return textures[0]; }
  bool valid() const { return generation && textures[0] && textures[1] && textures[0] != textures[1]; }
  bool matches(uint64_t group) const { return valid() && id() == group; }
};
}  // namespace rhwp::tiles
#endif
