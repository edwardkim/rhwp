#include "include/rhwp_native_reference/rhwp_texture_plugin.h"
#include "gl_state_guard.h"
#include "texture_state.h"
#include "rhwp_native_ffi.h"

#include <dlfcn.h>
#include <cstdlib>
#include <cstring>
#include <map>
#include <utility>

namespace {
using namespace rhwp_flutter;
constexpr size_t kMaxTextures = 2;
constexpr size_t kMaxNativeJson = 1024 * 1024;
constexpr char kChannel[] = "rhwp_native/texture_linux";

struct Value {
  FlValue* value;
  explicit Value(FlValue* value) : value(value) {}
  ~Value() { if (value) fl_value_unref(value); }
  Value(const Value&) = delete;
  Value& operator=(const Value&) = delete;
};

FlValue* field(FlValue* map, const char* key) {
  if (!map || fl_value_get_type(map) != FL_VALUE_TYPE_MAP)
    throw Failure("invalidArgument", "expected a method argument map");
  auto* value = fl_value_lookup_string(map, key);
  if (!value) throw Failure("invalidArgument", std::string("missing field: ") + key);
  return value;
}

uint64_t number(FlValue* value) {
  if (!value || fl_value_get_type(value) != FL_VALUE_TYPE_INT || fl_value_get_int(value) < 0)
    throw Failure("invalidArgument", "expected a nonnegative signed-64-bit integer");
  return uint64_t(fl_value_get_int(value));
}
uint64_t integer(FlValue* map, const char* key) { return number(field(map, key)); }
uint32_t integer32(FlValue* map, const char* key) {
  auto value = integer(map, key);
  if (value > UINT32_MAX) throw Failure("invalidArgument", "integer exceeds u32 range");
  return uint32_t(value);
}
double real(FlValue* map, const char* key) {
  auto* value = field(map, key);
  if (fl_value_get_type(value) == FL_VALUE_TYPE_FLOAT) return fl_value_get_float(value);
  if (fl_value_get_type(value) == FL_VALUE_TYPE_INT) return double(fl_value_get_int(value));
  throw Failure("invalidArgument", "expected a numeric value");
}
bool boolean(FlValue* map, const char* key) {
  auto* value = field(map, key);
  return fl_value_get_type(value) == FL_VALUE_TYPE_BOOL && fl_value_get_bool(value);
}
std::string string(FlValue* map, const char* key, size_t limit = 4096) {
  auto* value = field(map, key);
  if (fl_value_get_type(value) != FL_VALUE_TYPE_STRING) throw Failure("invalidArgument", "expected a string");
  const char* text = fl_value_get_string(value);
  if (std::strlen(text) > limit) throw Failure("limitExceeded", "string exceeds its bound");
  return text;
}
void set_int(FlValue* map, const char* key, uint64_t value) {
  fl_value_set_string_take(map, key, fl_value_new_int(int64_t(value)));
}
void set_bool(FlValue* map, const char* key, bool value) {
  fl_value_set_string_take(map, key, fl_value_new_bool(value));
}
void set_text(FlValue* map, const char* key, const std::string& value) {
  fl_value_set_string_take(map, key, fl_value_new_string(value.c_str()));
}

FlValue* stamp_value(const Stamp& stamp) {
  auto* map = fl_value_new_map();
  set_int(map, "documentId", stamp.document);
  set_int(map, "revision", stamp.revision);
  set_int(map, "fontGeneration", stamp.font);
  set_int(map, "viewGeneration", stamp.view);
  set_int(map, "deviceGeneration", stamp.device);
  fl_value_set_string_take(map, "deviceScale", fl_value_new_float(stamp.dpr));
  return map;
}

// Native strings are freed on the same calling thread. This codec only sees
// bounded ABI metadata; it never parses document bytes or calls a legacy HOST.
FlValue* decode_native(char* raw, void (*free_string)(char*)) {
  if (!raw) throw Failure("nativeError", "native returned a null result");
  const size_t length = strnlen(raw, kMaxNativeJson + 1);
  if (length > kMaxNativeJson) { free_string(raw); throw Failure("limitExceeded", "native result is too large"); }
  GBytes* bytes = g_bytes_new(raw, length);
  free_string(raw);
  auto* codec = fl_json_message_codec_new();
  GError* error = nullptr;
  FlValue* value = fl_message_codec_decode_message(FL_MESSAGE_CODEC(codec), bytes, &error);
  g_object_unref(codec);
  g_bytes_unref(bytes);
  if (!value) {
    std::string message = error ? error->message : "invalid native JSON";
    g_clear_error(&error);
    throw Failure("nativeError", message);
  }
  Value guard(value);
  if (!boolean(value, "ok")) throw Failure(string(value, "code", 48), string(value, "error", 4096));
  return fl_value_ref(value);
}

std::vector<uint8_t> encode(FlValue* value) {
  auto* codec = fl_json_message_codec_new();
  GError* error = nullptr;
  GBytes* bytes = fl_message_codec_encode_message(FL_MESSAGE_CODEC(codec), value, &error);
  g_object_unref(codec);
  if (!bytes) {
    std::string message = error ? error->message : "JSON encode failed";
    g_clear_error(&error);
    throw Failure("nativeError", message);
  }
  gsize length;
  const auto* data = static_cast<const uint8_t*>(g_bytes_get_data(bytes, &length));
  std::vector<uint8_t> result(data, data + length);
  g_bytes_unref(bytes);
  return result;
}

class NativeBridge final : public Bridge {
 public:
  using Create = decltype(&rhwp_gl_presenter_create);
  using Request = decltype(&rhwp_gl_presenter_request);
  using Presentation = decltype(&rhwp_presentation_request);
  using Free = decltype(&rhwp_string_free);
  void* library;
  Create create_fn;
  Request request_fn;
  Presentation presentation_fn;
  Free free_fn;
  std::string path;
  std::atomic<uint64_t> snapshot_release_errors{0};

