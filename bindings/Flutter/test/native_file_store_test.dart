import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:rhwp_native_reference/rhwp_native.dart';

import 'support/fake_transport.dart';

class SaveTransport extends FakeTransport {
  Uint8List serialized = Uint8List.fromList(utf8.encode('prepared document'));
  int losses = 0;
  int? savedRevision;
  bool ackFailure = false;
  bool malformedReport = false;
  bool copyFailure = false;
  bool cancelFailure = false;
  int? byteLength;
  bool recovered = false;
  Completer<Uint8List>? saveCopyGate;
  int prepares = 0, cancels = 0, acks = 0;
  final requestBodies = <Map<String, Object?>>[];
  @override
  Map<String, Object?> get info => {
    ...super.info,
    'savedRevision': savedRevision,
    'isDirty': savedRevision != revision,
  };
  Map<String, Object?> status() => {
    'ok': true,
    'kind': 'saveStatus',
    'documentId': documentId,
    'isDirty': savedRevision != revision,
    'status': {
      'revision': revision,
      'savedRevision': savedRevision,
      'dirty': savedRevision != revision,
    },
  };
  @override
  Future<Map<String, Object?>> hostRequest(
    Map<String, Object?> request, {
    Uint8List? data,
  }) async {
    requestBodies.add(Map.of(request));
    if (request['op'] == 'workerOpenRecovery') {
      recovered = true;
      savedRevision = null;
      final result = await super.hostRequest({
        ...request,
        'op': 'workerOpen',
      }, data: data);
      return result;
    }
    if (request['op'] == 'workerSubmit') {
      final command = request['command']! as Map;
      final op = command['op'];
      if (op == 'prepareSave' ||
          op == 'cancelSave' ||
          op == 'acknowledgeSave') {
        commands.add(op! as String);
        final id = ++serial;
        Map<String, Object?> completion;
        if (op == 'prepareSave') {
          ++prepares;
          completion = {
            'ok': true,
            'kind': 'preparedSave',
            'documentId': documentId,
            'token': 17,
            'revision': revision,
            'buffer': 70,
            'byteLength': byteLength ?? serialized.length,
            'report': malformedReport ? <String, Object?>{} : {'count': losses},
          };
        } else if (op == 'cancelSave') {
          ++cancels;
          completion = cancelFailure
              ? {
                  'ok': false,
                  'code': 'cancelFailed',
                  'error': 'injected cleanup failure',
                }
              : status();
        } else {
          ++acks;
          if (ackFailure) {
            completion = {
              'ok': false,
              'code': 'ackFailure',
              'error': 'injected acknowledgement failure',
            };
          } else {
            // The protocol acknowledges the prepared revision, not a later edit.
            savedRevision = preparedRevision;
            completion = status();
          }
        }
        completions.add({...completion, 'requestId': id});
        return {'ok': true, 'requestId': id};
      }
    }
    return super.hostRequest(request, data: data);
  }

  int preparedRevision = 0;
  @override
  Future<Uint8List> copyAndReleaseBuffer(int buffer, int byteLength) async {
    if (buffer != 70) return super.copyAndReleaseBuffer(buffer, byteLength);
    copied.add(buffer);
    preparedRevision = revision;
    try {
      if (copyFailure) throw StateError('copy failed');
      return saveCopyGate == null
          ? Uint8List.fromList(serialized)
          : await saveCopyGate!.future;
    } finally {
      released.add(buffer);
    }
  }
}

Future<(NativeDocument, NativeFileStore, SaveTransport)> fixture() async {
  final transport = SaveTransport();
  final document = await NativeDocument.connect(transport);
  await document.newDocument();
  addTearDown(document.dispose);
  return (document, NativeFileStore(document), transport);
}

