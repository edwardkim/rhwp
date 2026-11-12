#ifndef RHWP_WINDOWS_PRODUCER_READY_EVENTS_H_
#define RHWP_WINDOWS_PRODUCER_READY_EVENTS_H_
#include "../common/producer_ready.h"
#include <flutter/event_channel.h>
#include <flutter/event_stream_handler_functions.h>
#include <flutter/standard_method_codec.h>
#include <windows.h>
#include <atomic>
#include <memory>

namespace rhwp::d3d {
// The producer never sends through the platform messenger. A private message-
// only window drains hints on the thread which registered this Flutter plugin.
class ProducerReadyEvents : public std::enable_shared_from_this<ProducerReadyEvents> {
  using Value = flutter::EncodableValue;
  using Map = flutter::EncodableMap;
  static constexpr UINT kReady = WM_APP + 0x524;
 public:
  explicit ProducerReadyEvents(flutter::BinaryMessenger* messenger) {
    WNDCLASSW cls{};
    cls.lpfnWndProc = procedure; cls.hInstance = GetModuleHandleW(nullptr);
    cls.lpszClassName = L"RhwpProducerReadyEvents";
    if (!RegisterClassW(&cls) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) return;
    window_ = CreateWindowExW(0, cls.lpszClassName, L"", 0, 0, 0, 0, 0,
                             HWND_MESSAGE, nullptr, cls.hInstance, this);
    if (!window_.load()) return; // Polling remains available on this host.
    try {
    channel_ = std::make_unique<flutter::EventChannel<Value>>(messenger,
        "rhwp_native/texture_windows/events", &flutter::StandardMethodCodec::GetInstance());
    channel_->SetStreamHandler(std::make_unique<flutter::StreamHandlerFunctions<Value>>(
        [this](const Value*, std::unique_ptr<flutter::EventSink<Value>>&& sink)
            -> std::unique_ptr<flutter::StreamHandlerError<Value>> {
          sink_ = std::move(sink); mailbox_.listen(true); return nullptr;
        }, [this](const Value*) -> std::unique_ptr<flutter::StreamHandlerError<Value>> {
          mailbox_.listen(false); sink_.reset(); return nullptr;
        }));
    } catch (...) {
      // A constructor failure does not run our destructor. Do not leave a live
      // HWND whose GWLP_USERDATA points at this partially constructed object.
      stop();
      throw;
    }
  }
  bool available() const { return window_.load() != nullptr; }
  void stop() {
    mailbox_.listen(false);
    if (channel_) channel_->SetStreamHandler(nullptr);
    channel_.reset();
    sink_.reset();
    if (auto window = window_.exchange(nullptr)) DestroyWindow(window);
  }
  ~ProducerReadyEvents() { stop(); }
  void offer(const ProducerReadyFrame& frame) noexcept {
    try {
      if (!mailbox_.offer(frame)) return;
      const auto window = window_.load();
      if (!window || !PostMessageW(window, kReady, 0, 0)) mailbox_.listen(false);
    } catch (...) { mailbox_.listen(false); }
  }
 private:
  static LRESULT CALLBACK procedure(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    if (message == WM_NCCREATE) {
      auto* created = reinterpret_cast<CREATESTRUCTW*>(lparam);
      SetWindowLongPtrW(window, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(created->lpCreateParams));
    }
    auto* self = reinterpret_cast<ProducerReadyEvents*>(GetWindowLongPtrW(window, GWLP_USERDATA));
    if (message == kReady && self) { self->drain(); return 0; }
    if (message == WM_NCDESTROY) SetWindowLongPtrW(window, GWLP_USERDATA, 0);
    return DefWindowProcW(window, message, wparam, lparam);
  }
  void drain() noexcept {
    try {
      for (const auto& frame : mailbox_.take()) {
        if (!sink_) break;
        auto number = [](uint64_t value) { return Value(int64_t(value)); };
        Map stamp{{Value("documentId"), number(frame.document)}, {Value("revision"), number(frame.revision)},
            {Value("fontGeneration"), number(frame.font)}, {Value("viewGeneration"), number(frame.view)},
            {Value("deviceGeneration"), number(frame.device)}, {Value("deviceScale"), Value(frame.scale)}};
        sink_->Success(Value(Map{{Value("event"), Value("producerReady")},
            {Value("textureId"), number(frame.texture)}, {Value("deviceGeneration"), number(frame.device)},
            {Value("serial"), number(frame.serial)}, {Value("stamp"), Value(stamp)}}));
      }
    } catch (...) { /* Event delivery never changes renderer ownership. */ }
  }
  std::atomic<HWND> window_{nullptr};
  ProducerReadyMailbox mailbox_;
  std::unique_ptr<flutter::EventChannel<Value>> channel_;
  std::unique_ptr<flutter::EventSink<Value>> sink_;
};
}  // namespace rhwp::d3d
#endif