  NativeBridge(void* library, std::string path) : library(library), path(std::move(path)) {
    load(create_fn, "rhwp_gl_presenter_create");
    load(request_fn, "rhwp_gl_presenter_request");
    load(presentation_fn, "rhwp_presentation_request");
    load(free_fn, "rhwp_string_free");
    decltype(&rhwp_session_capabilities) capabilities;
    load(capabilities, "rhwp_session_capabilities");
    Value caps(decode_native(capabilities(), free_fn));
    if (!boolean(field(caps.value, "gpuGl"), "hostFboRendering") ||
        !boolean(field(caps.value, "presentation"), "workerRenderSnapshots")) {
      throw Failure("featureDisabled", "native-skia-gl and worker presentation snapshots are required");
    }
  }
  template <typename Function> void load(Function& function, const char* name) {
    void* address = dlsym(library, name);
    static_assert(sizeof(function) == sizeof(address), "unsupported Linux loader ABI");
    std::memcpy(&function, &address, sizeof(function));
    if (!function) throw Failure("unsupported", std::string("missing native symbol: ") + name);
  }
  FlValue* request(uint64_t presenter, FlValue* request) {
    auto bytes = encode(request);
    return decode_native(request_fn(presenter, bytes.data(), bytes.size()), free_fn);
  }
  SnapshotInfo inspect(uint64_t snapshot) override {
    Value request(fl_value_new_map());
    set_text(request.value, "op", "snapshotInfo");
    set_int(request.value, "snapshot", snapshot);
    auto bytes = encode(request.value);
    Value response(decode_native(presentation_fn(bytes.data(), bytes.size()), free_fn));
    auto* info = field(response.value, "info");
    return {integer(info, "documentId"), integer(info, "revision"), integer(info, "fontGeneration"),
            boolean(info, "current"), string(info, "profile", 16) == "screen"};
  }
  void release_snapshot(uint64_t snapshot) noexcept override {
    try {
      Value request(fl_value_new_map());
      set_text(request.value, "op", "snapshotRelease");
      set_int(request.value, "snapshot", snapshot);
      auto bytes = encode(request.value);
      Value response(decode_native(presentation_fn(bytes.data(), bytes.size()), free_fn));
    } catch (...) {
      // Keep a bounded visible failure counter; never call this successful
      // cleanup merely because the client-side lease object was destroyed.
      snapshot_release_errors.fetch_add(1);
    }
  }
  uint64_t snapshot_release_failures() const noexcept override { return snapshot_release_errors.load(); }
  uint64_t create(uint64_t context, uint64_t generation, Callbacks callbacks) override {
    constexpr char config[] = "{\"maxPixels\":4194304,\"maxPages\":8,\"maxTargetBytes\":16777216,"
        "\"maxScratchBytes\":33554432,\"resourceCacheBytes\":16777216}";
    Value response(decode_native(create_fn(context, generation, callbacks.load, callbacks.current,
        callbacks.user, reinterpret_cast<const uint8_t*>(config), sizeof(config) - 1), free_fn));
    return integer(response.value, "presenter");
  }
  uint64_t render(uint64_t presenter, uint64_t context, const Description& frame, const Target& target) override {
    Value command(fl_value_new_map());
    set_text(command.value, "op", "render");
    set_int(command.value, "clearColor", frame.clear);
    fl_value_set_string_take(command.value, "stamp", stamp_value(frame.stamp));
    auto* output = fl_value_new_map();
    set_int(output, "contextId", context);
    set_int(output, "deviceGeneration", frame.stamp.device);
    set_int(output, "fbo", target.fbo);
    set_int(output, "width", target.width);
    set_int(output, "height", target.height);
    set_int(output, "sampleCount", 0);
    set_int(output, "stencilBits", 0);
    set_text(output, "format", "rgba8");
    // Flutter publicly borrows this texture with a TopLeft image origin.
    set_text(output, "origin", "topLeft");
    fl_value_set_string_take(command.value, "target", output);
    auto* pages = fl_value_new_list();
    for (const auto& page : frame.pages) {
      auto* item = fl_value_new_map();
      set_int(item, "snapshot", page.snapshot);
      set_int(item, "backgroundColor", page.background);
      fl_value_set_string_take(item, "scale", fl_value_new_float(page.scale));
      fl_value_set_string_take(item, "originX", fl_value_new_float(page.x));
      fl_value_set_string_take(item, "originY", fl_value_new_float(page.y));
      fl_value_append_take(pages, item);
    }
    fl_value_set_string_take(command.value, "placements", pages);
    Value result(request(presenter, command.value));
    return integer(field(result.value, "info"), "wholeFrameReadbacks");
  }
  void release(uint64_t presenter, uint64_t context, uint64_t generation) override {
    Value command(fl_value_new_map());
    set_text(command.value, "op", "release");
    set_int(command.value, "contextId", context);
    set_int(command.value, "deviceGeneration", generation);
    Value result(request(presenter, command.value));
  }
  void abandon(uint64_t presenter) override {
    Value command(fl_value_new_map());
    set_text(command.value, "op", "abandon");
    Value result(request(presenter, command.value));
  }
};

std::shared_ptr<NativeBridge> open_native(const std::string& input) {
  if (input.empty() || input[0] != '/') throw Failure("invalidArgument", "libraryPath must be absolute");
  char* resolved = realpath(input.c_str(), nullptr);
  if (!resolved) throw Failure("invalidArgument", "native library path could not be resolved");
  std::string path(resolved);
  std::free(resolved);
  // Process-resident, one canonical native module, matching the P97 owner
  // dispatcher's dlopen. TLS destructors and deferred callbacks stay mapped.
  static auto* mutex = new std::mutex;
  static auto* pinned = new std::shared_ptr<NativeBridge>;
  std::lock_guard<std::mutex> lock(*mutex);
  if (*pinned) {
    if ((*pinned)->path != path) throw Failure("invalidArgument", "one native module path is allowed per process");
    return *pinned;
  }
  void* library = dlopen(path.c_str(), RTLD_NOW | RTLD_LOCAL);
  if (!library) throw Failure("unsupported", "could not load native library");
  try { *pinned = std::make_shared<NativeBridge>(library, path); }
  catch (...) { dlclose(library); throw; }
  return *pinned;
}

class LinuxContext final : public Context {
 public:
  using Current = void* (*)();
  using Proc = void* (*)(const char*);
  GdkGLContext* gdk;
  Current get_current;
  Proc get_proc;
  void* native;
  std::thread::id owner = std::this_thread::get_id();

