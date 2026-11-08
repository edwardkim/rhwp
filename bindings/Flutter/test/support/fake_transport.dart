import 'dart:async';
import 'dart:collection';
import 'dart:typed_data';

import 'package:rhwp_native_reference/rhwp_native.dart';

class FakeTransport implements NativeWorkerTransport {
  @override
  final capabilities = <String, Object?>{
    'worker': true,
    'nativeSkia': true,
    'renderTiledRegion': true,
  };
  final commands = <String>[];
  final completions = Queue<Map<String, Object?>>();
  final released = <int>[];
  final copied = <int>[];
  final opened = <Uint8List>[];
  bool autoComplete = true;
  bool autoRender = true;
  int serial = 0;
  int documentId = 20;
  int revision = 0;
  int fontGeneration = 0;
  int closes = 0;
  Completer<Uint8List>? copyGate;
  Completer<void>? closeGate;

  Map<String, Object?> get info => {
    'documentId': documentId,
    'revision': revision,
    'fontGeneration': fontGeneration,
    'pageCount': 1,
  };

  Map<String, Object?> frame({int? requestId, int? font, int? rev}) => {
    'ok': true,
    'requestId': requestId ?? serial,
    'kind': 'frame',
    'buffer': 42,
    'width': 4,
    'height': 4,
    'stride': 16,
    'byteLength': 64,
    'originX': 0,
    'originY': 0,
    'revision': rev ?? revision,
    'fontGeneration': font ?? fontGeneration,
  };

  @override
  Future<int> createWorker() async => 11;

  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    if (request['op'] == 'workerCancel') {
      commands.add('cancel');
      return {'ok': true, 'outcome': 'cancelled'};
    }
    if (request['op'] == 'workerOpen') {
      commands.add('open');
      opened.add(Uint8List.fromList(data!));
      ++documentId;
      final id = ++serial;
      if (autoComplete) {
        completions.add({
          'ok': true,
          'requestId': id,
          'kind': 'info',
          'info': info,
        });
      }
      return {'ok': true, 'requestId': id};
    }
    final command = request['command']! as Map;
    final op = command['op']! as String;
    commands.add(op);
    final id = ++serial;
    if (op == 'new') ++documentId;
    if (autoComplete) {
      if (op == 'new' || op == 'info') {
        completions.add({
          'ok': true,
          'requestId': id,
          'kind': 'info',
          'info': info,
        });
      } else if (op == 'pageSize') {
        completions.add({
          'ok': true,
          'requestId': id,
          'kind': 'pageSize',
          'size': {'width': 4.0, 'height': 4.0},
        });
      } else if (autoRender &&
          (op == 'renderRegion' || op == 'renderTiledRegion')) {
        completions.add(frame(requestId: id));
      }
    }
    return {'ok': true, 'requestId': id};
  }

  @override
  Future<Map<String, Object?>?> pollWorker(int worker) async =>
      completions.isEmpty ? null : completions.removeFirst();

  @override
  Future<Uint8List> copyAndReleaseBuffer(int buffer, int byteLength) async {
    copied.add(buffer);
    try {
      return copyGate == null
          ? Uint8List.fromList(
              List.generate(byteLength, (index) => index % 4 == 3 ? 255 : 0),
            )
          : await copyGate!.future;
    } finally {
      released.add(buffer);
    }
  }

  @override
  Future<void> releaseBuffer(int buffer) async {
    released.add(buffer);
  }

  @override
  Future<void> close() async {
    ++closes;
    await closeGate?.future;
  }
}

Future<void> waitFor(bool Function() condition) async {
  final deadline = DateTime.now().add(const Duration(seconds: 5));
  while (!condition()) {
    if (DateTime.now().isAfter(deadline)) {
      throw TimeoutException('Expected native test state was not reached.');
    }
    await Future<void>.delayed(const Duration(milliseconds: 1));
  }
}
