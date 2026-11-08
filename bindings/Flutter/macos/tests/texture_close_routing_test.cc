#include "../Classes/TextureCloseRouting.hpp"
#include "../Classes/MetalViewLifecycle.hpp"
#include "../Classes/MetalFrameLifetime.hpp"
#include <iostream>
#include <stdexcept>

using namespace rhwp::metal;
static void require(bool value) {
  if (!value) throw std::runtime_error("macOS CV close identity/retirement invariant failed");
}
struct Owner { uint64_t texture, generation; bool finished = false; };
struct Receipt { uint64_t texture, generation; };
using Registry = RetirementRegistry<Owner, Receipt>;
static TextureCloseIdentity identity(const std::shared_ptr<Owner>& value) {
  return value ? TextureCloseIdentity{value->texture,value->generation,true} : TextureCloseIdentity{};
}
static TextureCloseRoute route(Registry& registry, uint64_t texture, uint64_t generation,
    TextureCloseIdentity active = {}, TextureCloseIdentity legacy = {}) {
  const auto retained = registry.lookup(generation);
  const auto* completed = registry.receipt(generation);
  bool known = false;
  for (const auto& record : registry.records()) if (record.second.value->texture == texture) known = true;
  for (const auto& receipt : registry.receipts()) if (receipt.second.texture == texture) known = true;
  return ResolveTextureClose(texture,generation,active,identity(retained),
      completed ? TextureCloseIdentity{completed->texture,completed->generation,true} : TextureCloseIdentity{},legacy,known);
}
int main() {
  {
    Registry registry;
    auto first = std::make_shared<Owner>(Owner{7,101}), second = std::make_shared<Owner>(Owner{8,102});
    require(registry.insert(101,first) && registry.insert(102,second));
    require(route(registry,7,0,identity(first)) == TextureCloseRoute::requiresGeneration);
    require(route(registry,7,101,identity(first)) == TextureCloseRoute::negotiated);
    require(route(registry,8,101,identity(second)) == TextureCloseRoute::stale);
    require(route(registry,7,999,identity(first)) == TextureCloseRoute::stale);
    require(registry.close(101) && registry.close(102));
    require(!registry.acknowledge(101,first->finished,Receipt{7,101}));
    require(!registry.insert(103,std::make_shared<Owner>(Owner{7,103})));
    second->finished = true;
    require(registry.acknowledge(102,second->finished,Receipt{8,102}));
    require(route(registry,7,101) == TextureCloseRoute::negotiated && registry.lookup(101) == first);
    require(route(registry,8,102) == TextureCloseRoute::completed);
    // Flutter IDs are object addresses. A new active C may reuse A's address.
    auto newer = std::make_shared<Owner>(Owner{7,103});
    require(registry.insert(103,newer));
    require(route(registry,7,101,identity(newer)) == TextureCloseRoute::negotiated);
    require(registry.lookup(101) == first && registry.registered(103));
    first->finished = true;
    require(registry.acknowledge(101,true,Receipt{7,101}));
    require(route(registry,7,101,identity(newer)) == TextureCloseRoute::completed);
    require(route(registry,8,101) == TextureCloseRoute::stale);
    require(route(registry,7,0,identity(newer)) == TextureCloseRoute::requiresGeneration);
    require(registry.registered(103)); // Late A status/close/ack never selects C.
  }
  {
    Registry registry;
    TextureCloseIdentity active{7,20,false}, old{8,19,false};
    require(route(registry,7,0,active,old) == TextureCloseRoute::legacyActive);
    require(route(registry,7,20,active,old) == TextureCloseRoute::legacyActive);
    require(route(registry,7,19,active,old) == TextureCloseRoute::stale);
    require(route(registry,8,0,{},old) == TextureCloseRoute::legacyClosed);
    require(route(registry,8,19,{},old) == TextureCloseRoute::legacyClosed);
    require(route(registry,9,0,{},old) == TextureCloseRoute::unknown);
    require(ResolveTextureClose(9,0,active,{},{},{},false) == TextureCloseRoute::stale);
  }
  {
    Registry registry;
    auto owner = std::make_shared<Owner>(Owner{7,101,true});
    require(registry.insert(101,owner) && registry.close(101));
    require(registry.acknowledge(101,true,Receipt{7,101}));
    ImmutableFrameCredits credits(4,64*1024*1024,16*1024*1024);
    require(credits.stats().outstanding_frames == 0);
    auto anotherFrame = credits.reserve(16*1024*1024);
    require(anotherFrame != nullptr);
    require(route(registry,7,101) == TextureCloseRoute::completed);
    require(registry.receipt(101)->generation == 101 && credits.stats().outstanding_frames == 1);
    require(credits.stats().reserved_bytes == 16*1024*1024);
    // Completion routing stays terminal while the process-wide budget changes.
  }
  {
    Registry registry;
    ImmutableFrameCredits credits(4,64*1024*1024,16*1024*1024);
    for (int i = 0; i < 4; ++i) {
      auto credit = credits.reserve(16*1024*1024);
      require(credit != nullptr);
      credit->retain_until_process_exit(std::make_shared<int>(i));
    }
    // Repeated retirement frees metadata only; immutable CV resources stay charged.
    for (uint64_t generation = 1; generation <= 400; ++generation) {
      auto owner = std::make_shared<Owner>(Owner{7,generation,true});
      std::weak_ptr<Owner> weak = owner;
      require(registry.insert(generation,owner) && registry.close(generation));
      require(registry.acknowledge(generation,true,Receipt{7,generation}));
      owner.reset();
      require(weak.expired() && registry.size() == 0 && registry.receipt_count() <= 2);
      require(route(registry,7,generation) == TextureCloseRoute::completed);
      require(route(registry,7,0) == TextureCloseRoute::requiresGeneration);
      require(!credits.reserve(1));
    }
    require(!registry.receipt(398) && registry.receipt(399) && registry.receipt(400));
    require(route(registry,7,398) == TextureCloseRoute::stale);
    const auto stats = credits.stats();
    require(stats.outstanding_frames == 4 && stats.reserved_bytes == 64*1024*1024);
  }
  std::cout << "macOS CV retirement: identity routing, concurrent close, compatibility, 400 acknowledgments and immutable credits PASS\n";
}