  LinuxContext() : gdk(gdk_gl_context_get_current()), get_current(nullptr), get_proc(nullptr), native(nullptr) {
    struct Resolver { void* library; const char* current; const char* proc; };
    // Handles intentionally remain resident while GL procedures may be used by
    // any deferred raster owner. EGL supports Wayland and X11; GLX is the other
    // public Linux desktop GL context path. No private Flutter API is involved.
    static Resolver resolvers[] = {{dlopen("libEGL.so.1", RTLD_NOW | RTLD_LOCAL), "eglGetCurrentContext", "eglGetProcAddress"},
        {dlopen("libGLX.so.0", RTLD_NOW | RTLD_LOCAL), "glXGetCurrentContext", "glXGetProcAddressARB"}};
    for (const auto& resolver : resolvers) {
      if (!resolver.library) continue;
      void* current = dlsym(resolver.library, resolver.current);
      void* proc = dlsym(resolver.library, resolver.proc);
      std::memcpy(&get_current, &current, sizeof(current));
      std::memcpy(&get_proc, &proc, sizeof(proc));
      if (get_current && get_proc && (native = get_current())) break;
    }
    if (!native) throw Failure("unsupported", "no current EGL or GLX context");
    const bool desktop = epoxy_is_desktop_gl();
    const int version = epoxy_gl_version();
    if ((desktop && version < 33) || (!desktop && version < 30))
      throw Failure("unsupported", "desktop OpenGL 3.3 or OpenGL ES 3 current context is required");
    // Flutter creates EGL contexts directly, so no GdkGLContext is registered
    // in its raster-thread TLS. The native current-context identity and owner
    // thread remain authoritative; a GDK context is an optional retained owner.
    if (gdk) g_object_ref(gdk);
  }
  ~LinuxContext() override { if (gdk) g_object_unref(gdk); }
  bool same_current() const noexcept override {
    return owner == std::this_thread::get_id() && get_current() == native &&
           (!gdk || gdk_gl_context_get_current() == gdk);
  }
  bool same_context(const Context& other) const noexcept override {
    const auto* context = dynamic_cast<const LinuxContext*>(&other);
    return context && context->gdk == gdk && context->native == native && context->get_current == get_current;
  }
  DriverInfo driver_info() const override {
    if (!same_current()) return {};
    auto text = [](GLenum key) {
      const auto* value = reinterpret_cast<const char*>(glGetString(key));
      return value ? std::string(value, strnlen(value, 256)) : std::string();
    };
    return {text(GL_VENDOR), text(GL_RENDERER), text(GL_VERSION)};
  }
  const void* procedure(const char* name) noexcept override {
    return same_current() ? get_proc(name) : nullptr;
  }
  Target allocate(uint32_t width, uint32_t height) override {
    Target target{0, 0, width, height};
    glActiveTexture(GL_TEXTURE0);
    glBindBuffer(GL_PIXEL_UNPACK_BUFFER, 0);
    glGenTextures(1, &target.texture);
    glBindTexture(GL_TEXTURE_2D, target.texture);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_BASE_LEVEL, 0);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAX_LEVEL, 0);
    glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, GLsizei(width), GLsizei(height), 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
    glGenFramebuffers(1, &target.fbo);
    glBindFramebuffer(GL_FRAMEBUFFER, target.fbo);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, target.texture, 0);
    if (!target.texture || !target.fbo || glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE) {
      destroy(target);
      throw Failure("unsupported", "GL texture framebuffer allocation failed");
    }
    return target;
  }
  void destroy(Target& target) noexcept override {
    if (same_current()) {
      if (target.fbo) {
        glDeleteFramebuffers(1, &target.fbo);
        if (!glIsFramebuffer(target.fbo)) GlStateGuard::forget_deleted_framebuffer(target.fbo);
      }
      if (target.texture) {
        glDeleteTextures(1, &target.texture);
        if (!glIsTexture(target.texture)) GlStateGuard::forget_deleted_texture(target.texture);
      }
    }
    target = {};
  }
};

