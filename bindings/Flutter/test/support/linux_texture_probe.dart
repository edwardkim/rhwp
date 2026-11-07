// Integration entry point for a disposable Flutter Linux runner. Requires the
// P102 owner dispatcher/client and a GL-enabled native library. It deliberately
// supports the pre-P93 snapshotInfo-only artifact for backend qualification;
// the production NativeTextureController still requires fullStampChecks.
import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_ffi.dart';

typedef _RequestNative = Pointer<Utf8> Function(Pointer<Uint8>, UintPtr);
typedef _RequestDart = Pointer<Utf8> Function(Pointer<Uint8>, int);
typedef _FreeNative = Void Function(Pointer<Utf8>);
typedef _FreeDart = void Function(Pointer<Utf8>);

final class _Metadata {
  _Metadata(String path) {
    final library = DynamicLibrary.open(path);
    request = library.lookupFunction<_RequestNative, _RequestDart>(
      'rhwp_presentation_request',
    );
    free = library.lookupFunction<_FreeNative, _FreeDart>('rhwp_string_free');
  }
  late final _RequestDart request;
  late final _FreeDart free;

  Map<String, Object?> call(String operation, int snapshot, [Object? stamp]) {
    final bytes = utf8.encode(
      jsonEncode({'op': operation, 'snapshot': snapshot, 'stamp': ?stamp}),
    );
    final input = calloc<Uint8>(bytes.length);
    Pointer<Utf8> output = nullptr;
    try {
      input.asTypedList(bytes.length).setAll(0, bytes);
      output = request(input, bytes.length);
      if (output == nullptr) throw StateError('missing metadata response');
      var length = 0;
      while (length <= 1024 * 1024 && output.cast<Uint8>()[length] != 0) {
        length++;
      }
      if (length > 1024 * 1024) throw StateError('unbounded metadata response');
      return Map<String, Object?>.from(
        jsonDecode(output.toDartString(length: length)) as Map,
      );
    } finally {
      if (output != nullptr) free(output);
      calloc.free(input);
    }
  }

  Future<void> release(int snapshot) async {
    if (call('snapshotRelease', snapshot)['ok'] != true) {
      throw StateError('snapshot release failed');
    }
  }
}

void main() => runApp(const MaterialApp(home: _Probe()));

class _Probe extends StatefulWidget {
  const _Probe();
  @override
  State<_Probe> createState() => _ProbeState();
}