void main() {
  late Directory directory;
  setUp(() async {
    directory = await Directory.systemTemp.createTemp('rhwp-dart-save-');
  });
  tearDown(() async {
    await directory.delete(recursive: true);
  });
  String path(String name) => '${directory.path}${Platform.pathSeparator}$name';

  test('new-file publication copies and releases before acknowledging exact checkpoint', () async {
    final (document, store, transport) = await fixture();
    final result = await store.save(destination: path('new.hwpx'));
    expect(result.cancelled, isFalse);
    expect(result.savedRevision, 0);
    expect(result.info.isDirty, isFalse);
    expect(await File(path('new.hwpx')).readAsBytes(), transport.serialized);
    expect(transport.copied, [70]);
    expect(transport.released, [70]);
    expect(transport.acks, 1);
    expect(transport.cancels, 0);
    expect(store.sourcePath, path('new.hwpx'));
    expect(document.info!.isDirty, isFalse);
    expect(await directory.list().length, 1);
  });

  test('loss rejection cancels same token and publishes no bytes', () async {
    final (_, store, transport) = await fixture();
    transport.losses = 1;
    final result = await store.save(
      destination: path('loss.hwpx'),
      approveLoss: (report) async {
        expect(report['count'], 1);
        return false;
      },
    );
    expect(result.cancelled, isTrue);
    expect(transport.prepares, 1);
    expect(transport.cancels, 1);
    expect(transport.acks, 0);
    expect(await File(path('loss.hwpx')).exists(), isFalse);
    expect(store.isBusy, isFalse);
  });

  test(
    'edit during loss approval keeps same bytes and later document dirty',
    () async {
      final (document, store, transport) = await fixture();
      transport.losses = 1;
      final result = await store.save(
        destination: path('race.hwpx'),
        approveLoss: (_) async {
          transport.revision = 1;
          await document.refreshInfo();
          return true;
        },
      );
      expect(result.savedRevision, 0);
      expect(result.info.revision, 1);
      expect(result.info.isDirty, isTrue);
      expect(transport.prepares, 1);
      expect(await File(path('race.hwpx')).readAsBytes(), transport.serialized);
    },
  );

  test(
    'opened source external change is preserved before native prepare',
    () async {
      final (_, store, transport) = await fixture();
      final file = await File(path('source.hwpx')).writeAsString('old');
      await store.openPath(file.path);
      await file.writeAsString('external');
      await expectLater(store.save(), throwsA(isA<NativeFileChanged>()));
      expect(await file.readAsString(), 'external');
      expect(transport.prepares, 0);
    },
  );

  test(
    'existing other destination needs explicit observed-stamp approval',
    () async {
      final (_, store, transport) = await fixture();
      final file = await File(path('existing.hwpx')).writeAsString('existing');
      final result = await store.save(destination: file.path);
      expect(result.cancelled, isTrue);
      expect(transport.prepares, 0);
      await expectLater(
        store.save(
          destination: file.path,
          approveOverwrite: (_) async {
            await file.writeAsString('external winner');
            return true;
          },
        ),
        throwsA(isA<NativeFileChanged>()),
      );
      expect(await file.readAsString(), 'external winner');
      expect(transport.cancels, 1);
    },
  );

  test(
    'post-publication ack failure preserves full destination and dirty state',
    () async {
      final (document, store, transport) = await fixture();
      transport.ackFailure = true;
      await expectLater(
        store.save(destination: path('published.hwpx')),
        throwsA(
          isA<NativePublicationError>().having(
            (e) => e.path,
            'preserved path',
            path('published.hwpx'),
          ),
        ),
      );
      expect(
        await File(path('published.hwpx')).readAsBytes(),
        transport.serialized,
      );
      expect(document.info!.isDirty, isTrue);
      expect(store.sourcePath, isNull);
      expect(transport.cancels, 1);
    },
  );

  test(
    'malformed report and copy failure release buffer and cancel token',
    () async {
      final (document, store, transport) = await fixture();
      transport.malformedReport = true;
      await expectLater(
        store.save(destination: path('bad.hwpx')),
        throwsA(isA<NativeCallException>()),
      );
      await waitFor(() => transport.cancels == 1);
      expect(transport.released, [70]);
      await document.refreshInfo();
      transport.malformedReport = false;
      transport.copyFailure = true;
      await expectLater(document.prepareSave('hwpx'), throwsStateError);
      await waitFor(() => transport.cancels == 2);
      expect(transport.released, [70, 70]);
      expect(await File(path('bad.hwpx')).exists(), isFalse);
    },
  );

  test('recovery uses separate artifact, cancels checkpoint and reopens dirty without source', () async {
    final (document, store, transport) = await fixture();
    final artifact = await store.writeRecovery(path('recovery'));
    expect(artifact, isNotNull);
    expect(transport.acks, 0);
    expect(transport.cancels, 1);
    expect(document.info!.isDirty, isTrue);
    expect(await store.writeRecovery(path('recovery')), isNull);
    await store.openRecovery(artifact!);
    expect(transport.recovered, isTrue);
    expect(document.info!.isDirty, isTrue);
    expect(store.sourcePath, isNull);
    expect(transport.opened.single, transport.serialized);
    final file = File(artifact);
    final bytes = await file.readAsBytes();
    bytes[bytes.length - 1] ^= 1;
    await file.writeAsBytes(bytes);
    await expectLater(store.openRecovery(artifact), throwsFormatException);
  });

  test('replacement snapshot rejects stale source before open and queued mutation before execution', () async {
    final (document, store, transport) = await fixture();
    final expected = document.info!;
    transport.revision++;
    await document.refreshInfo();
    expect(
      () => document.openBytes(Uint8List(1), replacing: expected),
      throwsA(isA<NativeCallException>()),
    );
    await expectLater(
      store.openPath(path('missing.hwpx'), replacing: expected),
      throwsA(anything),
    );
    final fresh = document.info!;
    transport.autoComplete = false;
    final queued = document.refreshInfo();
    final replacing = document.newDocument(replacing: fresh);
    final rejected = expectLater(
      replacing,
      throwsA(isA<NativeCallException>()),
    );
    await waitFor(() => transport.commands.last == 'info');
    transport.revision++;
    transport.completions.add({
      'ok': true,
      'kind': 'info',
      'requestId': transport.serial,
      'info': transport.info,
    });
    await queued;
    await rejected;
    expect(transport.commands.where((x) => x == 'new').length, 1);
  });

  test('source path resets when another document is created and whenIdle follows file operation', () async {
    final (document, store, transport) = await fixture();
    await store.save(destination: path('old.hwpx'));
    await document.newDocument();
    expect(store.sourcePath, isNull);
    transport.losses = 1;
    final gate = Completer<bool>();
    final saving = store.save(
      destination: path('wait.hwpx'),
      approveLoss: (_) => gate.future,
    );
    await waitFor(() => transport.prepares == 2);
    expect(store.isBusy, isTrue);
    var idle = false;
    final waiting = store.whenIdle.then((_) {
      idle = true;
    });
    await Future<void>.delayed(Duration.zero);
    expect(idle, isFalse);
    gate.complete(false);
    await saving;
    await waiting;
    expect(idle, isTrue);
  });
  test('dispose during prepared byte copy releases late buffer and returns no ticket', () async {
    final (document, _, transport) = await fixture();
    transport.saveCopyGate = Completer<Uint8List>();
    final prepared = document.prepareSave('hwpx');
    final failed = expectLater(
      prepared,
      throwsA(isA<NativeDisposedException>()),
    );
    await waitFor(() => transport.copied.contains(70));
    await document.dispose();
    await failed;
    transport.saveCopyGate!.complete(Uint8List.fromList(transport.serialized));
    await waitFor(() => transport.released.contains(70));
    expect(transport.closes, 1);
  });

  test(
    'cleanup has a reserved slot when normal FIFO admission is full',
    () async {
      final (document, _, transport) = await fixture();
      final prepared = await document.prepareSave('hwpx');
      transport.autoComplete = false;
      final queued = <Future<NativeDocumentInfo>>[];
      for (var i = 0; i < 64; i++) {
        queued.add(document.refreshInfo());
      }
      expect(document.refreshInfo, throwsA(isA<NativeCallException>()));
      final cleanup = document.cancelSave(prepared);
      await waitFor(() => transport.commands.last == 'info');
      // Complete only the operation that was already executing. Cleanup must be
      // selected before the 63 user reads waiting in the full queue.
      transport.completions.add({
        'ok': true,
        'kind': 'info',
        'requestId': transport.serial,
        'info': transport.info,
      });
      await cleanup;
      expect(transport.commands.indexOf('cancelSave'), 3);
      for (final future in queued) {
        unawaited(future.catchError((Object _) => document.info!));
      }
      await document.dispose();
    },
  );
  test('concurrent cancellation shares one reserved slot and blocks acknowledgement', () async {
    final (document, _, transport) = await fixture();
    final prepared = await document.prepareSave('hwpx');
    transport.autoComplete = false;
    final blocked = document.refreshInfo();
    final a = document.cancelSave(prepared);
    for (var i = 0; i < 100; i++) {
      expect(identical(document.cancelSave(prepared), a), isTrue);
    }
    await expectLater(document.acknowledgeSave(prepared), throwsStateError);
    await waitFor(() => transport.commands.last == 'info');
    transport.completions.add({
      'ok': true,
      'kind': 'info',
      'requestId': transport.serial,
      'info': transport.info,
    });
    await blocked;
    await a;
    expect(transport.cancels, 1);
  });

  test('malformed prepare and failed cleanup stop save admission until source replacement', () async {
    final (document, _, transport) = await fixture();
    transport.malformedReport = true;
    transport.cancelFailure = true;
    await expectLater(
      document.prepareSave('hwpx'),
      throwsA(isA<NativeCallException>()),
    );
    await waitFor(() => transport.cancels == 1);
    await document.refreshInfo();
    expect(() => document.prepareSave('hwpx'), throwsStateError);
    expect(transport.prepares, 1);
    await document.newDocument();
    transport.malformedReport = false;
    transport.cancelFailure = false;
    final next = await document.prepareSave('hwpx');
    await document.cancelSave(next);
    expect(transport.prepares, 2);
  });

  test(
    'oversized prepared bytes are released and cancelled before copying',
    () async {
      final (document, _, transport) = await fixture();
      transport.byteLength = maxReferenceBytes + 1;
      await expectLater(
        document.prepareSave('hwpx'),
        throwsA(isA<NativeCallException>()),
      );
      await waitFor(() => transport.cancels == 1);
      expect(transport.copied, isEmpty);
      expect(transport.released, [70]);
    },
  );

  test(
    'RHWP-R1 recovery interoperates with existing Python writer and reader',
    () async {
      final (_, store, transport) = await fixture();
      const examples = String.fromEnvironment(
        'RHWP_NATIVE_EXAMPLES',
        defaultValue: '../Native/examples',
      );
      final pythonPath = path('python.rhwp-recovery');
      final generated = await Process.run('python3', [
        '-c',
        'import sys; sys.path.insert(0,sys.argv[1]); from native_file_store import publish_recovery; publish_recovery(sys.argv[2],b"from Python",7,{"count":0})',
        Directory(examples).absolute.path,
        pythonPath,
      ]);
      expect(generated.exitCode, 0, reason: generated.stderr.toString());
      await store.openRecovery(pythonPath);
      expect(utf8.decode(transport.opened.single), 'from Python');
      final artifact = await store.writeRecovery(path('dart-recovery'));
      final inspected = await Process.run('python3', [
        '-c',
        'import sys,json; sys.path.insert(0,sys.argv[1]); from native_file_store import read_recovery; h,b=read_recovery(sys.argv[2]); print(json.dumps({"revision":h["revision"],"text":b.decode()}))',
        Directory(examples).absolute.path,
        artifact!,
      ]);
      expect(inspected.exitCode, 0, reason: inspected.stderr.toString());
      expect(jsonDecode(inspected.stdout as String), {
        'revision': transport.revision,
        'text': utf8.decode(transport.serialized),
      });
    },
  );
  test(
    'new hard-link publication stamp supports an immediate second save',
    () async {
      final (_, store, transport) = await fixture();
      await store.save(destination: path('twice.hwpx'));
      transport.serialized = Uint8List.fromList(
        utf8.encode('second publication'),
      );
      final second = await store.save();
      expect(second.cancelled, isFalse);
      expect(
        await File(path('twice.hwpx')).readAsString(),
        'second publication',
      );
      expect(transport.acks, 2);
    },
  );

  test(
    'recovery post-publication cleanup failure exposes preserved artifact',
    () async {
      final (document, store, transport) = await fixture();
      transport.cancelFailure = true;
      NativePublicationError? failure;
      try {
        await store.writeRecovery(path('preserved-recovery'));
      } on NativePublicationError catch (error) {
        failure = error;
      }
      expect(failure, isNotNull);
      expect(failure!.published, isTrue);
      expect(await File(failure.path).exists(), isTrue);
      expect(document.info!.isDirty, isTrue);
      expect(transport.acks, 0);
      expect(() => document.prepareSave('hwpx'), throwsStateError);
    },
  );
  test(
    'save source identity survives queued replacement before info completes',
    () async {
      final (document, store, transport) = await fixture();
      await store.save(destination: path('source-identity.hwpx'));
      final original = await File(path('source-identity.hwpx')).readAsBytes();
      transport.serialized = Uint8List.fromList(
        utf8.encode('replacement document'),
      );
      transport.autoComplete = false;
      final blocked = document.refreshInfo();
      final blockedId = transport.serial;
      final replacement = document.newDocument();
      final saving = store.save(approveOverwrite: (_) async => true);
      final rejected = expectLater(saving, throwsA(isA<NativeCallException>()));
      transport.autoComplete = true;
      transport.completions.add({
        'ok': true,
        'kind': 'info',
        'requestId': blockedId,
        'info': transport.info,
      });
      await blocked;
      await replacement;
      await rejected;
      expect(await File(path('source-identity.hwpx')).readAsBytes(), original);
      expect(transport.prepares, 1);
      expect(transport.acks, 1);
    },
  );

  test('recovery source identity survives queued replacement before info completes', () async {
    final (document, store, transport) = await fixture();
    transport.autoComplete = false;
    final blocked = document.refreshInfo();
    final blockedId = transport.serial;
    final replacement = document.newDocument();
    final recovering = store.writeRecovery(path('recovery-identity'));
    final rejected = expectLater(
      recovering,
      throwsA(isA<NativeCallException>()),
    );
    transport.autoComplete = true;
    transport.completions.add({
      'ok': true,
      'kind': 'info',
      'requestId': blockedId,
      'info': transport.info,
    });
    await blocked;
    await replacement;
    await rejected;
    expect(transport.prepares, 0);
    expect(transport.acks, 0);
    expect(await Directory(path('recovery-identity')).exists(), isFalse);
  });
}
