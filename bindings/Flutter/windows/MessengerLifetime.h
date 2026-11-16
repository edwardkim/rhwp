#ifndef RHWP_WINDOWS_MESSENGER_LIFETIME_H_
#define RHWP_WINDOWS_MESSENGER_LIFETIME_H_
#include <flutter_windows.h>
#include <memory>
#include <utility>

namespace rhwp::d3d {
// A C++ BinaryMessenger does not keep its engine alive. Retain the public C
// messenger instead, then check engine availability while holding the SDK lock
// around every call through the non-owning C++ channel/sink.
class MessengerLifetime {
 public:
  explicit MessengerLifetime(FlutterDesktopMessengerRef messenger)
      : messenger_(messenger ? FlutterDesktopMessengerAddRef(messenger) : nullptr,
                   &FlutterDesktopMessengerRelease) {}
  MessengerLifetime(const MessengerLifetime&) = delete;
  MessengerLifetime& operator=(const MessengerLifetime&) = delete;

  template <class Callback>
  bool if_available(Callback&& callback) const {
    if (!messenger_) return false;
    std::unique_ptr<FlutterDesktopMessenger, decltype(&FlutterDesktopMessengerUnlock)> lock(
        FlutterDesktopMessengerLock(messenger_.get()), &FlutterDesktopMessengerUnlock);
    if (!lock || !FlutterDesktopMessengerIsAvailable(messenger_.get())) return false;
    std::forward<Callback>(callback)();
    return true;
  }

 private:
  std::unique_ptr<FlutterDesktopMessenger, decltype(&FlutterDesktopMessengerRelease)> messenger_;
};
}  // namespace rhwp::d3d
#endif
