#include "bounded_tile_group.h"
#include "../windows/FrameLifetime.h"
#include "../windows/SurfaceImportTickets.h"
#include <iostream>
#include <map>

namespace {
int checks = 0;
#define CHECK(condition) do { ++checks; if (!(condition)) throw std::runtime_error("check failed: " #condition); } while (0)
struct Lease {
  uint64_t id = 0;
  bool transferred = false;
  std::map<uint64_t, int>* releases;
  ~Lease() { if (transferred) ++(*releases)[id]; }
};
void unique_batch_is_owned_until_the_last_tile_job() {
  const auto ids = rhwp::tiles::unique_snapshots({std::vector<uint64_t>{11,12,11}, std::vector<uint64_t>{12,13}});
  CHECK((ids == std::vector<uint64_t>{11,12,13}));
  std::map<uint64_t,int> releases;
  auto leases = rhwp::tiles::prepare_leases<Lease>(ids,[&](uint64_t id) {
    auto lease = std::make_shared<Lease>(); lease->id=id; lease->releases=&releases; return lease;
  });
  auto first = leases, second = leases;
  for (const auto& lease : leases) lease->transferred=true;
  leases.clear(); first.clear();
  CHECK(releases.empty());
  second.clear();
  CHECK(releases.size()==3);
  for (const auto id : ids) CHECK(releases[id]==1);
}
void partial_allocation_failure_does_not_take_any_snapshot() {
  const auto ids = rhwp::tiles::unique_snapshots({std::vector<uint64_t>{21,22}, std::vector<uint64_t>{22,23}});
  std::map<uint64_t,int> releases;
  bool failed=false;
  try {
    (void)rhwp::tiles::prepare_leases<Lease>(ids,[&](uint64_t id) {
      if (id==23) throw std::bad_alloc();
      auto lease = std::make_shared<Lease>(); lease->id=id; lease->releases=&releases; return lease;
    });
  } catch (const std::bad_alloc&) { failed=true; }
  CHECK(failed&&releases.empty());
  for (const auto& pages : std::vector<std::array<std::vector<uint64_t>,2>>{
      {{{}, {}}}, {{{0}, {}}}, {{{1,2,3}, {4,5}}}}) {
    failed=false;
    try {(void)rhwp::tiles::unique_snapshots(pages);} catch(const std::invalid_argument&) {failed=true;}
    CHECK(failed);
  }
}
void two_tiles_keep_the_existing_windows_four_frame_byte_ceiling() {
  using namespace rhwp::d3d;
  CHECK(frame_charge(2050,800)==16*1024*1024);
  CHECK(frame_charge(2048,1536)==0);
  ImmutableFrames frames;
  std::vector<std::shared_ptr<ImmutableFrames::Reservation>> credits;
  for (int index=0; index<4; ++index) {
    auto credit=frames.reserve(frame_charge(2050,800),PublicationMode::ImmutableOwnershipTransfer);
    CHECK(credit); credits.push_back(credit);
  }
  CHECK(frames.stats().outstanding==4&&frames.stats().bytes==64*1024*1024);
  CHECK(!frames.reserve(1,PublicationMode::ImmutableOwnershipTransfer));
  credits.pop_back();
  CHECK(frames.stats().outstanding==3&&frames.stats().bytes==48*1024*1024);
  CHECK(frames.reserve(frame_charge(2050,800),PublicationMode::ImmutableOwnershipTransfer));
  credits.clear(); CHECK(frames.stats().outstanding==0&&frames.stats().bytes==0);
}
struct Descriptor {
  void* handle=nullptr;
  void (*release_callback)(void*)=nullptr;
  void* release_context=nullptr;
};
void same_group_generation_does_not_alias_registrar_import_owners() {
  using Imports=rhwp::d3d::SurfaceImportTickets<Descriptor>;
  const rhwp::tiles::Identity group{71,{101,102}};
  CHECK(group.valid()&&group.matches(101)&&!group.matches(102));
  // Native stamp generation is shared. Each actual registrar still gets its
  // own monotonic import owner; removing tile A must not remove tile B.
  constexpr uint64_t first_owner=72,second_owner=73;
  Imports::register_owner(first_owner); Imports::register_owner(second_owner);
  bool refused=false;
  try {Imports::register_owner(group.generation);} catch(const std::runtime_error&) {refused=true;}
  CHECK(refused);
  {auto first=Imports::begin_callback(first_owner);auto second=Imports::begin_callback(second_owner);CHECK(first&&second);}
  Imports::unregistered(first_owner);
  {auto first=Imports::begin_callback(first_owner);auto second=Imports::begin_callback(second_owner);CHECK(!first&&second);}
  Imports::unregistered(second_owner);
}
}
int main() {
  try {
    unique_batch_is_owned_until_the_last_tile_job();
    partial_allocation_failure_does_not_take_any_snapshot();
    two_tiles_keep_the_existing_windows_four_frame_byte_ceiling();
    same_group_generation_does_not_alias_registrar_import_owners();
    std::cout<<"bounded tile group: 4 cases, "<<checks<<" checks passed\n";return 0;
  } catch(const std::exception& error) {std::cerr<<error.what()<<'\n';return 1;}
}
