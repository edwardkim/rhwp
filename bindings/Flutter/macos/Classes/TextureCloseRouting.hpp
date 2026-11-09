#ifndef RHWP_MACOS_TEXTURE_CLOSE_ROUTING_HPP
#define RHWP_MACOS_TEXTURE_CLOSE_ROUTING_HPP

#include <cstdint>

namespace rhwp::metal {
struct TextureCloseIdentity {
  uint64_t texture = 0, generation = 0;
  bool negotiated = false;
  bool matches(uint64_t handle, uint64_t device) const {
    return texture && generation && texture == handle && generation == device;
  }
};
enum class TextureCloseRoute {
  negotiated, completed, legacyActive, legacyClosed, requiresGeneration, stale, unknown
};
// The legacy Flutter texture ID is an object address and may be reused. Resolve
// an explicit generation before consulting the currently active texture with
// that same handle. Missing generations preserve only unnegotiated behavior.
inline TextureCloseRoute ResolveTextureClose(uint64_t texture, uint64_t generation,
    TextureCloseIdentity active, TextureCloseIdentity retained,
    TextureCloseIdentity completed, TextureCloseIdentity legacyClosed,
    bool retainedHandleKnown) {
  if (generation) {
    if (retained.generation) return retained.matches(texture, generation) ?
        TextureCloseRoute::negotiated : TextureCloseRoute::stale;
    if (completed.generation) return completed.matches(texture, generation) ?
        TextureCloseRoute::completed : TextureCloseRoute::stale;
    if (active.generation) return active.matches(texture, generation) && !active.negotiated ?
        TextureCloseRoute::legacyActive : TextureCloseRoute::stale;
    if (legacyClosed.matches(texture, generation)) return TextureCloseRoute::legacyClosed;
    return TextureCloseRoute::stale;
  }
  if (active.generation) {
    if (active.texture != texture) return TextureCloseRoute::stale;
    return active.negotiated ? TextureCloseRoute::requiresGeneration : TextureCloseRoute::legacyActive;
  }
  if (retainedHandleKnown) return TextureCloseRoute::requiresGeneration;
  return legacyClosed.texture == texture && legacyClosed.generation ?
      TextureCloseRoute::legacyClosed : TextureCloseRoute::unknown;
}
}  // namespace rhwp::metal
#endif