struct Notice {
  std::shared_ptr<Shared> state;
  GMainContext* main_context;
  GWeakRef texture, registrar;
  std::atomic<bool> queued{false}, registered{true};
  Notice(std::shared_ptr<Shared> state, FlTextureRegistrar* registrar, GObject* texture)
      : state(std::move(state)), main_context(g_main_context_ref_thread_default()) {
    g_weak_ref_init(&this->texture, texture);
    g_weak_ref_init(&this->registrar, registrar);
  }
  ~Notice() { g_weak_ref_clear(&texture); g_weak_ref_clear(&registrar); g_main_context_unref(main_context); }
};
struct Notices { std::mutex mutex; std::map<Shared*, std::weak_ptr<Notice>> entries; };
Notices& notice_state() {
  static auto* state = new Notices;  // May be read by late raster TLS notification.
  return *state;
}

void notify(const std::shared_ptr<Shared>& state) noexcept {
  try {
  std::shared_ptr<Notice> notice;
  {
    auto& notices = notice_state();
    std::lock_guard<std::mutex> lock(notices.mutex);
    auto found = notices.entries.find(state.get());
    if (found != notices.entries.end()) notice = found->second.lock();
  }
  if (!notice || notice->queued.exchange(true)) return;
  std::shared_ptr<Notice>* kept;
  try { kept = new std::shared_ptr<Notice>(notice); }
  catch (...) { notice->queued = false; return; }
  // invoke_full may run synchronously on a thread that can acquire the context.
  // An attached idle source always dispatches from GTK's main-context loop.
  GSource* source = g_idle_source_new();
  g_source_set_priority(source, G_PRIORITY_DEFAULT);
  g_source_set_callback(source, [](gpointer data) -> gboolean {
    auto notice = *static_cast<std::shared_ptr<Notice>*>(data);
    notice->queued = false;
    auto* texture = FL_TEXTURE(g_weak_ref_get(&notice->texture));
    auto* registrar = FL_TEXTURE_REGISTRAR(g_weak_ref_get(&notice->registrar));
    if (texture && registrar && notice->registered) {
      if (notice->state->status().cleanup_complete) {
        if (fl_texture_registrar_unregister_texture(registrar, texture)) notice->registered = false;
        else notice->state->error("unregister", "Flutter did not confirm texture unregistration");
      } else {
        fl_texture_registrar_mark_texture_frame_available(registrar, texture);
      }
    }
    if (registrar) g_object_unref(registrar);
    if (texture) g_object_unref(texture);
    return G_SOURCE_REMOVE;
  }, kept, [](gpointer data) { delete static_cast<std::shared_ptr<Notice>*>(data); });
  if (!g_source_attach(source, notice->main_context)) notice->queued = false;
  g_source_unref(source);
  } catch (...) { /* Accepted snapshot ownership never becomes a rejected RPC. */ }
}

}  // namespace

