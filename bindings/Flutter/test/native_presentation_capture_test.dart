import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native.dart';

import 'support/fake_transport.dart';

class CaptureTransport extends FakeTransport {
  CaptureTransport() {
    capabilities['presentation'] = {'workerRenderSnapshots': true};
    capabilities['ownerPresentationCleanup'] = true;
  }
  Map<String, Object?>? stamp;
  bool pollStarted = false;
  Completer<Map<String, Object?>?>? gate;
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    if (request['op'] == 'presentationConfigure') {
      stamp = Map<String, Object?>.from(request['stamp']! as Map);
      return {'ok': true, 'source': 9};
    }
    final command = request['command'];
    if (command is Map && command['op'] == 'exportRenderSnapshot') {
      commands.add('exportRenderSnapshot');
      return {'ok': true, 'requestId': ++serial};
    }
    return super.hostRequest(request, data: data);
  }

  Map<String, Object?> get snapshot => {
    'ok': true,
    'requestId': serial,
    'kind': 'renderSnapshot',
    'snapshot': 77,
    'documentId': documentId,
    'info': {'current': true},
  };
  @override
  Future<Map<String, Object?>?> pollWorker(int worker) async {
    if (gate != null) {
      pollStarted = true;
      return gate!.future;
    }
    return super.pollWorker(worker);
  }
}

void main() {
  test('capture configures exact source stamp before snapshot and transfers one handle', () async {
    final transport = CaptureTransport();
    final document = await NativeDocument.connect(transport);
    addTearDown(document.dispose);
    await document.newDocument();
    final releases = <int>[];
    final generation = document.beginPresentation();
    final capture = document.capturePresentation(
      page: 0,
      viewGeneration: generation,
      deviceGeneration: 3,
      deviceScale: 1.5,
      releaseSnapshot: (snapshot) async {
        releases.add(snapshot);
      },
    );
    await waitFor(() => transport.commands.last == 'exportRenderSnapshot');
    expect(transport.stamp, {
      ...document.info!.target,
      'fontGeneration': 0,
      'viewGeneration': generation,
      'deviceGeneration': 3,
      'deviceScale': 1.5,
    });
    transport.completions.add(transport.snapshot);
    final result = await capture;
    expect(result['snapshot'], 77);
    expect(result['stamp'], transport.stamp);
    expect(releases, isEmpty);
  });
  test('a late capture from an older GUI presentation releases instead of displaying', () async {
    final transport = CaptureTransport();
    final document = await NativeDocument.connect(transport);
    addTearDown(document.dispose);
    await document.newDocument();
    final releases = <int>[];
    final capture = document.capturePresentation(
      page: 0,
      viewGeneration: document.beginPresentation(),
      deviceGeneration: 1,
      deviceScale: 1,
      releaseSnapshot: (snapshot) async {
        releases.add(snapshot);
      },
    );
    final error = expectLater(
      capture,
      throwsA(
        isA<NativeCallException>().having(
          (e) => e.code,
          'code',
          'stalePresentation',
        ),
      ),
    );
    await waitFor(() => transport.commands.last == 'exportRenderSnapshot');
    document.beginPresentation();
    transport.completions.add(transport.snapshot);
    await error;
    expect(releases, [77]);
  });
  test('poll begun before dispose releases a late transferred snapshot even after pending is cleared', () async {
    final transport = CaptureTransport();
    final document = await NativeDocument.connect(transport);
    await document.newDocument();
    final releases = <int>[];
    transport.gate = Completer<Map<String, Object?>?>();
    final capture = document.capturePresentation(
      page: 0,
      viewGeneration: document.beginPresentation(),
      deviceGeneration: 1,
      deviceScale: 1,
      releaseSnapshot: (snapshot) async {
        releases.add(snapshot);
      },
    );
    final error = expectLater(capture, throwsA(isA<NativeDisposedException>()));
    await waitFor(() => transport.pollStarted);
    await document.dispose();
    transport.gate!.complete(transport.snapshot);
    await error;
    await waitFor(() => releases.isNotEmpty);
    expect(releases, [77]);
  });
  test('raster invalidation fires after accepted edit is queued and before native receipt', () async {
    final transport = CaptureTransport();
    final document = await NativeDocument.connect(transport);
    addTearDown(document.dispose);
    await document.newDocument();
    final seen = <int>[];
    final subscription = document.rasterInvalidations.listen(seen.add);
    addTearDown(subscription.cancel);
    final edit = document.request('replace', {'text': 'a'});
    final error = expectLater(edit, throwsA(isA<NativeDisposedException>()));
    expect(seen, [document.rasterInvalidationGeneration]);
    expect(transport.commands.last, 'replace');
    await document.dispose();
    await error;
  });
}
