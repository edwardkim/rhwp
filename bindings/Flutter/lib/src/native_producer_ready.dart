import 'dart:async';

import 'package:flutter/services.dart';

/// One EventChannel listener per messenger/backend, shared by live controllers.
/// Subscription transitions are serialized so closing one owner cannot cancel
/// another owner's newly installed listener. Events only request a status check.
/// Availability means the native listen was acknowledged and no stream failure
/// was observed. Owners must retain pending-work polling and operation deadlines.
class NativeProducerReadyEvents {
  NativeProducerReadyEvents._(this.channel);
  final EventChannel channel;
  static final _hubs = <(BinaryMessenger, String), NativeProducerReadyEvents>{};
  final _listeners = <void Function(Object?), void Function(bool)?>{};
  bool _available = false;
  bool _changing = false, _listening = false, _failed = false;

  static VoidCallback attach(
    MethodChannel methods,
    void Function(Object?) listener, {
    void Function(bool)? onAvailabilityChanged,
  }) {
    final key = (methods.binaryMessenger, '${methods.name}/events');
    final hub = _hubs.putIfAbsent(
      key,
      () => NativeProducerReadyEvents._(
        EventChannel(key.$2, const StandardMethodCodec(), key.$1),
      ),
    );
    hub._listeners[listener] = onAvailabilityChanged;
    onAvailabilityChanged?.call(hub._available);
    unawaited(hub._sync(key));
    return () {
      hub._listeners.remove(listener);
      unawaited(hub._sync(key));
    };
  }

  Future<void> _sync((BinaryMessenger, String) key) async {
    if (_changing) return;
    _changing = true;
    try {
      while (true) {
        if (_listeners.isEmpty && _listening) {
          _listening = false;
          _setAvailable(false);
          channel.binaryMessenger.setMessageHandler(channel.name, null);
          await _control('cancel');
        } else if (_listeners.isNotEmpty && !_listening && !_failed) {
          _listening = true;
          channel.binaryMessenger.setMessageHandler(channel.name, (
            reply,
          ) async {
            if (reply == null) {
              _markFailed();
              return null;
            }
            if (_failed) return null;
            Object? event;
            try {
              event = channel.codec.decodeEnvelope(reply);
            } catch (_) {
              _markFailed();
              return null; // Error/malformed envelopes restore status polling.
            }
            for (final listener in _listeners.keys.toList()) {
              if (_listeners.containsKey(listener)) listener(event);
            }
            return null;
          });
          if (!await _control('listen')) {
            _markFailed();
            _listening = false;
            channel.binaryMessenger.setMessageHandler(channel.name, null);
            await _control('cancel');
          } else if (!_failed) {
            _setAvailable(true);
          }
        } else {
          break;
        }
      }
    } finally {
      _changing = false;
      if (_listeners.isEmpty && !_listening && identical(_hubs[key], this)) {
        _hubs.remove(key);
      }
    }
  }

  void _setAvailable(bool available) {
    if (_available == available) return;
    _available = available;
    for (final listener in _listeners.keys.toList()) {
      if (_listeners.containsKey(listener)) {
        _listeners[listener]?.call(available);
      }
    }
  }

  void _markFailed() {
    _failed = true;
    _setAvailable(false);
  }

  Future<bool> _control(String method) async {
    try {
      // Speak the public EventChannel protocol explicitly: its broadcast stream
      // does not await async onCancel, and this hub must await cancellation before
      // re-listening on the same messenger. Never replace another owner's handler.
      await MethodChannel(
        channel.name,
        channel.codec,
        channel.binaryMessenger,
      ).invokeMethod<void>(method).timeout(const Duration(seconds: 2));
      return true;
    } catch (_) {
      return false; // Optional event transport cannot fail renderer ownership.
    }
  }
}
