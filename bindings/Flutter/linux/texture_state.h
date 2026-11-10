#ifndef RHWP_FLUTTER_LINUX_TEXTURE_STATE_H_
#define RHWP_FLUTTER_LINUX_TEXTURE_STATE_H_

#include <atomic>
#include <cstdint>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <string>
#include <thread>
#include <vector>

namespace rhwp_flutter {

constexpr size_t kMaxPages = 8;
constexpr size_t kMaxRasterSlots = 4;  // Process-wide, including deferred retire.
constexpr uint64_t kMaxPixels = 4 * 1024 * 1024;

struct Failure : std::runtime_error {
  std::string code;
  Failure(std::string code, const std::string& message)
      : std::runtime_error(message), code(std::move(code)) {}
};

struct Stamp {
  uint64_t document = 0, revision = 0, font = 0, view = 0, device = 1;
  double dpr = 1;
};
struct Placement {
  uint64_t snapshot = 0;
  double scale = 1, x = 0, y = 0;
  uint32_t background = 0xffffff;
};
struct Description {
  Stamp stamp;
  uint32_t width = 0, height = 0, clear = 0xff55575a;
  std::vector<Placement> pages;
};
struct SnapshotInfo {
  uint64_t document = 0, revision = 0, font = 0;
  bool current = false, screen = false;
};
struct Target {
  uint32_t texture = 0, fbo = 0, width = 0, height = 0;
};
struct DriverInfo { std::string vendor, renderer, version; };
struct Callbacks {
  const void* (*load)(void*, const char*);
  int32_t (*current)(void*, uint64_t);
  void* user;
};

class Bridge {
 public:
  virtual ~Bridge() = default;
  virtual SnapshotInfo inspect(uint64_t snapshot) = 0;
  virtual void release_snapshot(uint64_t snapshot) noexcept = 0;
  virtual uint64_t snapshot_release_failures() const noexcept { return 0; }
  virtual uint64_t create(uint64_t context, uint64_t generation, Callbacks) = 0;
  virtual uint64_t render(uint64_t presenter, uint64_t context,
                          const Description&, const Target&) = 0;
  virtual void release(uint64_t presenter, uint64_t context, uint64_t generation) = 0;
  virtual void abandon(uint64_t presenter) = 0;
};

// A retained context supplies procedures only to its creating raster OS thread.
// No engine/Skia objects, document handles or legacy TLS calls cross this seam.
class Context {
 public:
  virtual ~Context() = default;
  virtual bool same_current() const noexcept = 0;
  virtual bool same_context(const Context&) const noexcept = 0;
  // Called once when this raster context becomes a retained owner, while it is
  // current. This describes the actual consumer/producer context, not a probe.
  virtual DriverInfo driver_info() const { return {}; }
  virtual const void* procedure(const char*) noexcept = 0;
  virtual Target allocate(uint32_t width, uint32_t height) = 0;
  virtual void destroy(Target&) noexcept = 0;
  // Only a backend that proves all associated driver allocations are gone may
  // return true. Public Linux GL share-group lifetime provides no such proof.
  virtual bool abandon_reclaims_resources() const noexcept { return false; }
};

struct SnapshotLease {
  std::shared_ptr<Bridge> bridge;
  uint64_t id;
  bool owned = false;
  ~SnapshotLease() { if (owned) bridge->release_snapshot(id); }
};
struct Frame {
  Description description;
  uint64_t serial;
  std::vector<std::shared_ptr<SnapshotLease>> leases;
};
struct Status {
  uint64_t device = 1, accepted = 0, rendered = 0, frames = 0;
  uint64_t whole_frame_readbacks = 0, abandoned_targets = 0, orphaned_owners = 0;
  bool closing = false, cleanup_complete = false, raster_owner = false, owner_thread_exit = false;
  std::string code, message;
  Stamp accepted_stamp;
  // Distinct IDs in the accepted transaction, retained through close so a
  // delayed submit reply can be reconciled without guessing lease ownership.
  std::vector<uint64_t> accepted_snapshots;
  Stamp rendered_stamp;
  DriverInfo driver;
};

class Shared : public std::enable_shared_from_this<Shared> {
 public:
  explicit Shared(std::shared_ptr<Bridge> bridge, uint64_t device = 1) : bridge(std::move(bridge)) { status_.device = device; }
  // New IDs transfer only after a successful return. IDs already retained by
  // this texture are borrowed from its existing leases, never released twice.
  uint64_t submit(Description description);
  static void submit_group(const std::shared_ptr<Shared>& first, const std::shared_ptr<Shared>& second,
                           Description first_frame, Description second_frame);
  void close();  // Only metadata/registry release: never waits for GPU work.
  Status status() const;
  void error(const std::string& code, const std::string& message);
  std::shared_ptr<Bridge> bridge;
  // Invoked after dropping all metadata locks. The plugin posts a bounded idle
  // callback; tests can observe completion without depending on a GUI toolkit.
  void (*notify)(const std::shared_ptr<Shared>&) = nullptr;

 private:
  friend class RasterRegistry;
  mutable std::mutex mutex_;
  std::shared_ptr<const Frame> latest_;
  std::vector<std::weak_ptr<SnapshotLease>> leases_;
  Status status_;
};

class RasterRegistry {
 public:
  RasterRegistry();
  ~RasterRegistry();
  RasterRegistry(const RasterRegistry&) = delete;
  RasterRegistry& operator=(const RasterRegistry&) = delete;
  // Must be called exclusively by the same raster OS thread. Returns either a
  // newly committed target or its last-good target after a replay rejection.
  Target populate(const std::shared_ptr<Shared>&, std::shared_ptr<Context>);
  void drain_retired(const std::shared_ptr<Shared>& sampled);
  static size_t live_slots();
  static size_t pending_retire();
  static size_t orphaned_slots();

 private:
  struct Slot;
  std::vector<std::unique_ptr<Slot>> slots_;
  std::thread::id owner_ = std::this_thread::get_id();
  void retire(size_t index, bool thread_exit);
};

}  // namespace rhwp_flutter
#endif