typedef struct _RhwpTexture { FlTextureGL parent_instance; std::shared_ptr<Notice>* notice; } RhwpTexture;
typedef struct _RhwpTextureClass { FlTextureGLClass parent_class; } RhwpTextureClass;
G_DEFINE_TYPE(RhwpTexture, rhwp_texture, fl_texture_gl_get_type())

static gboolean populate(FlTextureGL* texture, uint32_t* target, uint32_t* name,
                         uint32_t* width, uint32_t* height, GError** error) {
  auto* self = reinterpret_cast<RhwpTexture*>(texture);
  auto state = (*self->notice)->state;
  try {
    // One owner OS thread per Flutter raster context. This registry pins native
    // module/callback/context state even if plugin disposal stops populate calls.
    static thread_local RasterRegistry registry;
    auto context = std::make_shared<LinuxContext>();
    GlStateGuard restore;
    const auto frame = registry.populate(state, std::move(context));
    *target = GL_TEXTURE_2D; *name = frame.texture; *width = frame.width; *height = frame.height;
    return TRUE;
  } catch (const Failure& failure) {
    // A newly registered Texture may be painted before the owner captures its
    // first snapshot. Warmup is not a persistent renderer/device failure.
    if (error) g_clear_error(error);
    if (failure.code != "closing" && failure.code != "notReady")
      state->error(failure.code, failure.what());
    // Flutter 3.47's engine dereferences error->message on every FALSE result,
    // including benign warmup/close. Never return FALSE with a null GError.
    g_set_error_literal(error, g_quark_from_static_string("rhwp-texture"), 1, failure.what());
  } catch (const std::exception& failure) {
    state->error("nativeError", failure.what());
    if (error) g_clear_error(error);
    g_set_error_literal(error, g_quark_from_static_string("rhwp-texture"), 1, failure.what());
  }
  return FALSE;
}

static void rhwp_texture_dispose(GObject* object) {
  auto* self = reinterpret_cast<RhwpTexture*>(object);
  if (self->notice) (*self->notice)->state->close();
  G_OBJECT_CLASS(rhwp_texture_parent_class)->dispose(object);
}
static void rhwp_texture_finalize(GObject* object) {
  auto* self = reinterpret_cast<RhwpTexture*>(object);
  if (self->notice) {
    auto& notices = notice_state();
    std::lock_guard<std::mutex> lock(notices.mutex);
    notices.entries.erase((*self->notice)->state.get());
    delete self->notice;
  }
  G_OBJECT_CLASS(rhwp_texture_parent_class)->finalize(object);
}
static void rhwp_texture_class_init(RhwpTextureClass* klass) {
  FL_TEXTURE_GL_CLASS(klass)->populate = populate;
  G_OBJECT_CLASS(klass)->dispose = rhwp_texture_dispose;
  G_OBJECT_CLASS(klass)->finalize = rhwp_texture_finalize;
}
static void rhwp_texture_init(RhwpTexture* self) { self->notice = nullptr; }

