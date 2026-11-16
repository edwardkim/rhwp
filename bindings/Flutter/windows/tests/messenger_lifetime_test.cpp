// Executes the production guard with a modeled public C SDK boundary. No real
// Windows/Flutter engine or GPU execution is claimed by this portable test.
#include "MessengerLifetime.h"
#include <atomic>
#include <condition_variable>
#include <future>
#include <iostream>
#include <mutex>
#include <stdexcept>
#include <thread>

namespace {
int checks = 0;
#define CHECK(value) do { ++checks; if (!(value)) throw std::runtime_error("check failed: " #value); } while (0)
}
struct FlutterDesktopMessenger {
  std::recursive_mutex mutex;
  int references = 1, retained = 0, released = 0, locks = 0, unlocks = 0;
  int depth = 0, availability_checks = 0, sends = 0;
  bool available = true;
};
extern "C" FlutterDesktopMessengerRef FlutterDesktopMessengerAddRef(FlutterDesktopMessengerRef messenger) {
  CHECK(messenger && messenger->references > 0);
  ++messenger->references; ++messenger->retained; return messenger;
}
extern "C" void FlutterDesktopMessengerRelease(FlutterDesktopMessengerRef messenger) {
  CHECK(messenger && messenger->references > 0 && messenger->depth == 0);
  --messenger->references; ++messenger->released;
}
extern "C" FlutterDesktopMessengerRef FlutterDesktopMessengerLock(FlutterDesktopMessengerRef messenger) {
  messenger->mutex.lock();
  CHECK(messenger->references > 0);
  ++messenger->depth; ++messenger->locks; return messenger;
}
extern "C" void FlutterDesktopMessengerUnlock(FlutterDesktopMessengerRef messenger) {
  CHECK(messenger->depth > 0);
  --messenger->depth; ++messenger->unlocks; messenger->mutex.unlock();
}
extern "C" bool FlutterDesktopMessengerIsAvailable(FlutterDesktopMessengerRef messenger) {
  CHECK(messenger->depth > 0 && messenger->references > 0);
  ++messenger->availability_checks; return messenger->available;
}
namespace {
using rhwp::d3d::MessengerLifetime;
void send(FlutterDesktopMessenger& messenger) {
  CHECK(messenger.references > 0 && messenger.available && messenger.depth > 0);
  ++messenger.sends;
}
void available_callback_keeps_engine_guarded_and_balances_ownership() {
  FlutterDesktopMessenger messenger;
  {
    MessengerLifetime owner(&messenger);
    CHECK(messenger.references == 2 && messenger.retained == 1);
    CHECK(owner.if_available([&] { send(messenger); }));
    CHECK(messenger.sends == 1 && messenger.depth == 0);
    CHECK(messenger.locks == 1 && messenger.unlocks == 1 && messenger.availability_checks == 1);
  }
  CHECK(messenger.references == 1 && messenger.released == 1);
}
void shutdown_skips_dead_cpp_messenger_after_engine_reference_drops() {
  FlutterDesktopMessenger messenger;
  {
    MessengerLifetime owner(&messenger);
    messenger.available = false;
    FlutterDesktopMessengerRelease(&messenger); // Engine's reference is gone.
    CHECK(messenger.references == 1);
    CHECK(!owner.if_available([&] { send(messenger); }));
    CHECK(!owner.if_available([&] { throw std::runtime_error("dead handler unregister"); }));
    CHECK(messenger.sends == 0 && messenger.depth == 0 && messenger.locks == messenger.unlocks);
  }
  CHECK(messenger.references == 0 && messenger.released == 2);
}
void throwing_callback_unlocks_and_can_be_followed_by_normal_use() {
  FlutterDesktopMessenger messenger;
  {
    MessengerLifetime owner(&messenger);
    bool threw = false;
    try { owner.if_available([&] { throw std::runtime_error("modeled codec allocation failure"); }); }
    catch (const std::runtime_error&) { threw = true; }
    CHECK(threw && messenger.depth == 0 && messenger.locks == messenger.unlocks);
    CHECK(owner.if_available([&] { send(messenger); }));
    CHECK(messenger.sends == 1 && messenger.depth == 0);
  }
  CHECK(messenger.references == 1 && messenger.retained == messenger.released);
}
void plugin_and_event_owner_keep_independent_retained_leases() {
  FlutterDesktopMessenger messenger;
  {
    MessengerLifetime events(&messenger);
    {
      MessengerLifetime plugin(&messenger);
      CHECK(messenger.references == 3);
      CHECK(plugin.if_available([&] { send(messenger); }));
    }
    CHECK(messenger.references == 2);
    CHECK(events.if_available([&] { send(messenger); }));
    messenger.available = false;
    FlutterDesktopMessengerRelease(&messenger);
    CHECK(messenger.references == 1);
    CHECK(!events.if_available([&] { send(messenger); }));
  }
  CHECK(messenger.references == 0 && messenger.retained == 2 && messenger.released == 3);
}
void sdk_shutdown_cannot_invalidate_a_callback_holding_the_lock() {
  FlutterDesktopMessenger messenger;
  MessengerLifetime owner(&messenger);
  std::promise<void> callback_entered, shutdown_requested;
  auto entered = callback_entered.get_future();
  auto requested = shutdown_requested.get_future();
  std::atomic<bool> callback_completed{false}, shutdown_completed{false};
  std::thread shutdown([&] {
    entered.wait(); shutdown_requested.set_value();
    std::lock_guard<std::recursive_mutex> lock(messenger.mutex);
    // Same mutex as the modeled SDK Lock/Unlock pair above.
    messenger.available = false;
    shutdown_completed.store(true);
  });
  CHECK(owner.if_available([&] {
    callback_entered.set_value(); requested.wait();
    CHECK(!shutdown_completed.load());
    send(messenger); callback_completed.store(true);
  }));
  shutdown.join();
  CHECK(callback_completed.load() && shutdown_completed.load());
  CHECK(!owner.if_available([&] { send(messenger); }));
  CHECK(messenger.sends == 1 && messenger.depth == 0);
}
void missing_messenger_never_calls_the_sdk_or_callback() {
  MessengerLifetime missing(nullptr);
  CHECK(!missing.if_available([] { throw std::runtime_error("missing messenger callback"); }));
}
}  // namespace
int main() {
  try {
    available_callback_keeps_engine_guarded_and_balances_ownership();
    shutdown_skips_dead_cpp_messenger_after_engine_reference_drops();
    throwing_callback_unlocks_and_can_be_followed_by_normal_use();
    plugin_and_event_owner_keep_independent_retained_leases();
    sdk_shutdown_cannot_invalidate_a_callback_holding_the_lock();
    missing_messenger_never_calls_the_sdk_or_callback();
    std::cout << "production messenger lifetime: 6 cases, " << checks << " checks passed\n";
    return 0;
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
