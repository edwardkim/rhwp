#ifndef RHWP_TEST_FLUTTER_WINDOWS_H_
#define RHWP_TEST_FLUTTER_WINDOWS_H_
// Minimal C API boundary for MessengerLifetime's portable tests only. This
// models SDK ownership/locking/availability and does not validate a real SDK.
struct FlutterDesktopMessenger;
using FlutterDesktopMessengerRef = FlutterDesktopMessenger*;
extern "C" {
FlutterDesktopMessengerRef FlutterDesktopMessengerAddRef(FlutterDesktopMessengerRef messenger);
void FlutterDesktopMessengerRelease(FlutterDesktopMessengerRef messenger);
FlutterDesktopMessengerRef FlutterDesktopMessengerLock(FlutterDesktopMessengerRef messenger);
void FlutterDesktopMessengerUnlock(FlutterDesktopMessengerRef messenger);
bool FlutterDesktopMessengerIsAvailable(FlutterDesktopMessengerRef messenger);
}
#endif