namespace {
struct Plugin {
  FlTextureRegistrar* registrar;
  std::map<int64_t, RhwpTexture*> textures;
  explicit Plugin(FlTextureRegistrar* registrar) : registrar(FL_TEXTURE_REGISTRAR(g_object_ref(registrar))) {}
  ~Plugin() {
    for (const auto& item : textures) {
      auto notice = *item.second->notice;
      notice->state->close();
      // No UI-thread wait or foreign-thread native handle access. A slot that
      // can no longer be sampled remains visible as pendingRetire until TLS.
      if (notice->registered.exchange(false)) fl_texture_registrar_unregister_texture(registrar, FL_TEXTURE(item.second));
      g_object_unref(item.second);
    }
    g_object_unref(registrar);
  }
  void prune() {
    for (auto it = textures.begin(); it != textures.end();) {
      auto notice = *it->second->notice;
      if (notice->state->status().cleanup_complete && !notice->registered) {
        g_object_unref(it->second);
        it = textures.erase(it);
      } else ++it;
    }
  }
  RhwpTexture* find(FlValue* arguments) {
    auto found = textures.find(int64_t(integer(arguments, "textureId")));
    if (found == textures.end()) throw Failure("invalidArgument", "unknown texture ID");
    return found->second;
  }
};

FlValue* status_value(const std::shared_ptr<Notice>& notice) {
  auto status = notice->state->status();
  auto* value = fl_value_new_map();
  set_int(value, "deviceGeneration", status.device);
  set_int(value, "acceptedSerial", status.accepted);
  set_int(value, "renderedSerial", status.rendered);
  set_int(value, "framesSubmitted", status.frames);
  set_int(value, "wholeFrameReadbacks", status.whole_frame_readbacks);
  set_int(value, "abandonedTargets", status.abandoned_targets);
  set_int(value, "orphanedOwners", status.orphaned_owners);
  set_int(value, "liveRasterSlots", RasterRegistry::live_slots());
  set_int(value, "pendingRetireSlots", RasterRegistry::pending_retire());
  const auto orphaned = RasterRegistry::orphaned_slots();
  set_int(value, "orphanedRasterSlots", orphaned);
  set_int(value, "rasterSlotLimit", kMaxRasterSlots);
  const auto release_failures = notice->state->bridge->snapshot_release_failures();
  set_int(value, "snapshotReleaseFailures", release_failures);
  set_text(value, "glVendor", status.driver.vendor);
  set_text(value, "glRenderer", status.driver.renderer);
  set_text(value, "glVersion", status.driver.version);
  set_bool(value, "closing", status.closing);
  set_bool(value, "cleanupComplete", status.cleanup_complete && !release_failures && !status.orphaned_owners);
  set_bool(value, "ownerThreadExit", status.owner_thread_exit);
  set_bool(value, "registered", notice->registered);
  set_text(value, "code", status.code);
  set_text(value, "message", status.message);
  if (status.accepted) fl_value_set_string_take(value, "acceptedStamp", stamp_value(status.accepted_stamp));
  if (status.rendered) fl_value_set_string_take(value, "renderedStamp", stamp_value(status.rendered_stamp));
  return value;
}

FlValue* invoke(Plugin& plugin, const std::string& method, FlValue* args) {
  if (method == "create") {
    plugin.prune();
    if (plugin.textures.size() >= kMaxTextures || RasterRegistry::live_slots() >= kMaxRasterSlots)
      throw Failure("retireLimit", "texture/raster slots are full; finish pending retire first");
    auto bridge = open_native(string(args, "libraryPath"));
    auto state = std::make_shared<Shared>(bridge);
    state->notify = notify;
    auto* texture = reinterpret_cast<RhwpTexture*>(g_object_new(rhwp_texture_get_type(), nullptr));
    std::unique_ptr<GObject, decltype(&g_object_unref)> allocated(G_OBJECT(texture), g_object_unref);
    auto notice = std::make_shared<Notice>(state, plugin.registrar, G_OBJECT(texture));
    texture->notice = new std::shared_ptr<Notice>(notice);
    if (!fl_texture_registrar_register_texture(plugin.registrar, FL_TEXTURE(texture))) {
      notice->registered = false;
      throw Failure("unsupported", "Flutter texture registration failed");
    }
    const auto id = fl_texture_get_id(FL_TEXTURE(texture));
    bool inserted = false;
    try {
      if (id <= 0) throw Failure("nativeError", "Flutter returned an invalid texture ID");
      inserted = plugin.textures.emplace(id, texture).second;
      if (!inserted) throw Failure("nativeError", "Flutter returned a duplicate texture ID");
      {
        auto& notices = notice_state();
        std::lock_guard<std::mutex> lock(notices.mutex);
        notices.entries.emplace(state.get(), notice);
      }
    } catch (...) {
      if (inserted) plugin.textures.erase(id);
      fl_texture_registrar_unregister_texture(plugin.registrar, FL_TEXTURE(texture));
      notice->registered = false;
      throw;
    }
    allocated.release();  // Plugin map now owns the initial GObject reference.
    auto* value = status_value(notice);
    set_int(value, "textureId", uint64_t(id));
    return value;
  }
  auto notice = *plugin.find(args)->notice;
  if (method == "status") return status_value(notice);
  if (method == "close") {
    notice->state->close();
    notify(notice->state);
    return status_value(notice);
  }
  if (method == "submit") {
    auto* stamp = field(args, "stamp");
    Description frame;
    frame.stamp = {integer(stamp, "documentId"), integer(stamp, "revision"), integer(stamp, "fontGeneration"),
                   integer(stamp, "viewGeneration"), integer(stamp, "deviceGeneration"), real(stamp, "deviceScale")};
    frame.width = integer32(args, "width"); frame.height = integer32(args, "height");
    frame.clear = integer32(args, "clearColor");
    auto* pages = field(args, "placements");
    if (fl_value_get_type(pages) != FL_VALUE_TYPE_LIST || fl_value_get_length(pages) > kMaxPages)
      throw Failure("limitExceeded", "placements must be a list of at most eight pages");
    for (size_t index = 0; index < fl_value_get_length(pages); ++index) {
      auto* page = fl_value_get_list_value(pages, index);
      frame.pages.push_back({integer(page, "snapshot"), real(page, "scale"), real(page, "originX"),
                            real(page, "originY"), integer32(page, "backgroundColor")});
    }
    const auto serial = notice->state->submit(std::move(frame));
    notify(notice->state);
    auto* value = status_value(notice);
    set_bool(value, "accepted", true);
    set_int(value, "acceptedSerial", serial);
    return value;
  }
  throw Failure("notImplemented", "unknown Linux texture method");
}
}  // namespace

