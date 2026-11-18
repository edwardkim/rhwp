import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/src/native_document.dart';
import 'package:rhwp_native_reference/src/native_presentation.dart';
import 'package:rhwp_native_reference/src/native_texture_view.dart';

import 'fake_transport.dart';

class GpuTransport extends FakeTransport
    implements NativePresentationOwnerIdentity {
  String ownerLibraryPath = '/canonical/librhwp_native_ffi.so';
  @override
  String get nativeLibraryPath => ownerLibraryPath;
  GpuTransport() {
    capabilities.addAll({
      'ownerPresentationCleanup': true,
      'presentation': {
        'workerRenderSnapshots': true,
        'crossThreadHandles': true,
        'fullStampChecks': true,
      },
    });
  }
  bool autoSnapshots = true;
  int nextSnapshot = 100;
  final snapshots = <int>[];
  final waitingSnapshots = <Map<String, Object?>>[];
  final presentationCalls = <String>[];
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    dynamic data,
  }) async {
    if (request['op'] == 'presentationConfigure') {
      presentationCalls.add('configure');
      return {'ok': true};
    }
    if (request['op'] == 'workerSubmit' &&
        (request['command'] as Map?)?['op'] == 'exportRenderSnapshot') {
      presentationCalls.add('capture');
      final id = ++serial, snapshot = ++nextSnapshot;
      snapshots.add(snapshot);
      final completion = <String, Object?>{
        'ok': true,
        'requestId': id,
        'kind': 'renderSnapshot',
        'snapshot': snapshot,
        'info': {...info, 'current': true, 'profile': 'screen'},
      };
      if (autoSnapshots) {
        completions.add(completion);
      } else {
        waitingSnapshots.add(completion);
      }
      return {'ok': true, 'requestId': id};
    }
    return super.hostRequest(request, data: data);
  }

  void completeSnapshot() {
    completions.add(waitingSnapshots.removeAt(0));
  }
}

class FakePresentation implements NativePresentation {
  @override
  String get libraryPath => '/canonical/librhwp_native_ffi.so';
  @override
  bool available = true;
  bool current = true;
  final released = <int>[];
  final checks = <NativePresentationStamp>[];
  Completer<bool>? checkGate;
  @override
  Future<Map<String, Object?>> snapshotInfo(int snapshot) async => {
    'snapshot': snapshot,
    'current': current,
  };
  @override
  Future<bool> snapshotCheck(
    int snapshot,
    NativePresentationStamp stamp,
  ) async {
    checks.add(stamp);
    final gate = checkGate;
    if (gate != null) return await gate.future;
    return current;
  }

  @override
  Future<void> releaseSnapshot(int snapshot) async {
    if (released.contains(snapshot)) {
      throw StateError('snapshot released twice');
    }
    released.add(snapshot);
  }
}