class _ProbeState extends State<_Probe> {
  static const channel = MethodChannel('rhwp_native/texture_linux');
  int? texture;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => unawaited(probe()));
  }

  Future<Map<String, Object?>> invoke(
    String op,
    Map<String, Object?> args,
  ) async => Map<String, Object?>.from(
    await channel.invokeMethod<Map<Object?, Object?>>(op, args) ?? {},
  );

  Future<Map<String, Object?>> closeTexture(int id) async {
    await invoke('close', {'textureId': id});
    for (var attempt = 0; attempt < 200; attempt++) {
      final status = await invoke('status', {'textureId': id});
      if (status['registered'] == false && status['cleanupComplete'] == true) {
        return status;
      }
      await Future<void>.delayed(const Duration(milliseconds: 25));
    }
    throw StateError(
      'texture close did not settle on its owning raster context',
    );
  }

  Future<void> probe() async {
    NativeDocument? document;
    _Metadata? metadata;
    int? ownedSnapshot;
    var stage = 'create';
    try {
      final native = await File(
        Platform.environment['RHWP_PROBE_NATIVE_LIBRARY']!,
      ).resolveSymbolicLinks();
      final created = await invoke('create', {'libraryPath': native});
      final id = created['textureId'];
      if (id is! int || id <= 0 || created['registered'] != true) {
        throw StateError('texture registration failed: $created');
      }
      setState(() => texture = id);
      // Exercise an initial populate before any snapshot exists. Warmup must
      // neither latch an error nor prevent the later accepted frame.
      await Future<void>.delayed(const Duration(milliseconds: 200));
      final warmup = await invoke('status', {'textureId': id});
      if (Platform.environment['RHWP_PROBE_EXPECT_UNSUPPORTED'] == '1') {
        if (warmup['code'] != 'unsupported' || warmup['framesSubmitted'] != 0) {
          throw StateError('ES 2 admission did not fail closed: $warmup');
        }
        final closed = await closeTexture(id);
        stdout.writeln(
          'RHWP_LINUX_GPU_PROBE ${jsonEncode({'ok': true, 'expectedUnsupported': true, 'warmup': warmup, 'closed': closed})}',
        );
        exit(0);
      }
      if (warmup['code'] != '') throw StateError('warmup failed: $warmup');
      stage = 'ownerConnect';
      final owner = await NativeOwnerTransport.connect(
        dispatcherLibrary: Platform.environment['RHWP_PROBE_OWNER_LIBRARY']!,
        nativeLibrary: native,
      );
      stage = 'documentConnect';
      document = await NativeDocument.connect(owner);
      stage = 'newDocument';
      await document.newDocument();
      stage = 'beginEdit';
      const position = {
        'section': 0,
        'paragraph': 0,
        'scalarOffset': 0,
        'cellPath': <Object>[],
      };
      await document.request('beginEdit', {
        'selection': {'anchor': position, 'focus': position},
      });
      stage = 'pasteText';
      await document.request('pasteText', {
        'text': 'RHWP EGL / GLES 3 reference',
      });
      metadata = _Metadata(native);
      stage = 'capture';
      final capture = await document.capturePresentation(
        page: 0,
        viewGeneration: document.beginPresentation(),
        deviceGeneration: created['deviceGeneration']! as int,
        deviceScale: 1,
        releaseSnapshot: metadata.release,
      );
      final snapshot = ownedSnapshot = capture['snapshot']! as int;
      final stamp = Map<String, Object?>.from(capture['stamp']! as Map);
      stage = 'snapshotInfo';
      final info = metadata.call('snapshotInfo', snapshot);
      final source = info['info'] as Map?;
      if (info['ok'] != true ||
          source?['current'] != true ||
          ![
            'documentId',
            'revision',
            'fontGeneration',
          ].every((key) => source?[key] == stamp[key])) {
        throw StateError('captured source metadata is stale: $info');
      }
      // After send, transport loss is ambiguous. This disposable probe retains
      // that one possible lease on failure rather than guessing a release.
      ownedSnapshot = null;
      stage = 'submit';
      final accepted = await invoke('submit', {
        'textureId': id,
        'stamp': stamp,
        'width': 320,
        'height': 240,
        'clearColor': 0xff55575a,
        'placements': [
          {
            'snapshot': snapshot,
            'scale': .4,
            'originX': 0,
            'originY': 0,
            'backgroundColor': 0xffffffff,
          },
        ],
      });
      if (accepted['accepted'] != true ||
          accepted['acceptedStamp'] is! Map ||
          !stamp.entries.every(
            (entry) =>
                (accepted['acceptedStamp'] as Map)[entry.key] == entry.value,
          )) {
        // A malformed/unknown receipt is ownership-ambiguous; do not release a
        // possibly transferred handle even in this disposable test process.
        throw StateError('unconfirmed transfer: $accepted');
      }
      stage = 'publication';
      Map<String, Object?>? rendered;
      for (var attempt = 0; attempt < 200; attempt++) {
        final status = await invoke('status', {'textureId': id});
        if (status['code'] != '') {
          throw StateError('GPU replay failed: $status');
        }
        if ((status['framesSubmitted'] as int) > 0 &&
            status['renderedStamp'] is Map &&
            stamp.entries.every(
              (entry) =>
                  (status['renderedStamp'] as Map)[entry.key] == entry.value,
            )) {
          rendered = status;
          break;
        }
        await Future<void>.delayed(const Duration(milliseconds: 25));
      }
      if (rendered == null || rendered['wholeFrameReadbacks'] != 0) {
        throw StateError('GPU publication/readback contract failed: $rendered');
      }
      final capabilities = owner.capabilities['presentation'] as Map?;
      final fullChecks = capabilities?['fullStampChecks'] == true;
      final checked = metadata.call(
        fullChecks ? 'snapshotCheck' : 'snapshotInfo',
        snapshot,
        fullChecks ? stamp : null,
      );
      if (checked['ok'] != true ||
          (fullChecks
                  ? checked['current']
                  : (checked['info'] as Map)['current']) !=
              true) {
        throw StateError('published snapshot is stale: $checked');
      }
      stage = 'close';
      final closed = await closeTexture(id);
      if (closed['snapshotReleaseFailures'] != 0 ||
          closed['liveRasterSlots'] != 0 ||
          closed['pendingRetireSlots'] != 0 ||
          metadata.call('snapshotInfo', snapshot)['ok'] == true) {
        throw StateError('GPU/snapshot cleanup failed: $closed');
      }
      await document.dispose();
      document = null;
      stdout.writeln(
        'RHWP_LINUX_GPU_PROBE ${jsonEncode({'ok': true, 'expectedUnsupported': false, 'warmup': warmup, 'stamp': stamp, 'fullStampChecks': fullChecks, 'sourceCheck': checked, 'rendered': rendered, 'closed': closed, 'hardwareAccelerationQualified': false})}',
      );
      exit(0);
    } catch (error, stack) {
      if (ownedSnapshot != null && metadata != null) {
        await metadata.release(ownedSnapshot);
      }
      await document?.dispose();
      stderr.writeln(
        'RHWP_LINUX_GPU_PROBE ${jsonEncode({'ok': false, 'stage': stage, 'error': '$error', 'stack': '$stack'})}',
      );
      exit(1);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    body: texture == null ? const SizedBox() : Texture(textureId: texture!),
  );
}