void rhwp_texture_plugin_register_with_registrar(FlPluginRegistrar* registrar) {
  // Keep this plugin's callback code mapped through raster TLS teardown too.
  Dl_info self;
  if (dladdr(reinterpret_cast<void*>(rhwp_texture_plugin_register_with_registrar), &self) && self.dli_fname)
    (void)dlopen(self.dli_fname, RTLD_NOW | RTLD_LOCAL | RTLD_NODELETE);
  auto* codec = fl_standard_method_codec_new();
  auto* channel = fl_method_channel_new(fl_plugin_registrar_get_messenger(registrar), kChannel, FL_METHOD_CODEC(codec));
  auto* plugin = new Plugin(fl_plugin_registrar_get_texture_registrar(registrar));
  fl_method_channel_set_method_call_handler(channel, [](FlMethodChannel*, FlMethodCall* call, gpointer user) {
    auto* plugin = static_cast<Plugin*>(user);
    FlMethodResponse* response;
    try {
      Value result(invoke(*plugin, fl_method_call_get_name(call), fl_method_call_get_args(call)));
      response = FL_METHOD_RESPONSE(fl_method_success_response_new(result.value));
    } catch (const Failure& failure) {
      response = FL_METHOD_RESPONSE(fl_method_error_response_new(failure.code.c_str(), failure.what(), nullptr));
    } catch (const std::exception& failure) {
      response = FL_METHOD_RESPONSE(fl_method_error_response_new("nativeError", failure.what(), nullptr));
    }
    fl_method_call_respond(call, response, nullptr);
    g_object_unref(response);
  }, plugin, [](gpointer data) { delete static_cast<Plugin*>(data); });
  g_object_unref(channel);
  g_object_unref(codec);
}