class FakeTexturePlugin {
  FakeTexturePlugin(this.platform);
  final NativeTexturePlatform platform;
  final calls = <String>[];
  final submissions = <Map<String, Object?>>[];
  final covers = <Map<String, Object?>>[];
  final retireAcks = <Map<String, Object?>>[];
  final owned = <int>{}, releasedByPlugin = <int>{};
  int nextTexture = 1, serial = 0;
  bool publish = true, closed = false, ready = true;
  String? submitError, statusError;
  bool acceptedBeforeReply = false;
  bool covered = true, rejectUncover = false;
  int visibilitySerial = 0;
  Completer<void>? createGate, submitGate, statusGate;
  Completer<void>? coverGate, uncoverGate, uncoverAckGate;
  Completer<void>? retireAckGate;
  Completer<void>? closeReplyGate;
  String? retireAckError;
  Map<String, Object?> retireAckOverrides = {};
  Map<String, Object?>? acceptedStamp;
  List<int> acceptedIds = [];
  void install() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(MethodChannel(platform.channelName), handle);
  }

  void uninstall() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(MethodChannel(platform.channelName), null);
  }

  Map<String, Object?> status() => {
    'deviceGeneration': 1,
    'ready': ready,
    'registered': !closed,
    'closed': closed,
    'closing': closed,
    'ownerFinished': closed,
    'cleanupComplete': closed,
    if (platform == NativeTexturePlatform.macosMetalView)
      'nativeViewLayerDetached': closed,
    'code': statusError ?? '',
    'errorCode': statusError ?? '',
    'error': statusError == null ? '' : 'strict replay failed',
    'acceptedSerial': serial,
    'acceptedStamp': acceptedStamp,
    'acceptedSnapshots': acceptedIds,
    'hasFrame':
        publish && acceptedStamp != null && !closed && statusError == null,
    'renderedSerial': publish ? serial : 0,
    'renderedStamp': publish ? acceptedStamp : null,
    'limits': {
      'continuousPresentationSupported':
          platform == NativeTexturePlatform.linux ||
          platform == NativeTexturePlatform.macosMetalView,
      'consumerRetirement': false,
      'maxFrames': 4,
      'outstandingFrames': owned.length,
    },
  };
  void accept(Map<String, Object?> arguments) {
    acceptedStamp = Map<String, Object?>.from(arguments['stamp']! as Map);
    acceptedIds = (arguments['placements']! as List)
        .map((value) => (value as Map)['snapshot']! as int)
        .toList();
    releasedByPlugin.addAll(owned.difference(acceptedIds.toSet()));
    owned
      ..clear()
      ..addAll(acceptedIds);
    ++serial;
  }

  Future<Object?> handle(MethodCall call) async {
    calls.add(call.method);
    final arguments = call.arguments == null
        ? <String, Object?>{}
        : Map<String, Object?>.from(call.arguments as Map);
    switch (call.method) {
      case 'create':
        await createGate?.future;
        return {
          ...status(),
          'textureId': nextTexture++,
          if (platform == NativeTexturePlatform.macosMetalView) ...{
            'ready': false,
            'presentationKind': 'nativeView',
            'viewType': 'rhwp_native/metal_view',
          },
        };
      case 'status':
        await statusGate?.future;
        return status();
      case 'submit':
        submissions.add(arguments);
        if (acceptedBeforeReply) accept(arguments);
        await submitGate?.future;
        if (closed && !acceptedBeforeReply) {
          throw PlatformException(code: 'closing');
        }
        if (submitError != null) throw PlatformException(code: submitError!);
        if (!acceptedBeforeReply) accept(arguments);
        return {...status(), 'accepted': true};
      case 'setCovered':
        covers.add(arguments);
        final hide = arguments['covered'] == true;
        if (hide) {
          await coverGate?.future;
        } else {
          await uncoverGate?.future;
        }
        final sequence = arguments['visibilitySerial']! as int;
        final latest = sequence >= visibilitySerial;
        if (latest) visibilitySerial = sequence;
        final matches =
            acceptedStamp != null &&
            NativePresentationStamp.fromJson(acceptedStamp!)
                .matches(arguments['stamp']);
        final accepted =
            latest &&
            !closed &&
            (hide || (!rejectUncover && publish && matches));
        if (accepted) covered = hide;
        final result = {
          'textureId': arguments['textureId'],
          'covered': covered,
          'visibilitySerial': sequence,
          'accepted': accepted,
          'renderedStamp': publish ? acceptedStamp : null,
        };
        if (!hide) await uncoverAckGate?.future;
        return result;
      case 'close':
        closed = true;
        covered = true;
        releasedByPlugin.addAll(owned);
        owned.clear();
        await closeReplyGate?.future;
        return status();
      case 'retireAck':
        retireAcks.add(arguments);
        await retireAckGate?.future;
        if (retireAckError != null) {
          throw PlatformException(code: retireAckError!);
        }
        return {
          'textureId': arguments['textureId'],
          'deviceGeneration': 1,
          'acknowledged': closed,
          'closed': closed,
          'ownerFinished': closed,
          'registered': !closed,
          'nativeViewLayerDetached': closed,
          ...retireAckOverrides,
        };
    }
    throw MissingPluginException();
  }
}

Future<NativeDocument> gpuDocument(GpuTransport transport) async {
  final document = await NativeDocument.connect(transport);
  await document.newDocument();
  return document;
}
